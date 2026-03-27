//! E2E: Provider 레지스트리 및 폴백 시나리오 테스트

use voxnote_core::provider::registry::{EngineType, ProviderRegistry};
use voxnote_core::llm::{GenerateConfig, LlmProvider};
use voxnote_core::error::LlmError;
use std::sync::Arc;
use async_trait::async_trait;

// ── Mock Providers ──────────────────────────────────────────────

struct MockLocalLlm;

#[async_trait]
impl LlmProvider for MockLocalLlm {
    async fn generate(&self, prompt: &str, _config: &GenerateConfig) -> Result<String, LlmError> {
        let preview: String = prompt.chars().take(50).collect();
        Ok(format!("[local] Summary of: {}...", preview))
    }
    fn max_context_length(&self) -> usize { 4096 }
    fn supports_grammar(&self) -> bool { true }
    fn name(&self) -> &str { "mock-local" }
}

struct MockCloudLlm {
    should_fail: bool,
}

#[async_trait]
impl LlmProvider for MockCloudLlm {
    async fn generate(&self, prompt: &str, _config: &GenerateConfig) -> Result<String, LlmError> {
        if self.should_fail {
            return Err(LlmError::Provider("Cloud API timeout".to_string()));
        }
        let preview: String = prompt.chars().take(50).collect();
        Ok(format!("[cloud] Summary of: {}...", preview))
    }
    fn max_context_length(&self) -> usize { 128000 }
    fn supports_grammar(&self) -> bool { false }
    fn name(&self) -> &str { "mock-cloud" }
}

// ── E2E-501: Provider 등록/조회/전환 ────────────────────────────

#[test]
fn e2e_501_provider_registry_lifecycle() {
    let mut registry = ProviderRegistry::new();

    // 등록
    let local: Arc<dyn LlmProvider> = Arc::new(MockLocalLlm);
    let cloud: Arc<dyn LlmProvider> = Arc::new(MockCloudLlm { should_fail: false });

    registry.register_llm("local", local);
    registry.register_llm("openai", cloud);

    // 목록 확인
    let providers = registry.list_llm_providers();
    assert_eq!(providers.len(), 2);

    // 기본 활성 (첫 번째 등록된 것)
    let active = registry.active_llm().unwrap();
    assert_eq!(active.name(), "mock-local");

    // 전환
    assert!(registry.set_active_llm("openai"));
    let active = registry.active_llm().unwrap();
    assert_eq!(active.name(), "mock-cloud");

    // 존재하지 않는 프로바이더
    assert!(!registry.set_active_llm("nonexistent"));
}

// ── E2E-502: 프로바이더 실행 테스트 ─────────────────────────────

#[tokio::test]
async fn e2e_502_provider_execution() {
    let mut registry = ProviderRegistry::new();
    let local: Arc<dyn LlmProvider> = Arc::new(MockLocalLlm);
    registry.register_llm("local", local);

    let provider = registry.active_llm().unwrap();
    let config = GenerateConfig::default();
    let result = provider
        .generate("오늘 회의에서 프로젝트 일정을 논의했습니다", &config)
        .await
        .unwrap();

    assert!(result.contains("[local]"));
    assert!(result.contains("Summary"));
}

// ── E2E-503: 클라우드 실패 → 로컬 폴백 ─────────────────────────

#[tokio::test]
async fn e2e_503_cloud_to_local_fallback() {
    use voxnote_core::provider::fallback::FallbackLlmProvider;

    let cloud: Arc<dyn LlmProvider> = Arc::new(MockCloudLlm { should_fail: true });
    let local: Arc<dyn LlmProvider> = Arc::new(MockLocalLlm);

    let fallback = FallbackLlmProvider::new(cloud, local).with_max_retries(2);
    let config = GenerateConfig::default();

    let (result, used_fallback) = fallback
        .generate_with_fallback("Test prompt for fallback", &config)
        .await
        .unwrap();

    assert!(used_fallback, "Should have fallen back to local");
    assert!(result.contains("[local]"), "Result should be from local provider");
}

// ── E2E-504: 클라우드 성공 시 폴백 미사용 ──────────────────────

#[tokio::test]
async fn e2e_504_cloud_success_no_fallback() {
    use voxnote_core::provider::fallback::FallbackLlmProvider;

    let cloud: Arc<dyn LlmProvider> = Arc::new(MockCloudLlm { should_fail: false });
    let local: Arc<dyn LlmProvider> = Arc::new(MockLocalLlm);

    let fallback = FallbackLlmProvider::new(cloud, local);
    let config = GenerateConfig::default();

    let (result, used_fallback) = fallback
        .generate_with_fallback("Test prompt", &config)
        .await
        .unwrap();

    assert!(!used_fallback, "Should NOT have fallen back");
    assert!(result.contains("[cloud]"));
}

// ── E2E-505: 양쪽 모두 실패 ────────────────────────────────────

#[tokio::test]
async fn e2e_505_both_providers_fail() {
    use voxnote_core::provider::fallback::FallbackLlmProvider;

    let cloud: Arc<dyn LlmProvider> = Arc::new(MockCloudLlm { should_fail: true });
    let local: Arc<dyn LlmProvider> = Arc::new(MockCloudLlm { should_fail: true }); // 로컬도 실패

    let fallback = FallbackLlmProvider::new(cloud, local).with_max_retries(1);
    let config = GenerateConfig::default();

    let result = fallback.generate_with_fallback("Test", &config).await;
    assert!(result.is_err(), "Both failing should return error");
}
