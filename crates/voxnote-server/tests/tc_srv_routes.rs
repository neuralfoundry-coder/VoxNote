//! TC-SRV: 서버 라우트 통합 테스트
//! 관련 요구사항: FR-SRV-001 ~ FR-SRV-008

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

fn app() -> axum::Router {
    use axum::routing::get;
    axum::Router::new()
        .route("/health", get(health_handler))
        .nest("/api/v1/auth", voxnote_server::routes::auth::router())
        .nest("/api/v1/license", voxnote_server::routes::license::router())
        .nest("/api/v1/models", voxnote_server::routes::models::router())
        .nest("/api/v1/user", voxnote_server::routes::users::router())
}

async fn health_handler() -> axum::Json<serde_json::Value> {
    axum::Json(serde_json::json!({"status": "ok"}))
}

// ── Health Check ────────────────────────────────────────────────

#[tokio::test]
async fn tc_srv_health_check() {
    let app = app();
    let resp = app
        .oneshot(Request::builder().uri("/health").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── Auth Routes ─────────────────────────────────────────────────

#[tokio::test]
async fn tc_srv_auth_login() {
    let app = app();
    let body = serde_json::json!({
        "provider": "google",
        "id_token": "test-token"
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["access_token"].is_string());
    assert!(json["refresh_token"].is_string());
    assert!(json["expires_in"].is_number());
}

#[tokio::test]
async fn tc_srv_auth_refresh() {
    let app = app();
    let body = serde_json::json!({ "refresh_token": "rt_test" });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

#[tokio::test]
async fn tc_srv_auth_logout() {
    let app = app();
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/logout")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── License Routes ──────────────────────────────────────────────

#[tokio::test]
async fn tc_srv_license_verify() {
    let app = app();
    let resp = app
        .oneshot(Request::builder().uri("/api/v1/license/verify").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["valid"], true);
}

#[tokio::test]
async fn tc_srv_license_activate() {
    let app = app();
    let body = serde_json::json!({
        "license_key": "VN-TEST-KEY",
        "device_name": "MacBook Pro"
    });
    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/license/activate")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);
}

// ── Models Routes ───────────────────────────────────────────────

#[tokio::test]
async fn tc_srv_models_catalog() {
    let app = app();
    let resp = app
        .oneshot(Request::builder().uri("/api/v1/models/catalog").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json.is_array());
}

#[tokio::test]
async fn tc_srv_models_download_url() {
    let app = app();
    let resp = app
        .oneshot(
            Request::builder()
                .uri("/api/v1/models/whisper-tiny/download")
                .body(Body::empty())
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert_eq!(json["model_id"], "whisper-tiny");
}

// ── User Routes ─────────────────────────────────────────────────

#[tokio::test]
async fn tc_srv_user_profile() {
    let app = app();
    let resp = app
        .oneshot(Request::builder().uri("/api/v1/user/profile").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let json: serde_json::Value = serde_json::from_slice(&body).unwrap();
    assert!(json["email"].is_string());
}
