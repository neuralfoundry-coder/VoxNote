use std::sync::Arc;
use tauri::{Emitter, State};
use tokio::sync::{mpsc, watch};
use uuid::Uuid;
use voxnote_core::audio::ringbuf::AudioRingBuffer;
use voxnote_core::config::AppConfig;
use voxnote_core::models::{Note, RecordingState};
use voxnote_core::pipeline::{PipelineEvent, TranscriptionPipeline};

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

    // 파이프라인 시작
    let config = state.config.lock().map_err(|e| e.to_string())?.clone();
    let store = state.store.clone();
    let note_id_clone = note_id.clone();

    tokio::spawn(async move {
        run_recording_pipeline(app, config, store, note_id_clone).await;
    });

    Ok(RecordingResponse {
        session_id,
        note_id,
        state: "recording".to_string(),
    })
}

#[tauri::command]
pub async fn stop_recording(state: State<'_, AppState>) -> Result<String, String> {
    let mut rec_state = state.recording_state.lock().map_err(|e| e.to_string())?;
    *rec_state = RecordingState::Stopped;

    // 파이프라인 종료 신호
    let tx_lock = SHUTDOWN_TX.get_or_init(|| std::sync::Mutex::new(None));
    if let Ok(mut tx) = tx_lock.lock() {
        if let Some(sender) = tx.take() {
            let _ = sender.send(true);
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
) {
    use voxnote_core::audio::resample::Resampler;
    use voxnote_core::audio::vad::{EnergyVad, VoiceActivityDetector};
    use voxnote_core::audio::accumulator::Accumulator;

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
    let note_id_clone = note_id.clone();
    tokio::spawn(async move {
        while let Some(event) = event_rx.recv().await {
            match &event {
                PipelineEvent::Segment(segment) => {
                    // DB에 저장
                    let _ = store_clone.insert_segment(segment);
                    // Frontend에 이벤트 전송
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

    // 파이프라인 실행
    let pipeline = TranscriptionPipeline::new(config.clone(), event_tx);
    let buf = AudioRingBuffer::new(500);
    let consumer = buf.consumer();

    // 오디오 캡처 (cpal)
    #[cfg(feature = "desktop")]
    {
        use voxnote_core::audio::capture::CpalCapture;

        let producer = buf.producer();
        let mut capture = match CpalCapture::new_default() {
            Ok(c) => c,
            Err(e) => {
                tracing::error!("Audio capture failed: {}", e);
                return;
            }
        };

        let sample_rate = capture.sample_rate();
        let channels = capture.channels();

        if let Err(e) = capture.start(move |data, sr, ch| {
            // 스테레오 → 모노 다운믹스 후 push
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

        // 처리 루프 (shutdown까지 실행)
        pipeline
            .run_processing_loop(consumer, note_id, shutdown_rx)
            .await;

        capture.stop();
    }

    #[cfg(not(feature = "desktop"))]
    {
        tracing::warn!("Desktop audio capture not available on this platform");
        pipeline
            .run_processing_loop(consumer, note_id, shutdown_rx)
            .await;
    }
}
