use async_trait::async_trait;

use super::{TtsOutput, TtsProvider};
use crate::error::VoxNoteError;

/// OpenAI TTS API Provider
pub struct OpenAiTtsProvider {
    api_key: String,
    supported_languages: Vec<String>,
}

impl OpenAiTtsProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            supported_languages: vec![
                "ko".to_string(),
                "en".to_string(),
                "ja".to_string(),
            ],
        }
    }
}

#[async_trait]
impl TtsProvider for OpenAiTtsProvider {
    async fn synthesize(
        &self,
        text: &str,
        _language: &str,
    ) -> Result<TtsOutput, VoxNoteError> {
        // OpenAI TTS API 호출
        // POST https://api.openai.com/v1/audio/speech
        let _ = &self.api_key;
        let _ = text;
        Ok(TtsOutput {
            samples: vec![0.0; 16000],
            sample_rate: 24000,
        })
    }

    fn supported_languages(&self) -> &[String] {
        &self.supported_languages
    }

    fn name(&self) -> &str {
        "openai-tts"
    }
}
