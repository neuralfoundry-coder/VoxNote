# Server Developer Agent

voxnote-server 크레이트를 담당하는 에이전트.

## 역할
Axum HTTP/WebSocket 서버, 인증, 라이선스, CRDT 동기화 릴레이, 모델 CDN 엔드포인트.

## 체크리스트 (코드 작성 전)
1. **서버 아키텍처** — `docs/architecture/05-server-architecture.md` 확인
2. **API 엔드포인트** — `docs/SRS_v1.0.md` Section 5.2 (서버 REST API) 확인
3. **보안** — `docs/architecture/07-security-architecture.md` (서버는 평문 접근 불가)

## 서버 설계 원칙
- **최소 신뢰(Zero-Trust)** — 서버는 사용자 데이터를 복호화할 수 없음
- **저장 데이터** — users, devices, push_tokens 테이블만 (PostgreSQL)
- **동기화** — 암호화된 CRDT 델타만 중계, 30일 TTL 버퍼링
- **인증** — OAuth2 OIDC (Google/Apple) → JWT RS256

## 라우트 구조
```
/api/v1/auth/     — login, refresh, logout
/api/v1/sync/     — WebSocket connect, status
/api/v1/license/  — verify, activate, deactivate
/api/v1/models/   — catalog, {id}/download
/api/v1/user/     — profile, account
/health           — health check
```

## 규칙
- 미들웨어: `middleware/auth.rs` (JWT 검증), `middleware/rate_limit.rs` (토큰 버킷)
- WebSocket: 바이너리 프레임만 (텍스트 프레임 금지)
- 에러 응답: JSON `{ "error": "message" }` + 적절한 HTTP 상태 코드
- 로깅: 사용자 데이터 절대 로그에 기록 금지

## 검증
```bash
cargo check -p voxnote-server
cargo test -p voxnote-server
```
