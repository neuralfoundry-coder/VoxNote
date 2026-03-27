//! 실제 Whisper 모델 기반 STT 추론 테스트
//!
//! whisper-tiny 모델을 로드하고 합성 오디오에 대한 전사를 수행합니다.
//! 이 테스트는 `stt` feature가 활성화되어야 하며,
//! ~/.voxnote/models/ggml-tiny.bin 파일이 필요합니다.

#[cfg(feature = "stt")]
mod whisper_tests {
    use std::path::PathBuf;
    use voxnote_core::audio::AudioChunk;
    use voxnote_core::stt::whisper::LocalSttProvider;
    use voxnote_core::stt::SttProvider;

    fn model_path() -> PathBuf {
        dirs::home_dir()
            .unwrap()
            .join(".voxnote/models/ggml-tiny.bin")
    }

    fn skip_if_no_model() -> bool {
        let path = model_path();
        if !path.exists() {
            eprintln!("SKIP: whisper model not found at {:?}", path);
            true
        } else {
            false
        }
    }

    /// 합성 음성 신호 (16kHz mono)
    fn generate_audio(duration_secs: f32) -> Vec<f32> {
        let sample_rate = 16000;
        let num_samples = (sample_rate as f32 * duration_secs) as usize;
        (0..num_samples)
            .map(|i| {
                let t = i as f32 / sample_rate as f32;
                // 440Hz + 880Hz 혼합 사인파 (단순 음성 시뮬레이션)
                ((2.0 * std::f32::consts::PI * 440.0 * t).sin()
                    + (2.0 * std::f32::consts::PI * 880.0 * t).sin() * 0.5)
                    * 0.3
            })
            .collect()
    }

    /// 무음 오디오 (16kHz mono)
    fn generate_silence(duration_secs: f32) -> Vec<f32> {
        vec![0.0f32; (16000.0 * duration_secs) as usize]
    }

    // ── 실제 모델 로드 테스트 ────────────────────────────────────

    #[test]
    fn real_whisper_model_load() {
        if skip_if_no_model() { return; }

        let start = std::time::Instant::now();
        let provider = LocalSttProvider::new(model_path());
        let load_time = start.elapsed();

        assert!(provider.is_ok(), "Model load should succeed");
        assert!(
            load_time < std::time::Duration::from_secs(5),
            "Model load should take <5s, took {:?}",
            load_time
        );

        let provider = provider.unwrap();
        assert_eq!(provider.name(), "whisper-local");

        let languages = provider.supported_languages();
        assert!(!languages.is_empty());
        eprintln!("Model loaded in {:?}, {} languages supported", load_time, languages.len());
    }

    // ── 합성 오디오 전사 (음성 아닌 신호에 대한 robustness) ─────

    #[tokio::test]
    async fn real_whisper_transcribe_synthetic() {
        if skip_if_no_model() { return; }

        let provider = LocalSttProvider::new(model_path()).unwrap();

        // 3초 합성 오디오
        let audio = generate_audio(3.0);
        let chunk = AudioChunk::new(audio, 0);

        let start = std::time::Instant::now();
        let result = provider.transcribe(&chunk, "test-note").await;
        let inference_time = start.elapsed();

        assert!(result.is_ok(), "Transcription should not error: {:?}", result.err());

        let segments = result.unwrap();
        eprintln!(
            "Transcribed {} segments in {:?}",
            segments.len(),
            inference_time
        );

        // 합성 신호이므로 결과가 비어있거나 짧을 수 있음 — 에러 없이 완료가 핵심
        assert!(
            inference_time < std::time::Duration::from_secs(10),
            "Inference should complete in <10s, took {:?}",
            inference_time
        );
    }

    // ── 무음 전사 (빈 결과 또는 무의미 텍스트) ──────────────────

    #[tokio::test]
    async fn real_whisper_transcribe_silence() {
        if skip_if_no_model() { return; }

        let provider = LocalSttProvider::new(model_path()).unwrap();

        let silence = generate_silence(2.0);
        let chunk = AudioChunk::new(silence, 0);

        let result = provider.transcribe(&chunk, "silence-test").await;
        assert!(result.is_ok());

        let segments = result.unwrap();
        eprintln!("Silence transcription: {} segments", segments.len());
        // 무음이므로 의미 있는 텍스트가 거의 없어야 함
    }

    // ── 연속 전사 (initial_prompt 문맥 연속) ────────────────────

    #[tokio::test]
    async fn real_whisper_sequential_chunks() {
        if skip_if_no_model() { return; }

        let mut provider = LocalSttProvider::new(model_path()).unwrap();

        // 5개 청크 연속 전사
        let mut all_text = String::new();
        for i in 0..5 {
            let audio = generate_audio(2.0);
            let chunk = AudioChunk::new(audio, i * 2000);

            let start = std::time::Instant::now();
            let segments = provider.transcribe(&chunk, "seq-test").await.unwrap();
            let time = start.elapsed();

            for seg in &segments {
                all_text.push_str(&seg.text);
                all_text.push(' ');
            }

            // initial_prompt 설정 (마지막 텍스트)
            if !all_text.is_empty() {
                let last_50: String = all_text.chars().rev().take(50).collect::<String>().chars().rev().collect();
                provider.set_initial_prompt(&last_50);
            }

            eprintln!("Chunk {}: {} segments in {:?}", i, segments.len(), time);
        }

        eprintln!("Total text length: {} chars", all_text.len());
    }

    // ── 모델 크기 검증 ──────────────────────────────────────────

    #[test]
    fn real_whisper_model_file_integrity() {
        if skip_if_no_model() { return; }

        let path = model_path();
        let metadata = std::fs::metadata(&path).unwrap();
        let size_mb = metadata.len() / 1_048_576;

        assert!(
            size_mb >= 70 && size_mb <= 80,
            "Whisper tiny should be ~75MB, got {}MB",
            size_mb
        );
    }

    // ── 언어 목록 확인 ──────────────────────────────────────────

    #[test]
    fn real_whisper_supported_languages() {
        if skip_if_no_model() { return; }

        let provider = LocalSttProvider::new(model_path()).unwrap();
        let langs = provider.supported_languages();

        let codes: Vec<&str> = langs.iter().map(|l| l.code.as_str()).collect();
        assert!(codes.contains(&"auto"), "Should support auto-detect");
        assert!(codes.contains(&"ko"), "Should support Korean");
        assert!(codes.contains(&"en"), "Should support English");
        assert!(codes.contains(&"ja"), "Should support Japanese");
    }

    // ── 추론 성능 벤치마크 (5회 반복 평균) ──────────────────────

    #[tokio::test]
    async fn real_whisper_inference_benchmark() {
        if skip_if_no_model() { return; }

        let provider = LocalSttProvider::new(model_path()).unwrap();
        let audio = generate_audio(3.0); // 3초 오디오

        let mut times = Vec::new();
        for _ in 0..5 {
            let chunk = AudioChunk::new(audio.clone(), 0);
            let start = std::time::Instant::now();
            let _ = provider.transcribe(&chunk, "bench").await;
            times.push(start.elapsed());
        }

        let avg = times.iter().map(|t| t.as_millis()).sum::<u128>() / times.len() as u128;
        let min = times.iter().map(|t| t.as_millis()).min().unwrap();
        let max = times.iter().map(|t| t.as_millis()).max().unwrap();

        eprintln!(
            "Whisper-tiny 3s inference: avg={}ms, min={}ms, max={}ms",
            avg, min, max
        );

        assert!(
            avg < 5000,
            "Average inference should be <5s for tiny model, got {}ms",
            avg
        );
    }
}

// stt feature 비활성화 시에도 컴파일 통과
#[cfg(not(feature = "stt"))]
#[test]
fn whisper_tests_require_stt_feature() {
    eprintln!("Whisper tests skipped: stt feature not enabled");
}
