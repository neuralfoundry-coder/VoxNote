use std::sync::Arc;
use tauri::{Emitter, State};

use voxnote_core::model_manager::downloader::{DownloadProgress, ModelDownloader};

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
            // 모델 파일이 존재하는지 확인 (ID 또는 알려진 파일명)
            let is_downloaded = models_dir.join(&m.id).exists()
                || models_dir.join(format!("{}.bin", m.id)).exists();
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
    app: tauri::AppHandle,
    state: State<'_, AppState>,
    model_id: String,
) -> Result<String, String> {
    // 레지스트리에서 모델 정보 조회
    let (url, sha256, dest) = {
        let registry = state.registry.lock().map_err(|e| e.to_string())?;
        let reg = registry
            .as_ref()
            .ok_or("Model registry not loaded")?;

        let entry = reg
            .models
            .iter()
            .find(|m| m.id == model_id)
            .ok_or_else(|| format!("Model '{}' not found in registry", model_id))?;

        let config = state.config.lock().map_err(|e| e.to_string())?;
        let models_dir = config.models_dir();
        let dest = models_dir.join(&entry.id);

        (
            entry.download_url.clone(),
            entry.sha256.clone(),
            dest,
        )
    };

    // 비동기 다운로드 시작
    let downloader = ModelDownloader::new();
    let model_id_clone = model_id.clone();
    let app_clone = app.clone();

    let progress_cb: Option<Box<dyn Fn(DownloadProgress) + Send + Sync>> =
        Some(Box::new(move |progress: DownloadProgress| {
            let _ = app_clone.emit("model:download-progress", &progress);
        }));

    tokio::spawn(async move {
        match downloader
            .download(&model_id_clone, &url, &dest, &sha256, progress_cb)
            .await
        {
            Ok(path) => {
                tracing::info!("Model downloaded: {:?}", path);
                let _ = app.emit(
                    "model:download-complete",
                    serde_json::json!({ "model_id": model_id_clone, "path": path.to_string_lossy() }),
                );
            }
            Err(e) => {
                tracing::error!("Model download failed: {}", e);
                let _ = app.emit(
                    "model:download-error",
                    serde_json::json!({ "model_id": model_id_clone, "error": e.to_string() }),
                );
            }
        }
    });

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
