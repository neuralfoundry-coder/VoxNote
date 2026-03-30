use tauri::State;

use voxnote_core::storage::ProviderConfigRow;

use crate::state::AppState;

#[derive(serde::Serialize, serde::Deserialize)]
pub struct ProviderConfig {
    pub id: Option<String>,
    pub engine_type: String,
    pub provider: String,
    pub model_id: Option<String>,
    pub endpoint: Option<String>,
    pub is_active: bool,
    pub api_key: Option<String>,
}

#[tauri::command]
pub async fn get_provider_config(
    state: State<'_, AppState>,
) -> Result<Vec<ProviderConfig>, String> {
    let configs = state
        .store
        .get_provider_configs()
        .map_err(|e| e.to_string())?;

    Ok(configs
        .into_iter()
        .map(|c| {
            let api_key = c
                .config_json
                .as_deref()
                .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
                .and_then(|v| v.get("api_key").and_then(|k| k.as_str()).map(String::from));

            ProviderConfig {
                id: Some(c.id),
                engine_type: c.engine_type,
                provider: c.provider,
                model_id: c.model_id,
                endpoint: c.endpoint,
                is_active: c.is_active,
                api_key,
            }
        })
        .collect())
}

#[tauri::command]
pub async fn set_provider_config(
    state: State<'_, AppState>,
    config: ProviderConfig,
) -> Result<(), String> {
    // API 키를 config_json으로 직렬화
    let config_json = config.api_key.as_ref().map(|key| {
        serde_json::json!({ "api_key": key }).to_string()
    });

    let now = chrono::Utc::now();
    let row = ProviderConfigRow {
        id: config.id.unwrap_or_else(|| uuid::Uuid::new_v4().to_string()),
        engine_type: config.engine_type.clone(),
        provider: config.provider.clone(),
        model_id: config.model_id,
        endpoint: config.endpoint.clone(),
        is_active: config.is_active,
        config_json,
        created_at: now,
        updated_at: now,
    };

    // 활성화 시 같은 engine_type의 다른 provider 비활성화
    if config.is_active {
        state
            .store
            .deactivate_providers(&config.engine_type)
            .map_err(|e| e.to_string())?;
    }

    state
        .store
        .upsert_provider_config(&row)
        .map_err(|e| e.to_string())?;

    // Provider Registry 업데이트
    if config.is_active && config.engine_type == "llm" {
        if let Ok(mut registry) = state.provider_registry.lock() {
            let api_key = config.api_key.as_deref().unwrap_or("");
            if let Some(provider) = crate::state::create_llm_provider(
                &config.provider,
                api_key,
                row.model_id.as_deref(),
                config.endpoint.as_deref(),
            ) {
                registry.register_llm(&config.provider, provider);
                registry.set_active_llm(&config.provider);
            }
        }
    }

    Ok(())
}

#[tauri::command]
pub async fn test_provider(
    state: State<'_, AppState>,
    provider: String,
) -> Result<String, String> {
    // 해당 provider의 설정을 DB에서 조회
    let configs = state
        .store
        .get_provider_configs()
        .map_err(|e| e.to_string())?;

    let config = configs
        .iter()
        .find(|c| c.provider == provider)
        .ok_or_else(|| format!("Provider '{}' not configured", provider))?;

    let api_key = config
        .config_json
        .as_deref()
        .and_then(|s| serde_json::from_str::<serde_json::Value>(s).ok())
        .and_then(|v| v.get("api_key").and_then(|k| k.as_str()).map(String::from))
        .unwrap_or_default();

    if api_key.is_empty() && provider != "ollama" {
        return Err(format!("No API key configured for {}", provider));
    }

    // 간단한 테스트 생성 요청
    use voxnote_core::llm::{GenerateConfig, LlmProvider};

    if let Some(llm) = crate::state::create_llm_provider(
        &provider,
        &api_key,
        config.model_id.as_deref(),
        config.endpoint.as_deref(),
    ) {
        let test_config = GenerateConfig {
            max_tokens: 10,
            ..Default::default()
        };
        match llm.generate("Say 'OK' in one word.", &test_config).await {
            Ok(_) => return Ok(format!("Provider {} connection test: OK", provider)),
            Err(e) => return Err(format!("Provider {} test failed: {}", provider, e)),
        }
    }

    Err(format!("Could not create provider instance for {}", provider))
}

#[tauri::command]
pub async fn list_available_providers() -> Result<Vec<String>, String> {
    let mut providers = vec!["whisper-local".to_string()];

    providers.extend([
        "openai".to_string(),
        "anthropic".to_string(),
        "gemini".to_string(),
        "ollama".to_string(),
    ]);

    Ok(providers)
}
