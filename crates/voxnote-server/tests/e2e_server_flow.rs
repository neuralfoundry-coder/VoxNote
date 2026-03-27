//! E2E: 서버 전체 API 플로우 테스트
//!
//! 실제 앱 사용 시나리오:
//!   로그인 → 라이선스 확인 → 모델 카탈로그 → 프로필 조회

use axum::body::Body;
use axum::http::{Request, StatusCode};
use tower::ServiceExt;

fn app() -> axum::Router {
    use axum::routing::get;
    axum::Router::new()
        .route("/health", get(|| async {
            axum::Json(serde_json::json!({"status": "ok"}))
        }))
        .nest("/api/v1/auth", voxnote_server::routes::auth::router())
        .nest("/api/v1/license", voxnote_server::routes::license::router())
        .nest("/api/v1/models", voxnote_server::routes::models::router())
        .nest("/api/v1/user", voxnote_server::routes::users::router())
}

// ── E2E-401: 전체 인증 플로우 ───────────────────────────────────

#[tokio::test]
async fn e2e_401_full_auth_flow() {
    let app = app();

    // 1. 로그인
    let login_body = serde_json::json!({
        "provider": "google",
        "id_token": "valid-google-token"
    });
    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&login_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    let body = axum::body::to_bytes(resp.into_body(), usize::MAX).await.unwrap();
    let auth: serde_json::Value = serde_json::from_slice(&body).unwrap();
    let access_token = auth["access_token"].as_str().unwrap();
    let refresh_token = auth["refresh_token"].as_str().unwrap();
    assert!(!access_token.is_empty());
    assert!(!refresh_token.is_empty());

    // 2. 토큰 리프레시
    let refresh_body = serde_json::json!({ "refresh_token": refresh_token });
    let resp = app.clone()
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/refresh")
                .header("content-type", "application/json")
                .body(Body::from(serde_json::to_string(&refresh_body).unwrap()))
                .unwrap(),
        )
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::OK);

    // 3. 로그아웃
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

// ── E2E-402: 앱 시작 시 서버 통신 시뮬레이션 ───────────────────

#[tokio::test]
async fn e2e_402_app_startup_server_calls() {
    let app = app();

    // 앱 시작 시 순서: health → license verify → models catalog → user profile
    let endpoints = vec![
        ("GET", "/health"),
        ("GET", "/api/v1/license/verify"),
        ("GET", "/api/v1/models/catalog"),
        ("GET", "/api/v1/user/profile"),
    ];

    for (method, uri) in endpoints {
        let resp = app.clone()
            .oneshot(
                Request::builder()
                    .method(method)
                    .uri(uri)
                    .body(Body::empty())
                    .unwrap(),
            )
            .await
            .unwrap();
        assert_eq!(
            resp.status(),
            StatusCode::OK,
            "Endpoint {} {} should return 200",
            method,
            uri
        );
    }
}

// ── E2E-403: 잘못된 요청 처리 ───────────────────────────────────

#[tokio::test]
async fn e2e_403_invalid_json_body() {
    let app = app();

    let resp = app
        .oneshot(
            Request::builder()
                .method("POST")
                .uri("/api/v1/auth/login")
                .header("content-type", "application/json")
                .body(Body::from("invalid json {{{"))
                .unwrap(),
        )
        .await
        .unwrap();

    // 잘못된 JSON → 422 또는 400
    assert!(
        resp.status() == StatusCode::BAD_REQUEST
            || resp.status() == StatusCode::UNPROCESSABLE_ENTITY,
        "Invalid JSON should return 400 or 422, got {}",
        resp.status()
    );
}

// ── E2E-404: 존재하지 않는 엔드포인트 ──────────────────────────

#[tokio::test]
async fn e2e_404_not_found() {
    let app = app();
    let resp = app
        .oneshot(Request::builder().uri("/api/v1/nonexistent").body(Body::empty()).unwrap())
        .await
        .unwrap();
    assert_eq!(resp.status(), StatusCode::NOT_FOUND);
}
