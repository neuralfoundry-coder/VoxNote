use tauri::State;

use crate::state::AppState;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProviderConfig {
    pub engine_type: String,
    pub provider: String,
    pub model_id: Option<String>,
    pub endpoint: Option<String>,
    pub is_active: bool,
}

#[tauri::command]
pub async fn get_provider_config(
    state: State<'_, AppState>,
) -> Result<Vec<ProviderConfig>, String> {
    // TODO: DB에서 provider_config 테이블 조회
    Ok(vec![
        ProviderConfig {
            engine_type: "stt".to_string(),
            provider: "whisper-local".to_string(),
            model_id: Some("whisper-tiny-q8".to_string()),
            endpoint: None,
            is_active: true,
        },
    ])
}

#[tauri::command]
pub async fn set_provider_config(
    state: State<'_, AppState>,
    config: ProviderConfig,
) -> Result<(), String> {
    // TODO: provider_config 테이블에 저장
    Ok(())
}

#[tauri::command]
pub async fn test_provider(
    state: State<'_, AppState>,
    provider: String,
) -> Result<String, String> {
    // TODO: 실제 연결 테스트
    Ok(format!("Provider {} connection test: OK", provider))
}

#[tauri::command]
pub async fn list_available_providers() -> Result<Vec<String>, String> {
    Ok(vec![
        "whisper-local".to_string(),
        "openai".to_string(),
        "anthropic".to_string(),
        "gemini".to_string(),
        "ollama".to_string(),
    ])
}
