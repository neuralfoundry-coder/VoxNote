use axum::{routing::{get, post, delete}, Json, Router};
use serde::{Deserialize, Serialize};

pub fn router() -> Router {
    Router::new()
        .route("/verify", get(verify))
        .route("/activate", post(activate))
        .route("/deactivate", delete(deactivate))
}

#[derive(Serialize)]
pub struct LicenseStatus {
    pub valid: bool,
    pub license_type: String,
    pub max_devices: u32,
    pub active_devices: u32,
}

async fn verify() -> Json<LicenseStatus> {
    Json(LicenseStatus {
        valid: true,
        license_type: "personal".to_string(),
        max_devices: 3,
        active_devices: 1,
    })
}

#[derive(Deserialize)]
pub struct ActivateRequest {
    pub license_key: String,
    pub device_name: String,
}

async fn activate(Json(req): Json<ActivateRequest>) -> Json<serde_json::Value> {
    Json(serde_json::json!({
        "activated": true,
        "device_name": req.device_name
    }))
}

async fn deactivate() -> Json<serde_json::Value> {
    Json(serde_json::json!({ "deactivated": true }))
}
