use std::sync::Arc;
use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::audio::accumulator::Accumulator;
use crate::audio::resample::Resampler;
use crate::audio::vad::{EnergyVad, VoiceActivityDetector};
use crate::config::AppConfig;
use crate::models::Segment;
use crate::stt::SttProvider;

/// 파이프라인 이벤트 — Tauri 이벤트로 변환
#[derive(Debug, Clone, serde::Serialize)]
#[serde(tag = "type")]
pub enum PipelineEvent {
    /// STT 세그먼트 생성
    Segment(Segment),
    /// 녹음 상태 변경
    RecordingStatus { status: String },
    /// 에러 발생
    Error { message: String },
}

/// 실시간 전사 파이프라인
///
/// 오디오 캡처 → 리샘플링 → VAD → 어큐뮬레이터 → STT → 이벤트
pub struct TranscriptionPipeline {
    config: AppConfig,
    event_tx: mpsc::UnboundedSender<PipelineEvent>,
    stt_provider: Option<Arc<dyn SttProvider>>,
}

impl TranscriptionPipeline {
    pub fn new(
        config: AppConfig,
        event_tx: mpsc::UnboundedSender<PipelineEvent>,
        stt_provider: Option<Arc<dyn SttProvider>>,
    ) -> Self {
        Self {
            config,
            event_tx,
            stt_provider,
        }
    }

    /// SttProvider를 통해 실제 전사를 수행하고 세그먼트를 emit
    async fn transcribe_chunk(
        &self,
        chunk: crate::audio::AudioChunk,
        note_id: &str,
    ) {
        let Some(provider) = &self.stt_provider else {
            // Provider 없으면 placeholder emit
            let _ = self.event_tx.send(PipelineEvent::Segment(
                Segment::new(
                    note_id,
                    &format!("[audio chunk: {:.1}s]", chunk.duration_secs()),
                    chunk.timestamp_ms,
                    chunk.timestamp_ms + (chunk.duration_secs() * 1000.0) as i64,
                ),
            ));
            return;
        };

        // whisper 전사 실행 (동기 블로킹 — current_thread runtime에서 직접 호출)
        info!("Whisper transcribe start: {:.1}s audio", chunk.duration_secs());
        match provider.transcribe(&chunk, note_id).await {
            Ok(segments) => {
                info!("Whisper transcribe done: {} segments", segments.len());
                // 문맥 연속성: 마지막 세그먼트 텍스트를 initial_prompt로 설정
                if !segments.is_empty() {
                    let context: String = segments
                        .iter()
                        .rev()
                        .take(3)
                        .map(|s| s.text.as_str())
                        .collect::<Vec<_>>()
                        .into_iter()
                        .rev()
                        .collect::<Vec<_>>()
                        .join(" ");
                    if let Some(ref p) = self.stt_provider {
                        p.set_initial_prompt(&context);
                    }
                }

                for segment in segments {
                    let _ = self.event_tx.send(PipelineEvent::Segment(segment));
                }
            }
            Err(e) => {
                warn!("STT transcription error: {}", e);
                let _ = self.event_tx.send(PipelineEvent::Error {
                    message: format!("STT error: {}", e),
                });
            }
        }
    }

    /// 처리 루프 실행 (별도 태스크에서 호출)
    ///
    /// ring buffer에서 오디오를 소비하여 리샘플링 → VAD → 어큐뮬레이터 → STT 전사
    pub async fn run_processing_loop(
        &self,
        consumer: crate::audio::ringbuf::RingBufferConsumer,
        note_id: String,
        mut shutdown_rx: tokio::sync::watch::Receiver<bool>,
    ) {
        let input_rate = self.config.audio.sample_rate;
        let channels = 1u16; // mono after capture

        let mut resampler = match Resampler::new(input_rate, channels) {
            Ok(r) => r,
            Err(e) => {
                let _ = self.event_tx.send(PipelineEvent::Error {
                    message: format!("Resampler init failed: {}", e),
                });
                return;
            }
        };

        // VAD 임계값을 낮게 설정 — 마이크 입력이 리샘플링 후 진폭이 작으므로
        // 기본값 0.01 RMS는 너무 높음. 0.002로 낮춰 대부분의 음성이 통과하도록 함.
        let mut vad = EnergyVad::new(0.3); // probability threshold
        vad.set_energy_floor(0.002); // 매우 낮은 에너지도 감지

        let mut accumulator = Accumulator::new(
            self.config.audio.window_size_secs,
            self.config.audio.overlap_secs,
            16000,
        );

        let vad_frame_size = 480; // 30ms at 16kHz
        let mut silence_counter = 0u32; // VAD 미통과 연속 횟수

        // config에서 언어 설정을 STT provider에 전달
        if let Some(ref provider) = self.stt_provider {
            let lang = self.config.stt.language.as_deref();
            provider.set_language(lang);
            info!(
                "Processing loop started for note {} (STT: {}, lang: {})",
                note_id,
                provider.name(),
                lang.unwrap_or("auto")
            );
        } else {
            info!("Processing loop started for note {} (STT: none)", note_id);
        }

        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        // 잔여 버퍼 플러시 → STT 전사
                        if let Some(chunk) = accumulator.flush() {
                            self.transcribe_chunk(chunk, &note_id).await;
                        }
                        info!("Processing loop stopped for note {}", note_id);
                        break;
                    }
                }
                _ = tokio::time::sleep(tokio::time::Duration::from_millis(50)) => {
                    // ring buffer에서 오디오 소비
                    let raw_samples = consumer.drain();
                    if raw_samples.is_empty() {
                        continue;
                    }

                    // 리샘플링 (→ 16kHz mono)
                    let resampled = match resampler.process(&raw_samples, channels) {
                        Ok(s) => s,
                        Err(e) => {
                            warn!("Resample error: {}", e);
                            continue;
                        }
                    };

                    if resampled.is_empty() {
                        continue;
                    }

                    // VAD 체크 (정보 표시용) + 모든 오디오를 accumulator에 전달
                    // 회의 녹음에서는 VAD로 음성을 제거하면 문맥이 끊어지므로,
                    // 모든 오디오를 STT에 전달하고 whisper가 자체적으로 무음을 처리
                    let mut has_speech = false;
                    for frame in resampled.chunks(vad_frame_size) {
                        if frame.len() >= vad_frame_size && vad.is_speech(frame).unwrap_or(false) {
                            has_speech = true;
                            break;
                        }
                    }

                    if has_speech {
                        silence_counter = 0;
                    } else {
                        silence_counter += 1;
                    }

                    // 모든 리샘플링된 오디오를 accumulator에 전달
                    let chunks = accumulator.push(&resampled);

                    for chunk in chunks {
                        info!(
                            "STT chunk ready: {:.1}s at {}ms — sending to whisper",
                            chunk.duration_secs(),
                            chunk.timestamp_ms
                        );

                        // STT 전사 실행
                        self.transcribe_chunk(chunk, &note_id).await;
                    }
                }
            }
        }
    }
}
