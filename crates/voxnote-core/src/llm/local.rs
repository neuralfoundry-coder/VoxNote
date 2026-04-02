use async_trait::async_trait;
use std::sync::Mutex;
use tracing::{debug, info};

use super::{GenerateConfig, LlmProvider};
use crate::error::LlmError;

use llama_cpp_2::context::params::LlamaContextParams;
use llama_cpp_2::llama_backend::LlamaBackend;
use llama_cpp_2::llama_batch::LlamaBatch;
use llama_cpp_2::model::params::LlamaModelParams;
use llama_cpp_2::model::{AddBos, LlamaModel, Special};
use llama_cpp_2::sampling::LlamaSampler;

/// llama.cpp 기반 로컬 LLM Provider
///
/// GGUF 양자화 모델을 로드하여 로컬 추론을 수행합니다.
/// 모델은 첫 호출 시 지연 로드되며, Mutex로 스레드 안전성을 보장합니다.
pub struct LocalLlmProvider {
    model_path: std::path::PathBuf,
    context_length: usize,
    gpu_layers: i32,
    // LlamaModel은 !Send이므로 Mutex<Option<>> + 전용 스레드에서 접근
    model: Mutex<Option<ModelState>>,
}

struct ModelState {
    backend: LlamaBackend,
    model: LlamaModel,
}

// SAFETY: ModelState는 Mutex 내부에서만 접근되며 동시 접근 불가
unsafe impl Send for ModelState {}
unsafe impl Sync for ModelState {}

impl LocalLlmProvider {
    pub fn new(
        model_path: std::path::PathBuf,
        context_length: usize,
        gpu_layers: i32,
    ) -> Result<Self, LlmError> {
        if !model_path.exists() {
            return Err(LlmError::ModelNotLoaded);
        }
        info!(
            "LocalLlmProvider created: {:?}, ctx={}, gpu_layers={}",
            model_path, context_length, gpu_layers
        );
        Ok(Self {
            model_path,
            context_length,
            gpu_layers,
            model: Mutex::new(None),
        })
    }

    /// 모델을 지연 로드 (첫 호출 시)
    fn ensure_loaded(&self) -> Result<(), LlmError> {
        let mut guard = self.model.lock().map_err(|e| {
            LlmError::Inference(format!("Model lock poisoned: {}", e))
        })?;

        if guard.is_some() {
            return Ok(());
        }

        info!("Loading LLM model: {:?}", self.model_path);

        let backend = LlamaBackend::init().map_err(|e| {
            LlmError::Inference(format!("Failed to init llama backend: {}", e))
        })?;

        let model_params =
            LlamaModelParams::default().with_n_gpu_layers(self.gpu_layers as u32);

        let model =
            LlamaModel::load_from_file(&backend, &self.model_path, &model_params).map_err(
                |e| LlmError::Inference(format!("Failed to load model: {}", e)),
            )?;

        info!("LLM model loaded successfully");
        *guard = Some(ModelState { backend, model });
        Ok(())
    }

    /// Mutex 내부에서 추론 실행
    fn generate_sync(&self, prompt: &str, config: &GenerateConfig) -> Result<String, LlmError> {
        self.ensure_loaded()?;

        let guard = self.model.lock().map_err(|e| {
            LlmError::Inference(format!("Model lock poisoned: {}", e))
        })?;

        let state = guard
            .as_ref()
            .ok_or_else(|| LlmError::Inference("Model not loaded".to_string()))?;

        // 컨텍스트 생성
        let ctx_params = LlamaContextParams::default()
            .with_n_ctx(std::num::NonZeroU32::new(self.context_length as u32));

        let mut ctx = state
            .model
            .new_context(&state.backend, ctx_params)
            .map_err(|e| LlmError::Inference(format!("Context creation failed: {}", e)))?;

        // 토큰화
        let tokens = state
            .model
            .str_to_token(prompt, AddBos::Always)
            .map_err(|e| LlmError::Inference(format!("Tokenization failed: {}", e)))?;

        debug!(
            "LocalLLM: prompt_tokens={}, max_tokens={}, temp={}",
            tokens.len(),
            config.max_tokens,
            config.temperature
        );

        // 컨텍스트 오버플로 체크
        if tokens.len() > self.context_length {
            return Err(LlmError::ContextOverflow {
                used: tokens.len(),
                max: self.context_length,
            });
        }

        // 프롬프트 평가
        let mut batch = LlamaBatch::new(self.context_length, 1);

        for (i, &token) in tokens.iter().enumerate() {
            let is_last = i == tokens.len() - 1;
            batch
                .add(token, i as i32, &[0], is_last)
                .map_err(|e| LlmError::Inference(format!("Batch add failed: {}", e)))?;
        }

        ctx.decode(&mut batch)
            .map_err(|e| LlmError::Inference(format!("Decode failed: {}", e)))?;

        // 샘플러 생성
        let mut sampler = if config.temperature < 0.01 {
            LlamaSampler::chain_simple([LlamaSampler::greedy()])
        } else {
            LlamaSampler::chain_simple([
                LlamaSampler::temp(config.temperature),
                LlamaSampler::top_p(config.top_p, 1),
                LlamaSampler::dist(42),
            ])
        };

        // 생성 루프
        let mut output_tokens = Vec::new();
        let mut n_cur = tokens.len();

        for _ in 0..config.max_tokens {
            let token = sampler.sample(&ctx, batch.n_tokens() - 1);
            sampler.accept(token);

            // EOS 체크
            if state.model.is_eog_token(token) {
                break;
            }

            output_tokens.push(token);

            // 다음 토큰 디코딩 준비
            batch.clear();
            batch
                .add(token, n_cur as i32, &[0], true)
                .map_err(|e| LlmError::Inference(format!("Batch add failed: {}", e)))?;

            ctx.decode(&mut batch)
                .map_err(|e| LlmError::Inference(format!("Decode failed: {}", e)))?;

            n_cur += 1;
        }

        // 토큰 → 텍스트 변환
        let mut result = String::new();
        #[allow(deprecated)]
        for token in &output_tokens {
            let piece = state
                .model
                .token_to_str(*token, Special::Tokenize)
                .map_err(|e| LlmError::Inference(format!("Detokenization failed: {}", e)))?;
            result.push_str(&piece);
        }

        // <think>...</think> 추론 트레이스 제거 (Qwen3.5 등 reasoning 모델)
        let result = strip_think_tags(&result);

        debug!(
            "LocalLLM: generated {} tokens, text_len={}",
            output_tokens.len(),
            result.len()
        );

        Ok(result)
    }
}

#[async_trait]
impl LlmProvider for LocalLlmProvider {
    async fn generate(&self, prompt: &str, config: &GenerateConfig) -> Result<String, LlmError> {
        // llama.cpp는 동기이므로 블로킹 스레드에서 실행할 필요 있으나,
        // LlamaModel이 !Send이므로 현재 스레드에서 직접 실행
        // (호출자가 spawn_blocking으로 호출해야 함)
        self.generate_sync(prompt, config)
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

/// `<think>...</think>` 추론 트레이스를 제거합니다.
///
/// Qwen3.5 등 reasoning-distilled 모델이 생성하는 CoT 블록을
/// 최종 응답에서 제거합니다. 중첩 태그는 가장 바깥 쌍만 제거합니다.
pub fn strip_think_tags(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut remaining = text;

    while let Some(start) = remaining.find("<think>") {
        // <think> 앞의 텍스트 보존
        result.push_str(&remaining[..start]);

        // 대응하는 </think> 찾기
        if let Some(end) = remaining[start..].find("</think>") {
            remaining = &remaining[start + end + "</think>".len()..];
        } else {
            // 닫힘 태그 없음 → 이후 전체가 think 블록으로 간주
            remaining = "";
            break;
        }
    }

    result.push_str(remaining);
    result.trim().to_string()
}

/// ChatML 형식으로 프롬프트를 래핑합니다.
///
/// `<|im_start|>system\n{system}<|im_end|>\n<|im_start|>user\n{user}<|im_end|>\n<|im_start|>assistant\n`
pub fn format_chatml(system: &str, user: &str) -> String {
    format!(
        "<|im_start|>system\n{system}<|im_end|>\n<|im_start|>user\n{user}<|im_end|>\n<|im_start|>assistant\n"
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_strip_think_tags_basic() {
        let input = "<think>reasoning here</think>Final answer";
        assert_eq!(strip_think_tags(input), "Final answer");
    }

    #[test]
    fn test_strip_think_tags_multiline() {
        let input = "<think>\nStep 1: ...\nStep 2: ...\n</think>\nThe result is 42.";
        assert_eq!(strip_think_tags(input), "The result is 42.");
    }

    #[test]
    fn test_strip_think_tags_no_tags() {
        let input = "No reasoning here.";
        assert_eq!(strip_think_tags(input), "No reasoning here.");
    }

    #[test]
    fn test_strip_think_tags_unclosed() {
        let input = "<think>unclosed reasoning";
        assert_eq!(strip_think_tags(input), "");
    }

    #[test]
    fn test_strip_think_tags_multiple() {
        let input = "<think>first</think>Hello <think>second</think>World";
        assert_eq!(strip_think_tags(input), "Hello World");
    }

    #[test]
    fn test_format_chatml() {
        let result = format_chatml("You are helpful.", "What is 2+2?");
        assert!(result.starts_with("<|im_start|>system\nYou are helpful.<|im_end|>"));
        assert!(result.contains("<|im_start|>user\nWhat is 2+2?<|im_end|>"));
        assert!(result.ends_with("<|im_start|>assistant\n"));
    }
}
