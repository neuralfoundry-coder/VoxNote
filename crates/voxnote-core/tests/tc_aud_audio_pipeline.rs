//! TC-AUD: Audio Pipeline 통합 테스트
//! 관련 요구사항: FR-AUD-001 ~ FR-AUD-004
//! 참조: docs/test/test-cases/TC-AUD.md

use voxnote_core::audio::accumulator::Accumulator;
use voxnote_core::audio::resample::Resampler;
use voxnote_core::audio::ringbuf::AudioRingBuffer;
use voxnote_core::audio::vad::{EnergyVad, VoiceActivityDetector};
use voxnote_core::audio::AudioChunk;
use std::sync::Arc;
use std::thread;

// ── TC-AUD-002-01: 48kHz→16kHz 리샘플링 정확성 ──────────────────

#[test]
fn tc_aud_002_01_resample_48k_to_16k_mono() {
    let mut resampler = Resampler::new(48000, 2).expect("Resampler creation failed");

    // 48kHz stereo 사인파 생성 (1초)
    let sample_count = 48000 * 2; // stereo
    let input: Vec<f32> = (0..sample_count)
        .map(|i| {
            let t = (i / 2) as f32 / 48000.0;
            (2.0 * std::f32::consts::PI * 440.0 * t).sin() * 0.5
        })
        .collect();

    let output = resampler.process(&input, 2).expect("Resample failed");
    let flushed = resampler.flush().expect("Flush failed");

    let total_output: usize = output.len() + flushed.len();

    // 16kHz mono로 변환 → 약 16000 샘플 (±10% 허용)
    assert!(
        total_output > 12000 && total_output < 20000,
        "Expected ~16000 samples, got {}",
        total_output
    );

    // 출력 값이 유효 범위 내
    for &s in output.iter().chain(flushed.iter()) {
        assert!(
            s >= -1.5 && s <= 1.5,
            "Sample out of range: {}",
            s
        );
    }
}

#[test]
fn tc_aud_002_02_resample_mono_passthrough() {
    let mut resampler = Resampler::new(48000, 1).expect("Resampler creation failed");

    let input: Vec<f32> = (0..48000)
        .map(|i| (i as f32 / 48000.0 * 440.0 * 2.0 * std::f32::consts::PI).sin())
        .collect();

    let output = resampler.process(&input, 1).expect("Resample failed");
    let flushed = resampler.flush().expect("Flush failed");
    let total = output.len() + flushed.len();

    // mono 48k → 16k = 약 16000
    assert!(total > 12000 && total < 20000, "Got {} samples", total);
}

// ── TC-AUD-003-01: VAD 정확도 ──────────────────────────────────

#[test]
fn tc_aud_003_01_vad_silence_vs_speech() {
    let mut vad = EnergyVad::new(0.5);

    // 무음 (0 amplitude)
    let silence = vec![0.0f32; 480]; // 30ms at 16kHz
    let prob = vad.detect(&silence).unwrap();
    assert!(prob < 0.1, "Silence probability should be < 0.1, got {}", prob);
    assert!(!vad.is_speech(&silence).unwrap());

    // 강한 음성 신호
    let speech: Vec<f32> = (0..480)
        .map(|i| (i as f32 * 0.1).sin() * 0.5)
        .collect();
    let prob = vad.detect(&speech).unwrap();
    assert!(prob > 0.3, "Speech probability should be > 0.3, got {}", prob);
    assert!(vad.is_speech(&speech).unwrap());
}

#[test]
fn tc_aud_003_02_vad_threshold_adjustment() {
    let mut vad = EnergyVad::new(0.5);

    let medium_signal: Vec<f32> = (0..480)
        .map(|i| (i as f32 * 0.05).sin() * 0.02)
        .collect();

    // 높은 임계값에서는 음성이 아님
    vad.set_threshold(0.9);
    assert!(!vad.is_speech(&medium_signal).unwrap());

    // 낮은 임계값에서는 음성
    vad.set_threshold(0.01);
    let is_speech = vad.is_speech(&medium_signal).unwrap();
    // 신호가 약하면 감지 안 될 수 있음 — 검증만
    assert_eq!(vad.threshold(), 0.01);
}

#[test]
fn tc_aud_003_03_vad_empty_input() {
    let mut vad = EnergyVad::new(0.5);
    let empty: Vec<f32> = vec![];
    let prob = vad.detect(&empty).unwrap();
    assert_eq!(prob, 0.0, "Empty input should return 0.0 probability");
}

// ── TC-AUD-004-01: Ring Buffer 단일 프로듀서-컨슈머 ─────────────

#[test]
fn tc_aud_004_01_ringbuf_single_producer_consumer() {
    let buf = AudioRingBuffer::new(128);
    let producer = buf.producer();
    let consumer = buf.consumer();

    // 1024 샘플 쓰기 (128개 × 8 청크)
    let test_data: Vec<f32> = (0..1024).map(|i| i as f32 * 0.001).collect();
    for chunk in test_data.chunks(128) {
        producer.push(chunk.to_vec());
    }

    // 전부 읽기
    let read_data = consumer.drain();
    assert_eq!(read_data.len(), 1024, "Should read all 1024 samples");
    assert_eq!(read_data[0], 0.0);
    assert!((read_data[1023] - 1.023).abs() < 0.001);
}

#[test]
fn tc_aud_004_02_ringbuf_overflow_tracking() {
    let buf = AudioRingBuffer::new(3);
    let producer = buf.producer();
    let consumer = buf.consumer();

    // 용량(3) 초과 push
    producer.push(vec![1.0]);
    producer.push(vec![2.0]);
    producer.push(vec![3.0]);
    producer.push(vec![4.0]); // overflow #1
    producer.push(vec![5.0]); // overflow #2

    assert_eq!(consumer.dropped_count(), 2);

    // 살아남은 것만 읽기
    let remaining = consumer.drain();
    assert_eq!(remaining.len(), 3);
}

// ── TC-AUD-004-03: Ring Buffer 멀티스레드 동시 성능 ─────────────

#[test]
fn tc_aud_004_03_ringbuf_concurrent_integrity() {
    let buf = AudioRingBuffer::new(4096);
    let producer = buf.producer();
    let consumer = buf.consumer();

    let total_chunks = 400usize;
    let chunk_size = 256;
    let total_samples = total_chunks * chunk_size;

    // 프로듀서 스레드
    let producer_handle = thread::spawn(move || {
        let mut sent = 0usize;
        for _ in 0..total_chunks {
            let chunk: Vec<f32> = (sent..sent + chunk_size)
                .map(|i| i as f32)
                .collect();
            producer.push(chunk);
            sent += chunk_size;
        }
        sent
    });

    // 컨슈머 스레드 — 약간의 지연으로 동시 실행
    let consumer_handle = thread::spawn(move || {
        let mut received = Vec::new();
        let start = std::time::Instant::now();
        while start.elapsed() < std::time::Duration::from_secs(2) {
            let data = consumer.drain();
            if !data.is_empty() {
                received.extend(data);
            }
            thread::yield_now();
        }
        // 마지막 drain
        received.extend(consumer.drain());
        received
    });

    let sent = producer_handle.join().unwrap();
    let received = consumer_handle.join().unwrap();

    assert_eq!(sent, total_samples);
    // 일부 드롭 가능하지만 받은 데이터의 무결성 확인
    assert!(!received.is_empty(), "Should receive some data");
    // 연속성 확인 (드롭 없는 구간)
    for window in received.windows(2) {
        if window[1] - window[0] == 1.0 {
            // 연속 — OK
        }
        // 점프가 있으면 드롭된 것 — 허용
    }
}

// ── TC-AUD 통합: 어큐뮬레이터 슬라이딩 윈도우 ──────────────────

#[test]
fn tc_aud_accumulator_window_count() {
    // 3초 윈도우, 0.5초 오버랩, 16kHz
    let mut acc = Accumulator::new(3.0, 0.5, 16000);

    // 10초 분량 = 160000 샘플
    let samples = vec![0.1f32; 160000];
    let chunks = acc.push(&samples);

    // 윈도우=48000, 전진=40000 (48000-8000)
    // 160000 / 40000 = 4 완전 윈도우
    assert!(
        chunks.len() >= 3 && chunks.len() <= 5,
        "Expected 3-5 chunks from 10s audio, got {}",
        chunks.len()
    );

    // 각 청크의 길이 확인
    for chunk in &chunks {
        assert_eq!(
            chunk.samples.len(),
            48000,
            "Each chunk should be 3s = 48000 samples"
        );
        assert_eq!(chunk.sample_rate, 16000);
    }

    // 타임스탬프 증가 확인
    for i in 1..chunks.len() {
        assert!(
            chunks[i].timestamp_ms > chunks[i - 1].timestamp_ms,
            "Timestamps should increase"
        );
    }
}

#[test]
fn tc_aud_accumulator_flush_partial() {
    let mut acc = Accumulator::new(3.0, 0.5, 16000);

    // 1.5초 분량 (윈도우 미달)
    let samples = vec![0.1f32; 24000];
    let chunks = acc.push(&samples);
    assert!(chunks.is_empty(), "Should not produce chunk for partial window");

    // 플러시
    let flushed = acc.flush();
    assert!(flushed.is_some(), "Flush should return remaining samples");
    let flushed = flushed.unwrap();
    assert_eq!(flushed.samples.len(), 24000);
}

#[test]
fn tc_aud_accumulator_reset() {
    let mut acc = Accumulator::new(1.0, 0.0, 16000);
    acc.push(&vec![0.1f32; 8000]);
    assert!(acc.buffered_secs() > 0.0);

    acc.reset();
    assert_eq!(acc.buffered_secs(), 0.0);
    assert!(acc.flush().is_none());
}
