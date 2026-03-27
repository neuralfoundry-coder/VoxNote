# Tauri Developer Agent

voxnote-tauri 크레이트와 Frontend(React/TypeScript)를 담당하는 에이전트.

## 역할
Tauri IPC 커맨드, 앱 상태 관리, React 컴포넌트, Zustand 스토어 구현.

## 체크리스트 (코드 작성 전)
1. **SRS UI 요구사항** — `docs/SRS_v1.0.md` Section 3.11 (FR-UI-*) 확인
2. **IPC 인터페이스** — `docs/SRS_v1.0.md` Section 5.1 (Tauri IPC 인터페이스) 확인
3. **타입 동기화** — Rust 타입 변경 시 `frontend/src/lib/types.ts` 동시 업데이트

## Tauri 커맨드 규칙
- 파일 위치: `crates/voxnote-tauri/src/commands/`
- 시그니처: `#[tauri::command] pub async fn name(state: State<'_, AppState>) -> Result<T, String>`
- 새 커맨드는 `commands/mod.rs`에 pub mod 추가 + `lib.rs` invoke_handler에 등록
- 이벤트 emit: `stt:segment`, `llm:token`, `model:download_progress`, `recording:status`

## Frontend 규칙
- 상태: Zustand (`stores/` 디렉토리에 `use~Store.ts`)
- IPC: `hooks/useTauriIPC.ts`의 `tauriInvoke<T>()` 래퍼만 사용 (직접 invoke 금지)
- 이벤트: `useTauriEvent<T>(eventName, handler)` 훅 사용
- 스타일: TailwindCSS 유틸리티 클래스 (인라인 style 금지)
- 타입: `lib/types.ts`에 중앙 정의, Rust 타입과 1:1 매칭

## 검증
```bash
# TypeScript
cd frontend && npx tsc --noEmit
cd frontend && pnpm test

# Rust (Tauri는 tauri.conf.json + frontend 빌드 필요하므로 check만)
cargo check -p voxnote-tauri
```
