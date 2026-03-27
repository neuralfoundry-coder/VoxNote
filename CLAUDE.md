# VoxNote — Claude Code 프로젝트 지침

## 프로젝트 개요

VoxNote는 Privacy-First 로컬 AI 회의록 서비스이다. 음성→전사→요약→저장을 로컬 디바이스에서 수행하며, 선택적으로 클라우드 Provider와 E2EE 동기화를 지원한다.

## 기준 문서 (반드시 참조)

코드를 작성하기 전에 해당 영역의 문서를 반드시 읽는다.

| 문서 | 경로 | 용도 |
|------|------|------|
| SRS v1.0 | `docs/SRS_v1.0.md` | 130개 요구사항, 우선순위, Phase, 수용 기준 |
| 시스템 개요 | `docs/architecture/01-system-overview.md` | 전체 아키텍처 |
| 코어 엔진 | `docs/architecture/02-core-engine.md` | trait 시그니처, 파이프라인 |
| 데이터 흐름 | `docs/architecture/03-data-flow.md` | 런타임 데이터 경로 |
| 크로스플랫폼 | `docs/architecture/04-cross-platform.md` | 모바일/WASM 전략 |
| 서버 아키텍처 | `docs/architecture/05-server-architecture.md` | API, 동기화 프로토콜 |
| 데이터 모델 | `docs/architecture/06-data-model.md` | DB 스키마 |
| 보안 아키텍처 | `docs/architecture/07-security-architecture.md` | E2EE, 키체인 |
| 프로젝트 구조 | `docs/architecture/08-project-structure.md` | 디렉토리, feature flags |
| 테스트 계획 | `docs/test/VN-TCP-001.md` | 테스트 전략, 도구 |
| 추적 매트릭스 | `docs/test/traceability-matrix.md` | 요구사항↔테스트 매핑 |
| 테스트 케이스 | `docs/test/test-cases/` | 350+ 테스트 케이스 |

## 설계 원칙 (위반 금지)

1. **LOCAL-FIRST** — AI 추론은 기본적으로 디바이스에서 수행. 클라우드는 opt-in.
2. **ZERO-TRUST** — 서버는 사용자 데이터에 접근 불가. 동기화 시 암호문만 중계.
3. **RUST-NATIVE** — 코어 로직 100% Rust. unsafe 최소화.
4. **PROVIDER-AGNOSTIC** — 로컬 모델과 클라우드 API를 동일 trait으로 처리.
5. **PRIVACY-HARD** — 프라이버시를 기술적으로 보장 (암호화, 네트워크 격리).

## 아키텍처 규칙

### 의존성 방향 (절대 역방향 금지)
```
voxnote-core (순수 Rust, 프레임워크 무관)
    ↑
    ├── voxnote-tauri (Tauri 의존)
    ├── voxnote-server (Axum 의존)
    └── voxnote-wasm (wasm-bindgen 의존)
```

- `voxnote-core`는 Tauri, Axum, wasm-bindgen에 **절대 의존하지 않는다**
- 모든 플랫폼 크레이트가 `core`에 의존하며, 역방향 의존은 금지

### Feature Flags
```toml
default = ["stt", "desktop"]
# AI: stt, llm, tts, diarize
# GPU: metal, cuda, vulkan
# Platform: desktop, mobile, wasm
# Network: sync, cloud-providers
```
- 선택적 기능은 반드시 `#[cfg(feature = "...")]`로 게이트
- 새 의존성 추가 시 optional 여부 확인

### 에러 처리
- `thiserror` 기반 `VoxNoteError` 통합 에러 타입 사용 (`crates/voxnote-core/src/error.rs`)
- `Result<T, VoxNoteError>` 반환. `Box<dyn Error>` 금지
- Tauri IPC 커맨드는 `Result<T, String>` 반환 (Tauri 규약)

### Provider Trait 패턴
새 Provider 추가 시 기존 trait을 구현:
- STT: `SttProvider` (`stt/mod.rs`)
- LLM: `LlmProvider` (`llm/mod.rs`)
- TTS: `TtsProvider` (`tts/mod.rs`)
- Diarize: `SpeakerDiarizer` (`diarize/mod.rs`)

## 보안 규칙 (위반 시 즉시 수정)

- **평문 저장 금지** — DB 데이터는 ChaCha20-Poly1305로 암호화
- **API 키 파일 저장 금지** — OS 키체인(macOS Keychain, Windows DPAPI, Linux Secret Service) 사용
- **API 키 메모리 보호** — `secrecy::Secret<String>` + `zeroize` on drop
- **TLS 1.3 강제** — `rustls`만 사용, OpenSSL 금지
- **로컬 모드 네트워크 격리** — 라이선스 검증 외 모든 네트워크 호출 차단
- **외부 API 사용 시 고지** — 사용자에게 전송 데이터 범위 명시

## 코딩 컨벤션

### Rust
- Edition 2021, MSRV 1.80
- `cargo clippy --all-features -- -D warnings` 통과 필수
- `cargo fmt --check` 통과 필수
- 비동기: `tokio` 런타임, `async_trait` 매크로
- 로깅: `tracing` 크레이트 (`info!`, `debug!`, `warn!`, `error!`)
- 직렬화: `serde` + `serde_json` + `toml`

### TypeScript/React
- React 19 + TypeScript 5, Vite 빌드
- 상태 관리: Zustand (`stores/` 디렉토리)
- IPC: `hooks/useTauriIPC.ts`의 `tauriInvoke<T>()` 래퍼 사용
- 스타일: TailwindCSS v4

### 네이밍
- 요구사항 ID: `FR-{CAT}-{NUM}` / `NFR-{CAT}-{NUM}`
- 테스트 ID: `TC-{CAT}-{REQ}-{SEQ}` (예: `TC-AUD-001-01`)
- Rust 테스트: `fn test_<component>_<scenario>_<expected>()`
- TS 테스트: `it('should <expected behavior>')`

## 테스트 규칙

### 커버리지 목표
- P0 요구사항: 100% 자동화
- P1 요구사항: 80%+ 자동화
- Rust 코드: ≥80% 라인 커버리지
- TS 코드: ≥75% 라인 커버리지

### 테스트 위치
- Rust 단위: 소스 파일 내 `#[cfg(test)] mod tests`
- Rust 통합: `tests/` 디렉토리
- TS 단위: `.test.tsx` / `.test.ts` (소스 파일 옆)
- E2E: `tests/e2e/` (Playwright)
- 벤치마크: `benches/` (Criterion)
- 퍼징: `fuzz/fuzz_targets/`

### 테스트 도구
- Rust: `cargo nextest`, `cargo tarpaulin` (커버리지)
- TS: `vitest`, `@testing-library/react`
- E2E: Playwright + tauri-driver
- 성능: Criterion
- 보안: `cargo-fuzz`, `cargo audit`, `cargo deny`

## DB 스키마 규칙

- **전체 9 테이블은 migration v1에서 생성 완료** (변경 시 새 migration 추가)
- 테이블: folders, notes, transcripts, transcript_fts, summaries, embeddings, vocabulary, provider_config, templates, speaker_profiles
- FTS5 트리거 자동 동기화 (INSERT/UPDATE/DELETE)
- 스키마 변경 시 `storage/migration.rs`에 새 버전 함수 추가

## 빌드 명령

```bash
# 개발 (macOS Apple Silicon)
cargo build -p voxnote-core --features stt,desktop,metal

# 테스트
cargo test -p voxnote-core --no-default-features
cargo test -p voxnote-server

# 프론트엔드
cd frontend && pnpm dev          # 개발 서버 (:1420)
cd frontend && pnpm test         # vitest
cd frontend && npx tsc --noEmit  # 타입 체크

# 서버
cargo run -p voxnote-server      # :8080

# Lint
cargo clippy --workspace --all-features -- -D warnings
cargo fmt --check
cargo deny check
```

## 우선순위 정의

- **P0** (38개) — MVP 필수. 100% 자동화 테스트. 매 빌드 회귀.
- **P1** (39개) — 초기 릴리즈. 80%+ 자동화. PR별 회귀.
- **P2** (16개) — 후속 개선. 50%+ 자동화. 릴리즈 전 테스트.
