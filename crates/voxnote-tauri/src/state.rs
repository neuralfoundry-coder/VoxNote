use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use voxnote_core::config::AppConfig;
use voxnote_core::model_manager::ModelRegistry;
use voxnote_core::models::RecordingState;
use voxnote_core::storage::SqliteStore;

/// Tauri 앱 상태
pub struct AppState {
    pub config: Arc<Mutex<AppConfig>>,
    pub store: Arc<SqliteStore>,
    pub recording_state: Arc<Mutex<RecordingState>>,
    pub registry: Arc<Mutex<Option<ModelRegistry>>>,
    pub stt_model_path: Arc<Mutex<Option<PathBuf>>>,
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

        // STT 모델 경로 자동 탐색
        let models_dir = config.models_dir();
        let stt_model_path = find_stt_model(&models_dir);

        if let Some(ref path) = stt_model_path {
            tracing::info!("STT model found: {:?}", path);
        } else {
            tracing::warn!("No STT model found in {:?}", models_dir);
        }

        Ok(Self {
            config: Arc::new(Mutex::new(config)),
            store: Arc::new(store),
            recording_state: Arc::new(Mutex::new(RecordingState::Idle)),
            registry: Arc::new(Mutex::new(registry)),
            stt_model_path: Arc::new(Mutex::new(stt_model_path)),
        })
    }
}

/// models 디렉토리에서 첫 번째 STT 모델 파일 탐색
fn find_stt_model(models_dir: &std::path::Path) -> Option<PathBuf> {
    if !models_dir.exists() {
        return None;
    }

    // ggml-*.bin 패턴 탐색
    let patterns = ["ggml-tiny.bin", "ggml-base.bin", "ggml-small.bin", "ggml-large-v3-turbo-q5_0.bin"];
    for name in &patterns {
        let path = models_dir.join(name);
        if path.exists() {
            return Some(path);
        }
    }

    // 아무 .bin 파일
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
