//! E2E: 오디오 파이프라인 실행 기반 통합 테스트
//!
//! 합성 오디오를 생성하여 실제 파이프라인 경로를 검증합니다.
//! 마이크 없이 전체 flow를 테스트합니다:
//!   합성 오디오 → RingBuffer → Resampler → VAD → Accumulator → Pipeline Events

use std::time::Duration;
use tokio::sync::{mpsc, watch};

use voxnote_core::audio::accumulator::Accumulator;
use voxnote_core::audio::resample::Resampler;
use voxnote_core::audio::ringbuf::AudioRingBuffer;
use voxnote_core::audio::vad::{EnergyVad, VoiceActivityDetector};
use voxnote_core::config::AppConfig;
use voxnote_core::pipeline::{PipelineEvent, TranscriptionPipeline};

/// 합성 음성 신호 생성 (440Hz 사인파)
fn generate_speech_signal(sample_rate: u32, duration_secs: f32, amplitude: f32) -> Vec<f32> {
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    (0..num_samples)
        .map(|i| {
            let t = i as f32 / sample_rate as f32;
            (2.0 * std::f32::consts::PI * 440.0 * t).sin() * amplitude
        })
        .collect()
}

/// 합성 무음 생성
fn generate_silence(sample_rate: u32, duration_secs: f32) -> Vec<f32> {
    let num_samples = (sample_rate as f32 * duration_secs) as usize;
    vec![0.0f32; num_samples]
}

/// 합성 스테레오 신호 → 채널 인터리빙
fn make_stereo(mono: &[f32]) -> Vec<f32> {
    mono.iter().flat_map(|&s| [s, s]).collect()
}

// ── E2E-001: 전체 리샘플링 파이프라인 ──────────────────────────

#[test]
fn e2e_001_resample_full_pipeline() {
    // 48kHz stereo 5초 → 16kHz mono
    let speech = generate_speech_signal(48000, 5.0, 0.5);
    let stereo = make_stereo(&speech);

    let mut resampler = Resampler::new(48000, 2).unwrap();
    let mut output = Vec::new();

    // 실제 앱처럼 4096 샘플(스테레오=2048프레임) 단위로 청크 처리
    for chunk in stereo.chunks(4096) {
        let resampled = resampler.process(chunk, 2).unwrap();
        output.extend(resampled);
    }
    let flushed = resampler.flush().unwrap();
    output.extend(flushed);

    // 16kHz mono 5초 → ~80000 샘플 (±15% 허용)
    let expected = 80000;
    assert!(
        output.len() > expected * 85 / 100 && output.len() < expected * 115 / 100,
        "Expected ~{} samples, got {}",
        expected,
        output.len()
    );

    // 출력값 범위 확인
    let max_val = output.iter().cloned().fold(0.0f32, f32::max);
    let min_val = output.iter().cloned().fold(0.0f32, f32::min);
    assert!(max_val <= 1.0 && min_val >= -1.0, "Output out of [-1, 1] range");
}

// ── E2E-002: VAD + 어큐뮬레이터 통합 ──────────────────────────

#[test]
fn e2e_002_vad_accumulator_speech_silence_pattern() {
    let mut vad = EnergyVad::new(0.3);
    let mut acc = Accumulator::new(1.0, 0.0, 16000); // 1초 윈도우, 오버랩 없음

    // 패턴: 2초 음성 → 1초 무음 → 2초 음성
    let speech1 = generate_speech_signal(16000, 2.0, 0.5);
    let silence = generate_silence(16000, 1.0);
    let speech2 = generate_speech_signal(16000, 2.0, 0.5);

    let frame_size = 480; // 30ms at 16kHz

    let all_samples: Vec<f32> = speech1.iter()
        .chain(silence.iter())
        .chain(speech2.iter())
        .cloned()
        .collect();

    let mut voiced_samples = Vec::new();
    for frame in all_samples.chunks(frame_size) {
        if frame.len() >= frame_size && vad.is_speech(frame).unwrap() {
            voiced_samples.extend_from_slice(frame);
        }
    }

    // VAD가 음성 구간을 필터링
    // 무음 1초(16000 샘플)가 제거되어야 함
    assert!(
        voiced_samples.len() < all_samples.len(),
        "VAD should filter out silence: {} < {}",
        voiced_samples.len(),
        all_samples.len()
    );

    // 어큐뮬레이터에 통과된 음성 샘플 전달
    let chunks = acc.push(&voiced_samples);

    // 약 4초의 음성 → 1초 윈도우 → ~4개 청크
    assert!(
        chunks.len() >= 2,
        "Should produce at least 2 chunks from ~4s speech, got {}",
        chunks.len()
    );

    // 각 청크가 1초(16000 샘플) 확인
    for chunk in &chunks {
        assert_eq!(chunk.samples.len(), 16000);
    }
}

// ── E2E-003: RingBuffer → Resampler → VAD → Accumulator 풀체인

#[test]
fn e2e_003_full_audio_chain() {
    let buf = AudioRingBuffer::new(100);
    let producer = buf.producer();
    let consumer = buf.consumer();

    // 프로듀서: 48kHz mono 3초 음성 (1024 샘플 청크로 push)
    let speech = generate_speech_signal(48000, 3.0, 0.5);
    for chunk in speech.chunks(1024) {
        producer.push(chunk.to_vec());
    }

    // 컨슈머: drain → resample → VAD → accumulate
    let raw = consumer.drain();
    assert!(!raw.is_empty());

    let mut resampler = Resampler::new(48000, 1).unwrap();
    let resampled = resampler.process(&raw, 1).unwrap();
    let flushed = resampler.flush().unwrap();
    let all_resampled: Vec<f32> = resampled.into_iter().chain(flushed).collect();

    // 16kHz 3초 → ~48000 샘플
    assert!(all_resampled.len() > 30000, "Should have >30000 resampled samples");

    let mut vad = EnergyVad::new(0.3);
    let frame_size = 480;
    let mut voiced = Vec::new();
    for frame in all_resampled.chunks(frame_size) {
        if frame.len() >= frame_size && vad.is_speech(frame).unwrap() {
            voiced.extend_from_slice(frame);
        }
    }

    let mut acc = Accumulator::new(2.0, 0.5, 16000);
    let chunks = acc.push(&voiced);

    // 결과 청크가 존재해야 함
    assert!(
        !chunks.is_empty() || acc.buffered_secs() > 0.0,
        "Should produce chunks or have buffered data"
    );
}

// ── E2E-004: TranscriptionPipeline 이벤트 수신 ─────────────────

#[tokio::test]
async fn e2e_004_pipeline_emits_events() {
    let config = AppConfig::default();
    let (event_tx, mut event_rx) = mpsc::unbounded_channel::<PipelineEvent>();
    let pipeline = TranscriptionPipeline::new(config, event_tx);

    let buf = AudioRingBuffer::new(200);
    let producer = buf.producer();
    let consumer = buf.consumer();

    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    // 파이프라인 시작
    let pipeline_handle = tokio::spawn(async move {
        pipeline
            .run_processing_loop(consumer, "test-note-001".to_string(), shutdown_rx)
            .await;
    });

    // 합성 음성 주입 (48kHz mono 기본, 5초분량 → 3초 윈도우에서 청크 생성)
    let speech = generate_speech_signal(48000, 5.0, 0.5);
    for chunk in speech.chunks(4096) {
        producer.push(chunk.to_vec());
    }

    // 이벤트 수집 (최대 3초 대기)
    let mut events = Vec::new();
    let deadline = tokio::time::Instant::now() + Duration::from_secs(3);
    loop {
        tokio::select! {
            Some(event) = event_rx.recv() => {
                events.push(event);
            }
            _ = tokio::time::sleep_until(deadline) => {
                break;
            }
        }
    }

    // 종료
    let _ = shutdown_tx.send(true);
    let _ = tokio::time::timeout(Duration::from_secs(2), pipeline_handle).await;

    // 이벤트 검증 — 5초 음성, 3초 윈도우 → 최소 1개 Segment 이벤트
    let segment_events: Vec<_> = events
        .iter()
        .filter(|e| matches!(e, PipelineEvent::Segment(_)))
        .collect();

    assert!(
        !segment_events.is_empty(),
        "Pipeline should emit at least 1 segment event, got {} total events",
        events.len()
    );

    // 세그먼트의 note_id 확인
    if let PipelineEvent::Segment(seg) = &segment_events[0] {
        assert_eq!(seg.note_id, "test-note-001");
        assert!(seg.start_ms >= 0);
    }
}

// ── E2E-005: Pipeline shutdown 정상 종료 ────────────────────────

#[tokio::test]
async fn e2e_005_pipeline_graceful_shutdown() {
    let config = AppConfig::default();
    let (event_tx, _event_rx) = mpsc::unbounded_channel::<PipelineEvent>();
    let pipeline = TranscriptionPipeline::new(config, event_tx);

    let buf = AudioRingBuffer::new(10);
    let consumer = buf.consumer();
    let (shutdown_tx, shutdown_rx) = watch::channel(false);

    let handle = tokio::spawn(async move {
        pipeline
            .run_processing_loop(consumer, "shutdown-test".to_string(), shutdown_rx)
            .await;
        true // 정상 종료 확인
    });

    // 즉시 종료 신호
    tokio::time::sleep(Duration::from_millis(200)).await;
    shutdown_tx.send(true).unwrap();

    let result = tokio::time::timeout(Duration::from_secs(3), handle).await;
    assert!(result.is_ok(), "Pipeline should shut down within 3s");
    assert!(result.unwrap().unwrap(), "Pipeline should return true on graceful exit");
}
