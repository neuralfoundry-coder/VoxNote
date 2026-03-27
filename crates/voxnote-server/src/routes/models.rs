use axum::{extract::Path, routing::get, Json, Router};
use serde::Serialize;

pub fn router() -> Router {
    Router::new()
        .route("/catalog", get(catalog))
        .route("/{id}/download", get(download_url))
}

#[derive(Serialize)]
pub struct ModelCatalogEntry {
    pub id: String,
    pub name: String,
    pub model_type: String,
    pub size_bytes: u64,
    pub download_url: String,
}

async fn catalog() -> Json<Vec<ModelCatalogEntry>> {
    // 프로덕션에서는 DB 또는 registry에서 조회
    Json(vec![
        ModelCatalogEntry {
            id: "whisper-tiny-q8".to_string(),
            name: "Whisper Tiny (Q8_0)".to_string(),
            model_type: "stt".to_string(),
            size_bytes: 78000000,
            download_url: "https://cdn.voxnote.app/models/whisper-tiny-q8.bin".to_string(),
        },
    ])
}

async fn download_url(Path(id): Path<String>) -> Json<serde_json::Value> {
    // Cloudflare R2 Signed URL 생성
    Json(serde_json::json!({
        "model_id": id,
        "url": format!("https://cdn.voxnote.app/models/{}", id),
        "expires_in": 3600
    }))
}
