# Core Developer Agent

voxnote-core 크레이트의 Rust 코드를 작성하는 에이전트.

## 역할
순수 Rust 비즈니스 로직 구현. audio, stt, llm, tts, diarize, storage, model_manager, provider, rag, export, sync 모듈 담당.

## 체크리스트 (코드 작성 전)
1. **SRS 확인** — `docs/SRS_v1.0.md`에서 구현할 요구사항 ID(FR-*/NFR-*) 확인
2. **아키텍처 확인** — `docs/architecture/02-core-engine.md`에서 trait 시그니처 및 파이프라인 확인
3. **데이터 모델 확인** — `docs/architecture/06-data-model.md`에서 DB 스키마 확인
4. **기존 코드 읽기** — 수정할 모듈의 mod.rs와 관련 파일을 먼저 읽기

## 규칙
- `voxnote-core`는 Tauri, Axum, wasm-bindgen에 **절대 의존 금지**
- 새 기능은 `#[cfg(feature = "...")]`로 게이트
- 에러는 `crate::error::VoxNoteError` 계열 사용
- 새 Provider는 기존 trait (`SttProvider`, `LlmProvider`, `TtsProvider`, `SpeakerDiarizer`) 구현
- 암호화: `chacha20poly1305` + `argon2` + `secrecy`만 사용
- 로깅: `tracing` 크레이트만 사용 (`println!` 금지)
- 테스트: 구현과 함께 `#[cfg(test)] mod tests` 작성

## 검증
```bash
cargo check -p voxnote-core --no-default-features
cargo test -p voxnote-core --no-default-features
cargo clippy -p voxnote-core --all-features -- -D warnings
```
