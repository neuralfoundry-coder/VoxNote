use std::sync::Arc;
use tauri::{Emitter, State};
use tokio::sync::{mpsc, watch};
use uuid::Uuid;
use voxnote_core::audio::ringbuf::AudioRingBuffer;
use voxnote_core::config::AppConfig;
use voxnote_core::models::{Note, NoteStatus, RecordingState};
use voxnote_core::pipeline::{PipelineEvent, TranscriptionPipeline};
use voxnote_core::stt::SttProvider;

use crate::state::AppState;

#[derive(serde::Serialize, Clone)]
pub struct RecordingResponse {
    pub session_id: String,
    pub note_id: String,
    pub state: String,
}

/// 전역 녹음 세션 (한 번에 하나만)
static SHUTDOWN_TX: std::sync::OnceLock<std::sync::Mutex<Option<watch::Sender<bool>>>> =
    std::sync::OnceLock::new();

/// 현재 녹음 중인 note_id
static CURRENT_NOTE_ID: std::sync::OnceLock<std::sync::Mutex<Option<String>>> =
    std::sync::OnceLock::new();

#[tauri::command]
pub async fn start_recording(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<RecordingResponse, String> {
    let mut rec_state = state.recording_state.lock().map_err(|e| e.to_string())?;
    if *rec_state == RecordingState::Recording {
        return Err("Already recording".to_string());
    }

    // 새 노트 생성
    let note = Note::new(format!(
        "Recording {}",
        chrono::Local::now().format("%Y-%m-%d %H:%M")
    ));
    let note_id = note.id.clone();
    state.store.insert_note(&note).map_err(|e| e.to_string())?;

    *rec_state = RecordingState::Recording;
    let session_id = Uuid::new_v4().to_string();

    // 현재 녹음 note_id 저장
    let note_lock = CURRENT_NOTE_ID.get_or_init(|| std::sync::Mutex::new(None));
    if let Ok(mut n) = note_lock.lock() {
        *n = Some(note_id.clone());
    }

    // 파이프라인 시작 — config, store, model_path만 전달
    // STT 모델 로드는 녹음 스레드 내부에서 수행 (Tauri 명령 즉시 반환)
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let store = state.store.clone();
    let note_id_clone = note_id.clone();
    let stt_model_path = state.stt_model_path.lock().map_err(|e| e.to_string())?.clone();
    let stt_provider_cache = state.stt_provider.clone();

    // CpalCapture의 Stream이 !Send이므로 전용 OS 스레드에서 실행
    std::thread::Builder::new()
        .name("voxnote-recording".to_string())
        .spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .expect("Failed to build tokio runtime for recording");
            rt.block_on(run_recording_pipeline(
                app,
                config,
                store,
                note_id_clone,
                stt_model_path,
                stt_provider_cache,
            ));
        })
        .map_err(|e| format!("Failed to spawn recording thread: {}", e))?;

    Ok(RecordingResponse {
        session_id,
        note_id,
        state: "recording".to_string(),
    })
}

#[tauri::command]
pub async fn stop_recording(
    app: tauri::AppHandle,
    state: State<'_, AppState>,
) -> Result<String, String> {
    let mut rec_state = state.recording_state.lock().map_err(|e| e.to_string())?;
    *rec_state = RecordingState::Stopped;

    // 파이프라인 종료 신호
    let tx_lock = SHUTDOWN_TX.get_or_init(|| std::sync::Mutex::new(None));
    if let Ok(mut tx) = tx_lock.lock() {
        if let Some(sender) = tx.take() {
            let _ = sender.send(true);
        }
    }

    // Note 상태를 Done으로 업데이트
    let note_lock = CURRENT_NOTE_ID.get_or_init(|| std::sync::Mutex::new(None));
    if let Ok(mut n) = note_lock.lock() {
        if let Some(note_id) = n.take() {
            if let Ok(Some(mut note)) = state.store.get_note(&note_id) {
                note.status = NoteStatus::Done;
                note.updated_at = chrono::Utc::now();
                let _ = state.store.update_note(&note);
                tracing::info!("Note {} status updated to Done", note_id);
            }
            // 프론트엔드에 노트 목록 갱신 이벤트
            let _ = app.emit("notes:updated", ());
        }
    }

    Ok("stopped".to_string())
}

#[tauri::command]
pub async fn pause_recording(state: State<'_, AppState>) -> Result<String, String> {
    let mut rec_state = state.recording_state.lock().map_err(|e| e.to_string())?;
    match *rec_state {
        RecordingState::Recording => {
            *rec_state = RecordingState::Paused;
            Ok("paused".to_string())
        }
        RecordingState::Paused => {
            *rec_state = RecordingState::Recording;
            Ok("resumed".to_string())
        }
        _ => Err("Not recording".to_string()),
    }
}

/// 실제 오디오 캡처 + 전사 파이프라인
async fn run_recording_pipeline(
    app: tauri::AppHandle,
    config: AppConfig,
    store: Arc<voxnote_core::storage::SqliteStore>,
    note_id: String,
    stt_model_path: Option<std::path::PathBuf>,
    stt_provider_cache: Arc<std::sync::Mutex<Option<Arc<dyn SttProvider>>>>,
) {
    // STT Provider 지연 로드 (녹음 스레드에서 수행 — UI 블로킹 없음)
    let stt_provider: Option<Arc<dyn SttProvider>> = {
        let mut guard = stt_provider_cache.lock().unwrap_or_else(|e| e.into_inner());
        if guard.is_none() {
            if let Some(ref path) = stt_model_path {
                let provider_type = config.stt.provider.as_deref()
                    .unwrap_or_else(|| crate::state::infer_stt_provider_type(path));
                *guard = crate::state::load_stt_provider(path, provider_type);
            }
        }
        guard.clone()
    };

    if let Some(ref p) = stt_provider {
        tracing::info!("Recording with STT provider: {}", p.name());
    } else {
        tracing::warn!("No STT provider — recording without transcription");
    }

    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<PipelineEvent>();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // Shutdown TX 저장
    let tx_lock = SHUTDOWN_TX.get_or_init(|| std::sync::Mutex::new(None));
    if let Ok(mut tx) = tx_lock.lock() {
        *tx = Some(shutdown_tx);
    }

    // 파이프라인 이벤트 → Tauri 이벤트 + DB 저장
    let app_clone = app.clone();
    let store_clone = store.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match &event {
                PipelineEvent::Segment(segment) => {
                    let _ = store_clone.insert_segment(segment);
                    let _ = app_clone.emit("stt:segment", segment);
                }
                PipelineEvent::RecordingStatus { status } => {
                    let _ = app_clone.emit("recording:status", status);
                }
                PipelineEvent::Error { message } => {
                    tracing::error!("Pipeline error: {}", message);
                    let _ = app_clone.emit("pipeline:error", message);
                }
            }
        }
    });

    // 오디오 캡처 먼저 생성하여 실제 디바이스 rate 확인
    use voxnote_core::audio::capture::CpalCapture;

    let mut capture = match CpalCapture::new_default() {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Audio capture failed: {}", e);
            return;
        }
    };

    // 실제 디바이스 rate로 config 갱신 후 파이프라인 생성
    let mut config = config;
    config.audio.sample_rate = capture.sample_rate();
    tracing::info!(
        "Audio capture: rate={}, channels={}",
        capture.sample_rate(),
        capture.channels()
    );

    let pipeline = TranscriptionPipeline::new(config, event_tx, stt_provider);
    let buf = AudioRingBuffer::new(500);
    let consumer = buf.consumer();
    let producer = buf.producer();

    if let Err(e) = capture.start(move |data, _sr, ch| {
        let mono: Vec<f32> = if ch > 1 {
            data.chunks_exact(ch as usize)
                .map(|frame| frame.iter().sum::<f32>() / ch as f32)
                .collect()
        } else {
            data.to_vec()
        };
        producer.push(mono);
    }) {
        tracing::error!("Audio start failed: {}", e);
        return;
    }

    tracing::info!("Recording pipeline started for note {}", note_id);

    // 처리 루프 (shutdown까지 실행)
    pipeline
        .run_processing_loop(consumer, note_id, shutdown_rx)
        .await;

    capture.stop();
    tracing::info!("Recording pipeline ended");
}
