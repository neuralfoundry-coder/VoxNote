use tokio::sync::mpsc;
use tracing::{debug, info, warn};

use crate::audio::accumulator::Accumulator;
use crate::audio::resample::Resampler;
use crate::audio::vad::{EnergyVad, VoiceActivityDetector};
use crate::config::AppConfig;
use crate::models::Segment;

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
}

impl TranscriptionPipeline {
    pub fn new(config: AppConfig, event_tx: mpsc::UnboundedSender<PipelineEvent>) -> Self {
        Self { config, event_tx }
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

        let mut vad = EnergyVad::new(self.config.audio.vad_threshold);
        let mut accumulator = Accumulator::new(
            self.config.audio.window_size_secs,
            self.config.audio.overlap_secs,
            16000,
        );

        let vad_frame_size = 480; // 30ms at 16kHz
        info!("Processing loop started for note {}", note_id);

        loop {
            tokio::select! {
                _ = shutdown_rx.changed() => {
                    if *shutdown_rx.borrow() {
                        // 잔여 버퍼 플러시
                        if let Some(chunk) = accumulator.flush() {
                            let _ = self.event_tx.send(PipelineEvent::Segment(
                                Segment::new(&note_id, "[flush]", chunk.timestamp_ms, chunk.timestamp_ms),
                            ));
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

                    // VAD 필터링 (프레임 단위)
                    let mut voiced_samples = Vec::new();
                    for frame in resampled.chunks(vad_frame_size) {
                        if frame.len() < vad_frame_size {
                            voiced_samples.extend_from_slice(frame);
                        } else if vad.is_speech(frame).unwrap_or(false) {
                            voiced_samples.extend_from_slice(frame);
                        }
                    }

                    if voiced_samples.is_empty() {
                        continue;
                    }

                    // 어큐뮬레이터 → AudioChunk 생성
                    let chunks = accumulator.push(&voiced_samples);

                    for chunk in chunks {
                        debug!(
                            "Audio chunk ready: {:.1}s at {}ms",
                            chunk.duration_secs(),
                            chunk.timestamp_ms
                        );

                        // STT 전사는 여기서 SttProvider를 호출
                        // Phase 1에서는 이벤트로 청크 정보를 전달
                        let _ = self.event_tx.send(PipelineEvent::Segment(
                            Segment::new(
                                &note_id,
                                &format!("[audio chunk: {:.1}s]", chunk.duration_secs()),
                                chunk.timestamp_ms,
                                chunk.timestamp_ms + (chunk.duration_secs() * 1000.0) as i64,
                            ),
                        ));
                    }
                }
            }
        }
    }
}
