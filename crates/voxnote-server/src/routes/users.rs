use axum::{routing::{get, put, delete}, Json, Router};
use serde::{Deserialize, Serialize};

pub fn router() -> Router {
    Router::new()
        .route("/profile", get(get_profile).put(update_profile))
        .route("/account", delete(delete_account))
}

#[derive(Serialize, Deserialize)]
pub struct UserProfile {
    pub id: String,
    pub email: String,
    pub nickname: String,
    pub avatar_url: Option<String>,
}

async fn get_profile() -> Json<UserProfile> {
    Json(UserProfile {
        id: "user-1".to_string(),
        email: "user@example.com".to_string(),
        nickname: "VoxNote User".to_string(),
        avatar_url: None,
    })
}

async fn update_profile(Json(profile): Json<UserProfile>) -> Json<UserProfile> {
    Json(profile)
}

async fn delete_account() -> Json<serde_json::Value> {
    // 30일 유예기간 후 삭제
    Json(serde_json::json!({
        "status": "scheduled_for_deletion",
        "grace_period_days": 30
    }))
}
