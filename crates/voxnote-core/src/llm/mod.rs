use async_trait::async_trait;

use crate::error::LlmError;

#[cfg(feature = "llm")]
pub mod local;

#[cfg(feature = "cloud-providers")]
pub mod cloud;

pub mod prompt;
pub mod templates;

/// LLM 생성 설정
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct GenerateConfig {
    /// 온도 (0.0 ~ 2.0, 기본: 0.3)
    pub temperature: f32,
    /// Top-P (0.0 ~ 1.0, 기본: 0.9)
    pub top_p: f32,
    /// 최대 생성 토큰 수
    pub max_tokens: usize,
    /// GBNF Grammar (JSON/Markdown 강제용)
    pub grammar: Option<String>,
}

impl Default for GenerateConfig {
    fn default() -> Self {
        Self {
            temperature: 0.3,
            top_p: 0.9,
            max_tokens: 2048,
            grammar: None,
        }
    }
}

/// LLM 생성 토큰
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Token {
    pub text: String,
    pub is_final: bool,
}

/// LLM Provider 트레이트 — 확장 포인트
///
/// 로컬(llama.cpp)과 클라우드(OpenAI, Anthropic, Gemini, Ollama 등)
/// 구현을 동일한 인터페이스로 통합합니다.
#[async_trait]
pub trait LlmProvider: Send + Sync {
    /// 프롬프트를 기반으로 텍스트 생성 (논스트리밍)
    async fn generate(&self, prompt: &str, config: &GenerateConfig) -> Result<String, LlmError>;

    /// 최대 컨텍스트 길이 (토큰 수)
    fn max_context_length(&self) -> usize;

    /// GBNF Grammar 지원 여부
    fn supports_grammar(&self) -> bool;

    /// Provider 이름
    fn name(&self) -> &str;
}
