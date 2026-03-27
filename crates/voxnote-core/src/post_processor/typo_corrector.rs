use std::sync::Arc;

use crate::error::LlmError;
use crate::llm::{GenerateConfig, LlmProvider};

/// LLM 경량 패스 오탈자 교정
pub struct TypoCorrector {
    llm: Arc<dyn LlmProvider>,
}

impl TypoCorrector {
    pub fn new(llm: Arc<dyn LlmProvider>) -> Self {
        Self { llm }
    }

    /// 전사 텍스트의 오탈자를 교정
    pub async fn correct(&self, text: &str) -> Result<String, LlmError> {
        if text.trim().is_empty() {
            return Ok(text.to_string());
        }

        let prompt = format!(
            "Fix any typos or transcription errors in the following text. \
             Only fix clear errors, do not change the meaning or style. \
             Return only the corrected text, nothing else.\n\n{}",
            text
        );

        let config = GenerateConfig {
            temperature: 0.1,
            top_p: 0.9,
            max_tokens: text.len() * 2,
            grammar: None,
        };

        self.llm.generate(&prompt, &config).await
    }
}
