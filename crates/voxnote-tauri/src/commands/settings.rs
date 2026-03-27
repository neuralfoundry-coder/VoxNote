use tauri::State;
use voxnote_core::config::AppConfig;

use crate::state::AppState;

#[tauri::command]
pub async fn get_settings(state: State<'_, AppState>) -> Result<AppConfig, String> {
    let config = state.config.lock().map_err(|e| e.to_string())?;
    Ok(config.clone())
}

#[tauri::command]
pub async fn update_settings(
    state: State<'_, AppState>,
    config: AppConfig,
) -> Result<(), String> {
    let mut current = state.config.lock().map_err(|e| e.to_string())?;
    *current = config.clone();

    // 설정 파일에 저장
    let base_dir = dirs::home_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join(".voxnote");
    let config_path = base_dir.join("config.toml");

    config.save(&config_path).map_err(|e| e.to_string())
}
