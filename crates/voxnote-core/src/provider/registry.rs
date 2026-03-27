use std::collections::HashMap;
use std::sync::Arc;

use crate::llm::LlmProvider;
use crate::stt::SttProvider;

/// 엔진 타입
#[derive(Debug, Clone, Hash, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub enum EngineType {
    Stt,
    Llm,
    Tts,
}

/// Provider 레지스트리 — 런타임 동적 등록/조회
///
/// 로컬 및 클라우드 프로바이더를 동일한 인터페이스로 관리합니다.
pub struct ProviderRegistry {
    llm_providers: HashMap<String, Arc<dyn LlmProvider>>,
    stt_providers: HashMap<String, Arc<dyn SttProvider>>,
    active_llm: Option<String>,
    active_stt: Option<String>,
}

impl ProviderRegistry {
    pub fn new() -> Self {
        Self {
            llm_providers: HashMap::new(),
            stt_providers: HashMap::new(),
            active_llm: None,
            active_stt: None,
        }
    }

    // ── LLM ──

    pub fn register_llm(&mut self, name: &str, provider: Arc<dyn LlmProvider>) {
        self.llm_providers.insert(name.to_string(), provider);
        if self.active_llm.is_none() {
            self.active_llm = Some(name.to_string());
        }
    }

    pub fn set_active_llm(&mut self, name: &str) -> bool {
        if self.llm_providers.contains_key(name) {
            self.active_llm = Some(name.to_string());
            true
        } else {
            false
        }
    }

    pub fn active_llm(&self) -> Option<Arc<dyn LlmProvider>> {
        self.active_llm
            .as_ref()
            .and_then(|name| self.llm_providers.get(name))
            .cloned()
    }

    pub fn list_llm_providers(&self) -> Vec<String> {
        self.llm_providers.keys().cloned().collect()
    }

    // ── STT ──

    pub fn register_stt(&mut self, name: &str, provider: Arc<dyn SttProvider>) {
        self.stt_providers.insert(name.to_string(), provider);
        if self.active_stt.is_none() {
            self.active_stt = Some(name.to_string());
        }
    }

    pub fn set_active_stt(&mut self, name: &str) -> bool {
        if self.stt_providers.contains_key(name) {
            self.active_stt = Some(name.to_string());
            true
        } else {
            false
        }
    }

    pub fn active_stt(&self) -> Option<Arc<dyn SttProvider>> {
        self.active_stt
            .as_ref()
            .and_then(|name| self.stt_providers.get(name))
            .cloned()
    }

    pub fn list_stt_providers(&self) -> Vec<String> {
        self.stt_providers.keys().cloned().collect()
    }
}
