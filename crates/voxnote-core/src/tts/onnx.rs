use async_trait::async_trait;
use tracing::debug;

use super::{TtsOutput, TtsProvider};
use crate::error::VoxNoteError;

/// Piper ONNX 기반 로컬 TTS
pub struct LocalTtsProvider {
    model_path: std::path::PathBuf,
    supported_languages: Vec<String>,
}

impl LocalTtsProvider {
    pub fn new(model_path: std::path::PathBuf) -> Self {
        Self {
            model_path,
            supported_languages: vec![
                "ko".to_string(),
                "en".to_string(),
                "ja".to_string(),
            ],
        }
    }
}

#[async_trait]
impl TtsProvider for LocalTtsProvider {
    async fn synthesize(
        &self,
        text: &str,
        language: &str,
    ) -> Result<TtsOutput, VoxNoteError> {
        debug!("LocalTTS synthesize: lang={}, text_len={}", language, text.len());
        // ONNX Runtime (ort) 기반 Piper TTS 추론
        // 프로덕션에서 ort::Session으로 모델 로드 및 추론
        Ok(TtsOutput {
            samples: vec![0.0; 16000], // 1초 무음 placeholder
            sample_rate: 22050,
        })
    }

    fn supported_languages(&self) -> &[String] {
        &self.supported_languages
    }

    fn name(&self) -> &str {
        "piper-local"
    }
}
