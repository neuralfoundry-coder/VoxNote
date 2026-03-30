use async_trait::async_trait;
use std::path::PathBuf;
use std::sync::Mutex;
use tracing::{debug, info};
use whisper_rs::{FullParams, SamplingStrategy, WhisperContext, WhisperContextParameters};

use super::{Language, SttProvider};
use crate::audio::AudioChunk;
use crate::error::SttError;
use crate::models::Segment;

/// whisper.cpp 기반 로컬 STT Provider
pub struct LocalSttProvider {
    ctx: Mutex<WhisperContext>,
    languages: Vec<Language>,
    initial_prompt: Mutex<String>,
    language: Mutex<Option<String>>,
}

impl LocalSttProvider {
    /// 모델 파일 경로로 생성
    pub fn new(model_path: PathBuf) -> Result<Self, SttError> {
        let params = WhisperContextParameters::default();
        let ctx = WhisperContext::new_with_params(
            model_path.to_str().ok_or_else(|| {
                SttError::Inference("Invalid model path".to_string())
            })?,
            params,
        )
        .map_err(|e| SttError::Inference(format!("Failed to load whisper model: {}", e)))?;

        info!("Whisper model loaded: {:?}", model_path);

        let languages = vec![
            Language::auto(),
            Language::new("ko", "Korean"),
            Language::new("en", "English"),
            Language::new("ja", "Japanese"),
            Language::new("zh", "Chinese"),
            Language::new("es", "Spanish"),
            Language::new("fr", "French"),
            Language::new("de", "German"),
        ];

        Ok(Self {
            ctx: Mutex::new(ctx),
            languages,
            initial_prompt: Mutex::new(String::new()),
            language: Mutex::new(None),
        })
    }
}

#[async_trait]
impl SttProvider for LocalSttProvider {
    async fn transcribe(
        &self,
        audio: &AudioChunk,
        note_id: &str,
    ) -> Result<Vec<Segment>, SttError> {
        let samples = audio.samples.clone();
        let timestamp_offset = audio.timestamp_ms;
        let note_id = note_id.to_string();
        let initial_prompt = self.initial_prompt.lock().unwrap().clone();
        let language = self.language.lock().unwrap().clone();

        let ctx = self.ctx.lock().map_err(|e| {
            SttError::Inference(format!("Failed to lock whisper context: {}", e))
        })?;

        let mut state = ctx.create_state().map_err(|e| {
            SttError::Inference(format!("Failed to create whisper state: {}", e))
        })?;

        let mut params = FullParams::new(SamplingStrategy::Greedy { best_of: 1 });

        // 언어 설정
        if let Some(ref lang) = language {
            if lang != "auto" {
                params.set_language(Some(lang));
            }
        }

        // 문맥 연속성
        if !initial_prompt.is_empty() {
            params.set_initial_prompt(&initial_prompt);
        }

        // 타임스탬프 활성화
        params.set_token_timestamps(true);
        params.set_print_special(false);
        params.set_print_realtime(false);
        params.set_print_progress(false);
        params.set_no_context(false);

        // 추론 실행
        state.full(params, &samples).map_err(|e| {
            SttError::Inference(format!("Whisper inference failed: {}", e))
        })?;

        // 결과 추출 (whisper-rs 0.16 API)
        let num_segments = state.full_n_segments();

        let mut segments = Vec::new();
        for i in 0..num_segments {
            let Some(seg) = state.get_segment(i) else {
                continue;
            };

            let text = seg.to_str().map_err(|e| {
                SttError::Inference(format!("Failed to get segment text: {}", e))
            })?;

            let start = seg.start_timestamp();
            let end = seg.end_timestamp();

            let trimmed = text.trim();
            if trimmed.is_empty() {
                continue;
            }

            // whisper 타임스탬프는 10ms 단위 → ms로 변환 + 오프셋 적용
            let start_ms = timestamp_offset + (start * 10);
            let end_ms = timestamp_offset + (end * 10);

            segments.push(Segment::new(&note_id, trimmed, start_ms, end_ms));
        }

        debug!(
            "Whisper transcribed {} segments from {:.1}s audio",
            segments.len(),
            audio.duration_secs()
        );

        Ok(segments)
    }

    fn supported_languages(&self) -> &[Language] {
        &self.languages
    }

    fn set_initial_prompt(&self, prompt: &str) {
        *self.initial_prompt.lock().unwrap() = prompt.to_string();
    }

    fn set_language(&self, language: Option<&str>) {
        *self.language.lock().unwrap() = language.map(String::from);
    }

    fn name(&self) -> &str {
        "whisper-local"
    }
}
