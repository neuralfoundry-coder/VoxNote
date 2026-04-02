use std::time::Instant;

use tauri::{Emitter, State};

use voxnote_core::model_manager::downloader::{DownloadProgress, ModelDownloader};
use voxnote_core::storage::ProviderConfigRow;

use crate::state::AppState;

#[derive(serde::Serialize)]
pub struct ModelTestResult {
    pub success: bool,
    pub output: String,
    pub duration_ms: u64,
}

#[derive(serde::Serialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub model_type: String,
    pub size_display: String,
    pub is_downloaded: bool,
    pub is_active: bool,
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

    // 활성 모델 ID 조회 (DB에서)
    let active_model_ids: std::collections::HashSet<String> = state
        .store
        .get_provider_configs()
        .unwrap_or_default()
        .into_iter()
        .filter(|c| c.is_active)
        .filter_map(|c| c.model_id)
        .collect();

    let models = reg
        .models
        .iter()
        .map(|m| {
            // 모델 파일/디렉토리가 존재하는지 확인
            let model_path = models_dir.join(&m.id);
            let is_downloaded = model_path.exists()
                || models_dir.join(format!("{}.bin", m.id)).exists()
                || model_path.is_dir();
            let is_active = active_model_ids.contains(&m.id);
            ModelInfo {
                id: m.id.clone(),
                name: m.name.clone(),
                model_type: format!("{:?}", m.model_type).to_lowercase(),
                size_display: m.size_display(),
                is_downloaded,
                is_active,
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
    let (url, sha256, dest, bundle_files) = {
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
            entry.files.clone(),
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
        let result = if let Some(files) = bundle_files {
            // 멀티파일 번들 다운로드
            downloader
                .download_bundle(&model_id_clone, &files, &dest, progress_cb)
                .await
        } else {
            // 단일 파일 다운로드
            downloader
                .download(&model_id_clone, &url, &dest, &sha256, progress_cb)
                .await
        };

        match result {
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

/// 모델 활성화 — 다운로드된 모델을 현재 STT/LLM 엔진으로 설정
///
/// 1. 모델 타입(stt/llm)에 따라 provider_config DB 업데이트
/// 2. 해당 엔진의 기존 활성 설정 비활성화
/// 3. STT: stt_model_path + stt_provider 캐시 무효화
/// 4. LLM: provider_registry에 등록 + 활성화
#[tauri::command]
pub async fn activate_model(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<String, String> {
    // 레지스트리에서 모델 정보 조회
    let (model_type, model_name) = {
        let registry = state.registry.lock().map_err(|e| e.to_string())?;
        let reg = registry.as_ref().ok_or("Model registry not loaded")?;
        let entry = reg
            .models
            .iter()
            .find(|m| m.id == model_id)
            .ok_or_else(|| format!("Model '{}' not found in registry", model_id))?;
        (
            format!("{:?}", entry.model_type).to_lowercase(),
            entry.name.clone(),
        )
    };

    // 모델 파일/디렉토리 존재 확인
    let config = state.config.lock().map_err(|e| e.to_string())?;
    let models_dir = config.models_dir();
    let model_path = models_dir.join(&model_id);

    if !model_path.exists() {
        return Err(format!("Model '{}' is not downloaded", model_id));
    }

    let engine_type = model_type.as_str();

    // provider 이름 결정
    let provider_name = match engine_type {
        "stt" => crate::state::infer_stt_provider_type(&model_path).to_string() + "-local",
        "llm" => "llama-local".to_string(),
        _ => return Err(format!("Unsupported model type: {}", engine_type)),
    };

    // DB: 같은 engine_type의 기존 활성 설정 비활성화
    state
        .store
        .deactivate_providers(engine_type)
        .map_err(|e| e.to_string())?;

    // DB: 새 설정 upsert
    let now = chrono::Utc::now();
    let row = ProviderConfigRow {
        id: uuid::Uuid::new_v4().to_string(),
        engine_type: engine_type.to_string(),
        provider: provider_name.clone(),
        model_id: Some(model_id.clone()),
        endpoint: None,
        is_active: true,
        config_json: None,
        created_at: now,
        updated_at: now,
    };
    state
        .store
        .upsert_provider_config(&row)
        .map_err(|e| e.to_string())?;

    // 엔진별 런타임 상태 업데이트
    match engine_type {
        "stt" => {
            // STT 모델 경로 업데이트 + 캐시 무효화
            if let Ok(mut path_guard) = state.stt_model_path.lock() {
                *path_guard = Some(model_path);
            }
            if let Ok(mut provider_guard) = state.stt_provider.lock() {
                *provider_guard = None; // 다음 녹음 시 재로드
            }
            tracing::info!("Activated STT model: {} ({})", model_name, provider_name);
        }
        "llm" => {
            // LLM provider 생성 + 등록
            let model_path_str = model_path.to_string_lossy().to_string();
            if let Some(provider) = crate::state::create_llm_provider(
                "llama-local",
                "",
                Some(&model_path_str),
                None,
            ) {
                if let Ok(mut registry) = state.provider_registry.lock() {
                    registry.register_llm("llama-local", provider);
                    registry.set_active_llm("llama-local");
                }
            }
            tracing::info!("Activated LLM model: {} (llama-local)", model_name);
        }
        _ => {}
    }

    Ok(format!("Model '{}' activated as {}", model_name, provider_name))
}

/// 모델 테스트 — 다운로드된 모델을 로드하고 간단한 추론을 실행하여 정상 동작 확인
#[tauri::command]
pub async fn test_model(
    state: State<'_, AppState>,
    model_id: String,
) -> Result<ModelTestResult, String> {
    // 레지스트리에서 모델 정보 조회
    let model_type = {
        let registry = state.registry.lock().map_err(|e| e.to_string())?;
        let reg = registry.as_ref().ok_or("Model registry not loaded")?;
        let entry = reg
            .models
            .iter()
            .find(|m| m.id == model_id)
            .ok_or_else(|| format!("Model '{}' not found in registry", model_id))?;
        format!("{:?}", entry.model_type).to_lowercase()
    };

    // 모델 경로 확인
    let model_path = {
        let config = state.config.lock().map_err(|e| e.to_string())?;
        config.models_dir().join(&model_id)
    };

    if !model_path.exists() {
        return Err(format!("Model '{}' is not downloaded", model_id));
    }

    let start = Instant::now();

    match model_type.as_str() {
        "stt" => test_stt_model(&model_path).await,
        "llm" => test_llm_model(&model_path).await,
        _ => Err(format!("Testing not supported for model type: {}", model_type)),
    }
    .map(|mut result| {
        result.duration_ms = start.elapsed().as_millis() as u64;
        result
    })
}

async fn test_stt_model(model_path: &std::path::Path) -> Result<ModelTestResult, String> {
    use std::sync::Arc;
    use voxnote_core::stt::SttProvider;

    let provider_type = crate::state::infer_stt_provider_type(model_path);

    // 직접 provider 생성하여 에러 메시지 캡처
    let provider: Arc<dyn SttProvider> = match provider_type {
        "whisper" | "" => {
            let metadata = std::fs::metadata(model_path)
                .map_err(|e| format!("Cannot read model file: {}", e))?;
            if metadata.len() < 1_000_000 {
                return Err(format!("Model file too small ({} bytes)", metadata.len()));
            }
            Arc::new(
                voxnote_core::stt::whisper::LocalSttProvider::new(model_path.to_path_buf())
                    .map_err(|e| format!("Whisper load failed: {}", e))?,
            )
        }
        #[cfg(feature = "stt-onnx")]
        "sensevoice" => {
            if !model_path.is_dir() {
                return Err(format!("SenseVoice requires a directory, got: {:?}", model_path));
            }
            Arc::new(
                voxnote_core::stt::sensevoice::SenseVoiceSttProvider::new(model_path)
                    .map_err(|e| format!("SenseVoice load failed: {}", e))?,
            )
        }
        #[cfg(feature = "stt-onnx")]
        "qwen-asr" => {
            if !model_path.is_dir() {
                return Err(format!("Qwen-ASR requires a directory, got: {:?}", model_path));
            }
            Arc::new(
                voxnote_core::stt::qwen_asr::QwenAsrSttProvider::new(model_path)
                    .map_err(|e| format!("Qwen-ASR load failed: {}", e))?,
            )
        }
        _ => return Err(format!("Unknown STT provider type: '{}'", provider_type)),
    };

    // 2초 무음 (16kHz mono) — 모델 로드 + 추론 파이프라인 검증
    let silence = vec![0.0f32; 32000];
    let chunk = voxnote_core::audio::AudioChunk::new(silence, 0);

    match provider.transcribe(&chunk, "test").await {
        Ok(segments) => {
            let text: String = segments.iter().map(|s| s.text.as_str()).collect::<Vec<_>>().join(" ");
            Ok(ModelTestResult {
                success: true,
                output: if text.trim().is_empty() {
                    "Model loaded and inference OK (silence input)".to_string()
                } else {
                    format!("OK: \"{}\"", text.trim())
                },
                duration_ms: 0, // filled by caller
            })
        }
        Err(e) => Ok(ModelTestResult {
            success: false,
            output: format!("Inference error: {}", e),
            duration_ms: 0,
        }),
    }
}

async fn test_llm_model(model_path: &std::path::Path) -> Result<ModelTestResult, String> {
    let model_path_str = model_path.to_string_lossy().to_string();
    let provider = crate::state::create_llm_provider(
        "llama-local",
        "",
        Some(&model_path_str),
        None,
    )
    .ok_or_else(|| format!("Failed to load LLM model: {:?}", model_path))?;

    let config = voxnote_core::llm::GenerateConfig {
        max_tokens: 10,
        ..Default::default()
    };

    match provider.generate("Say 'OK' in one word.", &config).await {
        Ok(text) => Ok(ModelTestResult {
            success: true,
            output: format!("OK: \"{}\"", text.trim()),
            duration_ms: 0,
        }),
        Err(e) => Ok(ModelTestResult {
            success: false,
            output: format!("Inference error: {}", e),
            duration_ms: 0,
        }),
    }
}
