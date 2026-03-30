use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use voxnote_core::config::AppConfig;
use voxnote_core::model_manager::ModelRegistry;
use voxnote_core::models::RecordingState;
use voxnote_core::provider::registry::ProviderRegistry;
use voxnote_core::stt::SttProvider;
use voxnote_core::storage::SqliteStore;

/// Tauri 앱 상태
pub struct AppState {
    pub config: Arc<Mutex<AppConfig>>,
    pub store: Arc<SqliteStore>,
    pub recording_state: Arc<Mutex<RecordingState>>,
    pub registry: Arc<Mutex<Option<ModelRegistry>>>,
    pub provider_registry: Arc<Mutex<ProviderRegistry>>,
    pub stt_model_path: Arc<Mutex<Option<PathBuf>>>,
    /// STT Provider (녹음 시 지연 로드, 별도 프로세스로 검증 후 로드)
    pub stt_provider: Arc<Mutex<Option<Arc<dyn SttProvider>>>>,
}

impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        let base_dir = dirs::home_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join(".voxnote");

        let config_path = base_dir.join("config.toml");
        let config = AppConfig::load(&config_path).unwrap_or_default();

        let db_path = config.data_dir().join("voxnote.db");
        let store = SqliteStore::open(&db_path)?;

        // 모델 레지스트리
        let registry_path = base_dir.join("models").join("registry.toml");
        let registry = ModelRegistry::load(&registry_path).ok();

        // STT 모델 경로만 저장 (로드는 녹음 시점에 수행)
        let models_dir = config.models_dir();
        let stt_model_path = find_stt_model(&models_dir);

        if let Some(ref path) = stt_model_path {
            tracing::info!("STT model found: {:?}", path);
        } else {
            tracing::warn!("No STT model found in {:?}", models_dir);
        }

        // Provider 레지스트리 초기화 (DB에서 설정 로드)
        let provider_registry = init_provider_registry(&store);

        Ok(Self {
            config: Arc::new(Mutex::new(config)),
            store: Arc::new(store),
            recording_state: Arc::new(Mutex::new(RecordingState::Idle)),
            registry: Arc::new(Mutex::new(registry)),
            provider_registry: Arc::new(Mutex::new(provider_registry)),
            stt_model_path: Arc::new(Mutex::new(stt_model_path)),
            stt_provider: Arc::new(Mutex::new(None)),
        })
    }
}

/// STT Provider 로드 (직접 로드 — whisper-rs 0.16은 호환 문제 해결됨)
pub fn load_stt_provider(model_path: &std::path::Path) -> Option<Arc<dyn SttProvider>> {
    let metadata = std::fs::metadata(model_path).ok()?;
    if metadata.len() < 1_000_000 {
        tracing::error!("STT model too small ({} bytes): {:?}", metadata.len(), model_path);
        return None;
    }

    tracing::info!("Loading STT model: {:?} ({:.1} MB)", model_path, metadata.len() as f64 / 1_048_576.0);
    match voxnote_core::stt::whisper::LocalSttProvider::new(model_path.to_path_buf()) {
        Ok(provider) => {
            tracing::info!("STT model loaded successfully");
            Some(Arc::new(provider) as Arc<dyn SttProvider>)
        }
        Err(e) => {
            tracing::error!("Failed to load STT model: {}", e);
            None
        }
    }
}

/// DB에서 provider_config를 읽어 ProviderRegistry 초기화
fn init_provider_registry(store: &SqliteStore) -> ProviderRegistry {
    let mut registry = ProviderRegistry::new();

    let configs = match store.get_provider_configs() {
        Ok(c) => c,
        Err(e) => {
            tracing::warn!("Failed to load provider configs: {}", e);
            return registry;
        }
    };

    for config in &configs {
        if !config.is_active {
            continue;
        }

        let config_json: serde_json::Value = config
            .config_json
            .as_deref()
            .and_then(|s| serde_json::from_str(s).ok())
            .unwrap_or(serde_json::Value::Null);

        let api_key = config_json
            .get("api_key")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string();

        match config.engine_type.as_str() {
            "llm" => {
                if let Some(provider) = create_llm_provider(
                    &config.provider,
                    &api_key,
                    config.model_id.as_deref(),
                    config.endpoint.as_deref(),
                ) {
                    registry.register_llm(&config.provider, provider);
                    registry.set_active_llm(&config.provider);
                    tracing::info!("Registered active LLM provider: {}", config.provider);
                }
            }
            "stt" => {
                tracing::info!("Active STT provider config: {}", config.provider);
            }
            _ => {}
        }
    }

    registry
}

/// cloud provider 이름으로 LLM provider 인스턴스 생성
pub fn create_llm_provider(
    provider_name: &str,
    api_key: &str,
    model_id: Option<&str>,
    endpoint: Option<&str>,
) -> Option<Arc<dyn voxnote_core::llm::LlmProvider>> {
    use voxnote_core::llm::cloud::*;

    if api_key.is_empty() && !matches!(provider_name, "ollama" | "llama-local") {
        tracing::warn!("No API key for provider: {}", provider_name);
        return None;
    }

    match provider_name {
        "llama-local" => {
            let model_path = model_id.map(std::path::PathBuf::from)?;
            match voxnote_core::llm::local::LocalLlmProvider::new(model_path, 4096, 99) {
                Ok(p) => Some(Arc::new(p)),
                Err(e) => {
                    tracing::error!("Failed to create local LLM provider: {}", e);
                    None
                }
            }
        }
        "openai" => Some(Arc::new(OpenAiLlmProvider::new(
            api_key.to_string(),
            model_id.unwrap_or("gpt-4o").to_string(),
            endpoint.map(String::from),
        ))),
        "anthropic" => Some(Arc::new(AnthropicLlmProvider::new(
            api_key.to_string(),
            model_id.unwrap_or("claude-sonnet-4-6-20250514").to_string(),
        ))),
        "gemini" => Some(Arc::new(GeminiLlmProvider::new(
            api_key.to_string(),
            model_id.unwrap_or("gemini-2.0-flash").to_string(),
        ))),
        "ollama" => Some(Arc::new(OllamaLlmProvider::new(
            model_id.unwrap_or("llama3").to_string(),
            endpoint.map(String::from),
        ))),
        _ => {
            tracing::warn!("Unknown LLM provider: {}", provider_name);
            None
        }
    }
}

/// models 디렉토리에서 첫 번째 STT 모델 파일 탐색
fn find_stt_model(models_dir: &std::path::Path) -> Option<PathBuf> {
    if !models_dir.exists() {
        return None;
    }

    let patterns = [
        "ggml-tiny.bin",
        "ggml-base.bin",
        "ggml-small.bin",
        "ggml-large-v3-turbo-q5_0.bin",
    ];
    for name in &patterns {
        let path = models_dir.join(name);
        if path.exists() {
            return Some(path);
        }
    }

    if let Ok(entries) = std::fs::read_dir(models_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "bin").unwrap_or(false) {
                return Some(path);
            }
        }
    }

    None
}
