use std::sync::Arc;
use tracing::{info, warn};

use crate::error::LlmError;
use crate::llm::{GenerateConfig, LlmProvider};

/// 클라우드 → 로컬 자동 폴백 Provider
///
/// 클라우드 API 호출이 3회 실패하면 자동으로 로컬 모델로 전환합니다.
pub struct FallbackLlmProvider {
    primary: Arc<dyn LlmProvider>,
    fallback: Arc<dyn LlmProvider>,
    max_retries: u32,
}

impl FallbackLlmProvider {
    pub fn new(primary: Arc<dyn LlmProvider>, fallback: Arc<dyn LlmProvider>) -> Self {
        Self {
            primary,
            fallback,
            max_retries: 3,
        }
    }

    pub fn with_max_retries(mut self, retries: u32) -> Self {
        self.max_retries = retries;
        self
    }

    /// 프라이머리 시도 → 실패 시 폴백
    pub async fn generate_with_fallback(
        &self,
        prompt: &str,
        config: &GenerateConfig,
    ) -> Result<(String, bool), LlmError> {
        let mut last_error = None;

        for attempt in 1..=self.max_retries {
            match self.primary.generate(prompt, config).await {
                Ok(result) => return Ok((result, false)), // false = no fallback
                Err(e) => {
                    warn!(
                        "Primary provider ({}) attempt {}/{} failed: {}",
                        self.primary.name(),
                        attempt,
                        self.max_retries,
                        e
                    );
                    last_error = Some(e);

                    if attempt < self.max_retries {
                        // Exponential backoff
                        let delay = std::time::Duration::from_millis(500 * 2u64.pow(attempt - 1));
                        tokio::time::sleep(delay).await;
                    }
                }
            }
        }

        // 폴백으로 전환
        info!(
            "Falling back from {} to {}",
            self.primary.name(),
            self.fallback.name()
        );

        match self.fallback.generate(prompt, config).await {
            Ok(result) => Ok((result, true)), // true = fallback used
            Err(e) => Err(LlmError::Provider(format!(
                "Both primary ({}: {}) and fallback ({}: {}) failed",
                self.primary.name(),
                last_error.map(|e| e.to_string()).unwrap_or_default(),
                self.fallback.name(),
                e
            ))),
        }
    }
}
