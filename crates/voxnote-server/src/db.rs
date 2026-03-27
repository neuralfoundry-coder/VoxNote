/// PostgreSQL 데이터베이스 연결 (Phase 3)
///
/// 서버는 최소 데이터만 저장합니다:
/// - users (id, email, nickname, avatar_url)
/// - devices (user_id, device_name, license_key, activated_at)
/// - push_tokens (device_id, platform, token)
///
/// 노트, 전사, 요약 등 사용자 데이터는 서버에 저장하지 않습니다.

pub struct Database {
    // pool: sqlx::PgPool, — Phase 3 실구현 시 연결
}

impl Database {
    pub async fn connect(_database_url: &str) -> Result<Self, String> {
        // sqlx::PgPool::connect(database_url).await
        Ok(Self {})
    }
}
