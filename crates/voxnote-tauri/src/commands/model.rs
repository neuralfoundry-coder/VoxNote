use tauri::State;
use voxnote_core::model_manager::{ModelEntry, ModelType};

use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub model_type: String,
    pub size_display: String,
    pub is_downloaded: bool,
    pub gpu_recommended: bool,
    pub description: Option<String>,
}

#[tauri::command]
pub async fn list_models(state: State<'_, AppState>) -> Result<Vec<ModelInfo>, String> {
    let registry = state.registry.lock().map_err(|e| e.to_string())?;

    let Some(ref reg) = *registry else {
        return Ok(Vec::new());
    };

    let config = state.config.lock().map_err(|e| e.to_string())?;
    let models_dir = config.models_dir();

    let models = reg
        .models
        .iter()
        .map(|m| {
            let is_downloaded = models_dir.join(&m.id).exists();
            ModelInfo {
                id: m.id.clone(),
                name: m.name.clone(),
                model_type: format!("{:?}", m.model_type).to_lowercase(),
                size_display: m.size_display(),
                is_downloaded,
                gpu_recommended: m.gpu_recommended,
                description: m.description.clone(),
            }
        })
        .collect();

    Ok(models)
}

#[tauri::command]
pub async fn download_model(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<String, String> {
    // TODO: Phase 1 — ModelDownloader를 사용하여 비동기 다운로드 실행
    // Tauri event emit으로 진행률 전달
    Ok(format!("Download started: {}", model_id))
}

#[tauri::command]
pub async fn delete_model(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<(), String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let disk = voxnote_core::model_manager::disk::DiskManager::new(
        config.models_dir(),
        config.model.max_cache_mb,
    );
    disk.delete_model(&model_id).map_err(|e| e.to_string())
}
