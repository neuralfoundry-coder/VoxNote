use axum::{routing::post, Json, Router};
use serde::{Deserialize, Serialize};

pub fn router() -> Router {
    Router::new()
        .route("/login", post(login))
        .route("/refresh", post(refresh))
        .route("/logout", post(logout))
}

#[derive(Deserialize)]
pub struct LoginRequest {
    pub provider: String,    // "google" | "apple"
    pub id_token: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub access_token: String,
    pub refresh_token: String,
    pub expires_in: u64,
}

async fn login(Json(req): Json<LoginRequest>) -> Json<AuthResponse> {
    // OAuth2 OIDC 토큰 검증 → JWT 발급
    // 1. Provider의 JWKS로 id_token 검증
    // 2. 사용자 생성 또는 조회
    // 3. JWT RS256 access_token + refresh_token 발급
    Json(AuthResponse {
        access_token: format!("at_{}", req.provider),
        refresh_token: format!("rt_{}", req.provider),
        expires_in: 900, // 15분
    })
}

#[derive(Deserialize)]
pub struct RefreshRequest {
    pub refresh_token: String,
}

async fn refresh(Json(req): Json<RefreshRequest>) -> Json<AuthResponse> {
    // refresh_token 검증 → 새 JWT 발급
    Json(AuthResponse {
        access_token: "at_refreshed".to_string(),
        refresh_token: "rt_refreshed".to_string(),
        expires_in: 900,
    })
}

async fn logout() -> Json<serde_json::Value> {
    // 세션 무효화
    Json(serde_json::json!({ "status": "logged_out" }))
}
