use async_trait::async_trait;
use tracing::{debug, info};

use super::{GenerateConfig, LlmProvider};
use crate::error::LlmError;

/// llama.cpp 기반 로컬 LLM Provider
///
/// GGUF 양자화 모델을 로드하여 로컬 추론을 수행합니다.
/// 세션 풀링, GBNF Grammar, GPU offloading을 지원합니다.
pub struct LocalLlmProvider {
    model_path: std::path::PathBuf,
    context_length: usize,
    gpu_layers: i32,
}

impl LocalLlmProvider {
    pub fn new(
        model_path: std::path::PathBuf,
        context_length: usize,
        gpu_layers: i32,
    ) -> Result<Self, LlmError> {
        if !model_path.exists() {
            return Err(LlmError::ModelNotLoaded);
        }
        info!("LocalLlmProvider created: {:?}, ctx={}, gpu_layers={}", model_path, context_length, gpu_layers);
        Ok(Self {
            model_path,
            context_length,
            gpu_layers,
        })
    }
}

#[async_trait]
impl LlmProvider for LocalLlmProvider {
    async fn generate(&self, prompt: &str, config: &GenerateConfig) -> Result<String, LlmError> {
        // llama.cpp FFI 호출 — Phase 2 실구현
        // 현재는 프롬프트 기반 스텁 응답
        debug!("LocalLLM generate: prompt_len={}, temp={}", prompt.len(), config.temperature);
        Err(LlmError::Inference(
            "llama.cpp FFI not yet linked. Use cloud-providers feature for now.".to_string(),
        ))
    }

    fn max_context_length(&self) -> usize {
        self.context_length
    }

    fn supports_grammar(&self) -> bool {
        true // llama.cpp GBNF 지원
    }

    fn name(&self) -> &str {
        "llama-local"
    }
}
