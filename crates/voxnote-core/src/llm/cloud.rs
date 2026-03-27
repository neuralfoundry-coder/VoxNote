use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tracing::debug;

use super::{GenerateConfig, LlmProvider};
use crate::error::LlmError;

// ── OpenAI ─────────────────────────────────────────────────────

pub struct OpenAiLlmProvider {
    client: Client,
    api_key: String,
    model: String,
    endpoint: String,
}

impl OpenAiLlmProvider {
    pub fn new(api_key: String, model: String, endpoint: Option<String>) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
            endpoint: endpoint.unwrap_or_else(|| "https://api.openai.com/v1".to_string()),
        }
    }
}

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<ChatMessage>,
    temperature: f32,
    top_p: f32,
    max_tokens: usize,
}

#[derive(Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<ChatChoice>,
}

#[derive(Deserialize)]
struct ChatChoice {
    message: ChatMessage,
}

#[async_trait]
impl LlmProvider for OpenAiLlmProvider {
    async fn generate(&self, prompt: &str, config: &GenerateConfig) -> Result<String, LlmError> {
        let request = ChatRequest {
            model: self.model.clone(),
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
            temperature: config.temperature,
            top_p: config.top_p,
            max_tokens: config.max_tokens,
        };

        let resp = self
            .client
            .post(format!("{}/chat/completions", self.endpoint))
            .bearer_auth(&self.api_key)
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::Provider(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(LlmError::Provider(format!("OpenAI API {}: {}", status, body)));
        }

        let chat_resp: ChatResponse = resp
            .json()
            .await
            .map_err(|e| LlmError::Provider(e.to_string()))?;

        chat_resp
            .choices
            .first()
            .map(|c| c.message.content.clone())
            .ok_or_else(|| LlmError::Provider("Empty response".to_string()))
    }

    fn max_context_length(&self) -> usize { 128000 }
    fn supports_grammar(&self) -> bool { false }
    fn name(&self) -> &str { "openai" }
}

// ── Anthropic ──────────────────────────────────────────────────

pub struct AnthropicLlmProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl AnthropicLlmProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }
}

#[derive(Serialize)]
struct AnthropicRequest {
    model: String,
    max_tokens: usize,
    messages: Vec<ChatMessage>,
}

#[derive(Deserialize)]
struct AnthropicResponse {
    content: Vec<ContentBlock>,
}

#[derive(Deserialize)]
struct ContentBlock {
    text: Option<String>,
}

#[async_trait]
impl LlmProvider for AnthropicLlmProvider {
    async fn generate(&self, prompt: &str, config: &GenerateConfig) -> Result<String, LlmError> {
        let request = AnthropicRequest {
            model: self.model.clone(),
            max_tokens: config.max_tokens,
            messages: vec![ChatMessage {
                role: "user".to_string(),
                content: prompt.to_string(),
            }],
        };

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request)
            .send()
            .await
            .map_err(|e| LlmError::Provider(e.to_string()))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().await.unwrap_or_default();
            return Err(LlmError::Provider(format!("Anthropic API {}: {}", status, body)));
        }

        let anthropic_resp: AnthropicResponse = resp
            .json()
            .await
            .map_err(|e| LlmError::Provider(e.to_string()))?;

        anthropic_resp
            .content
            .first()
            .and_then(|b| b.text.clone())
            .ok_or_else(|| LlmError::Provider("Empty response".to_string()))
    }

    fn max_context_length(&self) -> usize { 200000 }
    fn supports_grammar(&self) -> bool { false }
    fn name(&self) -> &str { "anthropic" }
}

// ── Gemini ─────────────────────────────────────────────────────

pub struct GeminiLlmProvider {
    client: Client,
    api_key: String,
    model: String,
}

impl GeminiLlmProvider {
    pub fn new(api_key: String, model: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model,
        }
    }
}

#[derive(Serialize)]
struct GeminiRequest {
    contents: Vec<GeminiContent>,
}

#[derive(Serialize, Deserialize)]
struct GeminiContent {
    parts: Vec<GeminiPart>,
}

#[derive(Serialize, Deserialize)]
struct GeminiPart {
    text: String,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<GeminiCandidate>,
}

#[derive(Deserialize)]
struct GeminiCandidate {
    content: GeminiContent,
}

#[async_trait]
impl LlmProvider for GeminiLlmProvider {
    async fn generate(&self, prompt: &str, _config: &GenerateConfig) -> Result<String, LlmError> {
        let request = GeminiRequest {
            contents: vec![GeminiContent {
                parts: vec![GeminiPart { text: prompt.to_string() }],
            }],
        };

        let url = format!(
            "https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}",
            self.model, self.api_key
        );

        let resp = self.client.post(&url).json(&request).send().await
            .map_err(|e| LlmError::Provider(e.to_string()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(LlmError::Provider(format!("Gemini API error: {}", body)));
        }

        let gemini_resp: GeminiResponse = resp.json().await
            .map_err(|e| LlmError::Provider(e.to_string()))?;

        gemini_resp.candidates.first()
            .and_then(|c| c.content.parts.first())
            .map(|p| p.text.clone())
            .ok_or_else(|| LlmError::Provider("Empty response".to_string()))
    }

    fn max_context_length(&self) -> usize { 1000000 }
    fn supports_grammar(&self) -> bool { false }
    fn name(&self) -> &str { "gemini" }
}

// ── Ollama (OpenAI-compatible) ─────────────────────────────────

pub struct OllamaLlmProvider {
    inner: OpenAiLlmProvider,
}

impl OllamaLlmProvider {
    pub fn new(model: String, endpoint: Option<String>) -> Self {
        let endpoint = endpoint.unwrap_or_else(|| "http://localhost:11434/v1".to_string());
        Self {
            inner: OpenAiLlmProvider::new(String::new(), model, Some(endpoint)),
        }
    }
}

#[async_trait]
impl LlmProvider for OllamaLlmProvider {
    async fn generate(&self, prompt: &str, config: &GenerateConfig) -> Result<String, LlmError> {
        self.inner.generate(prompt, config).await
    }
    fn max_context_length(&self) -> usize { 32768 }
    fn supports_grammar(&self) -> bool { false }
    fn name(&self) -> &str { "ollama" }
}

// ── Custom OpenAI-Compatible ───────────────────────────────────

pub struct CustomLlmProvider {
    inner: OpenAiLlmProvider,
    provider_name: String,
}

impl CustomLlmProvider {
    pub fn new(api_key: String, model: String, endpoint: String, name: String) -> Self {
        Self {
            inner: OpenAiLlmProvider::new(api_key, model, Some(endpoint)),
            provider_name: name,
        }
    }
}

#[async_trait]
impl LlmProvider for CustomLlmProvider {
    async fn generate(&self, prompt: &str, config: &GenerateConfig) -> Result<String, LlmError> {
        self.inner.generate(prompt, config).await
    }
    fn max_context_length(&self) -> usize { 32768 }
    fn supports_grammar(&self) -> bool { false }
    fn name(&self) -> &str { &self.provider_name }
}
