use axum::{
    extract::Request,
    http::StatusCode,
    middleware::Next,
    response::Response,
};

/// JWT RS256 인증 미들웨어
pub async fn jwt_auth(request: Request, next: Next) -> Result<Response, StatusCode> {
    let auth_header = request
        .headers()
        .get("authorization")
        .and_then(|v| v.to_str().ok());

    match auth_header {
        Some(header) if header.starts_with("Bearer ") => {
            let _token = &header[7..];
            // TODO: JWT RS256 서명 검증
            // 1. 공개키로 서명 검증
            // 2. exp 만료 확인
            // 3. claims를 request extensions에 삽입
            Ok(next.run(request).await)
        }
        _ => Err(StatusCode::UNAUTHORIZED),
    }
}
