# TC-NFR-MAINT: 유지보수성 비기능 요구사항 테스트 케이스

| 항목 | 내용 |
|------|------|
| **문서 ID** | TC-NFR-MAINT |
| **SRS 참조** | VoxNote SRS v1.0 - NFR-MAINT (NFR-MAINT-001 ~ NFR-MAINT-004) |
| **작성일** | 2026-03-27 |
| **상태** | 초안 |
| **테스트 코드 위치** | `tests/nfr/maint/`, `.github/workflows/` |

## 테스트 요약

| 테스트 유형 | 개수 |
|-------------|------|
| 구조 검증 (정적 분석) | 5 |
| CI 빌드 검증 | 3 |
| Integration | 2 |
| 코드 리뷰 | 2 |
| **합계** | **12** |

---

## NFR-MAINT-001: Cargo 워크스페이스 분리 - P1

### TC-NFR-MAINT-001-01: 워크스페이스 구조 및 크레이트 분리 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-MAINT-001-01 |
| **요구사항 ID** | NFR-MAINT-001 |
| **테스트 유형** | 구조 검증 (정적 분석) |
| **사전 조건** | - 프로젝트 소스 코드 접근 가능<br>- `cargo-workspaces` 도구 설치 |
| **테스트 절차** | 1. `Cargo.toml` (루트) 워크스페이스 멤버 목록 확인<br>2. 각 크레이트의 책임 영역 확인:<br>  - `voxnote-core`: 핵심 비즈니스 로직<br>  - `voxnote-stt`: 음성-텍스트 변환<br>  - `voxnote-llm`: LLM 요약/분석<br>  - `voxnote-ui`: 사용자 인터페이스<br>  - `voxnote-sync`: 동기화 엔진<br>  - `voxnote-crypto`: 암호화 모듈<br>3. 크레이트 간 순환 의존성 검사<br>4. 각 크레이트 독립 빌드 가능 여부 확인 |
| **기대 결과** | - 최소 5개 이상의 워크스페이스 크레이트 존재<br>- 각 크레이트의 단일 책임 원칙 준수<br>- 순환 의존성 0건<br>- 각 크레이트 독립 빌드 성공 (`cargo build -p <crate>`) |
| **테스트 코드 위치** | `tests/nfr/maint/workspace_structure.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | 전체 |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-MAINT-001-02: 크레이트 간 의존성 그래프 건전성

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-MAINT-001-02 |
| **요구사항 ID** | NFR-MAINT-001 |
| **테스트 유형** | 구조 검증 (정적 분석) |
| **사전 조건** | - `cargo-depgraph` 또는 `cargo-tree` 설치 |
| **테스트 절차** | 1. `cargo tree --workspace` 의존성 트리 생성<br>2. `cargo depgraph` 의존성 그래프 시각화<br>3. UI 크레이트 → Core 방향 의존성 확인 (역방향 없음)<br>4. Core 크레이트가 UI 의존하지 않음 확인<br>5. 외부 의존성 버전 통일성 확인 (워크스페이스 상속) |
| **기대 결과** | - 의존성 방향: UI → Core → Library (단방향)<br>- Core가 UI에 의존하지 않음<br>- 외부 크레이트 버전 워크스페이스 수준에서 통합 관리<br>- 불필요한 중복 의존성 없음 |
| **테스트 코드 위치** | `tests/nfr/maint/dependency_graph.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | 전체 |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-MAINT-001-03: 개별 크레이트 테스트 독립 실행

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-MAINT-001-03 |
| **요구사항 ID** | NFR-MAINT-001 |
| **테스트 유형** | CI 빌드 검증 |
| **사전 조건** | - CI 환경 준비 |
| **테스트 절차** | 1. 각 크레이트별 독립 테스트 실행:<br>  `cargo test -p voxnote-core`<br>  `cargo test -p voxnote-stt`<br>  `cargo test -p voxnote-llm`<br>  (각 크레이트 반복)<br>2. 전체 워크스페이스 테스트: `cargo test --workspace`<br>3. 독립 실행 결과와 워크스페이스 결과 비교 |
| **기대 결과** | - 모든 크레이트 독립 테스트 통과<br>- 워크스페이스 테스트와 독립 테스트 결과 동일<br>- 크레이트 간 테스트 격리 확인 (부수 효과 없음) |
| **테스트 코드 위치** | `.github/workflows/ci.yml` (per_crate_test 스텝) |
| **자동화 여부** | 자동 (CI) |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-MAINT-002: feature flag 조건부 컴파일 - P1

### TC-NFR-MAINT-002-01: feature flag 조합별 빌드 성공 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-MAINT-002-01 |
| **요구사항 ID** | NFR-MAINT-002 |
| **테스트 유형** | CI 빌드 검증 |
| **사전 조건** | - 정의된 feature flag 목록 확인<br>  (예: `gpu`, `local-only`, `sync`, `tts`, `cloud-api`) |
| **테스트 절차** | 1. 기본 feature set 빌드: `cargo build --release`<br>2. 최소 feature 빌드: `cargo build --release --no-default-features`<br>3. 개별 feature 활성화 빌드:<br>  `cargo build --release --features gpu`<br>  `cargo build --release --features local-only`<br>  `cargo build --release --features sync`<br>4. 조합 빌드: `cargo build --release --features "gpu,sync"`<br>5. 상충 feature 검증: `gpu` + `cpu-only` 동시 활성화 에러 확인 |
| **기대 결과** | - 모든 유효한 feature 조합 빌드 성공<br>- `--no-default-features` 빌드 성공 (최소 기능)<br>- 상충 feature 조합 시 명확한 컴파일 에러<br>- 각 조합별 바이너리 크기 기록 |
| **테스트 코드 위치** | `.github/workflows/ci.yml` (feature_matrix 스텝) |
| **자동화 여부** | 자동 (CI) |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-MAINT-002-02: feature flag에 따른 코드 경로 격리 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-MAINT-002-02 |
| **요구사항 ID** | NFR-MAINT-002 |
| **테스트 유형** | 구조 검증 (정적 분석) |
| **사전 조건** | - 소스 코드 접근 가능<br>- `cfg` 어트리뷰트 사용 현황 파악 |
| **테스트 절차** | 1. `#[cfg(feature = "...")]` 사용처 전수 조사<br>2. `local-only` feature 시 네트워크 코드 제외 확인<br>3. `gpu` feature 미활성 시 GPU 관련 코드 제외 확인<br>4. feature 간 코드 중복 최소화 확인<br>5. 미사용 코드(dead code) 경고 없음 확인 (각 feature 조합에서) |
| **기대 결과** | - `local-only`: 네트워크/API 코드 완전 제외<br>- `gpu` 미활성: GPU 바인딩 코드 제외<br>- 각 feature 조합에서 dead code 경고 0건<br>- feature 분기 지점 문서화 확인 |
| **테스트 코드 위치** | `tests/nfr/maint/feature_flag_isolation.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | 전체 |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-MAINT-002-03: feature flag별 테스트 스위트 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-MAINT-002-03 |
| **요구사항 ID** | NFR-MAINT-002 |
| **테스트 유형** | Integration |
| **사전 조건** | - 각 feature에 대한 테스트 코드 존재 확인 |
| **테스트 절차** | 1. `cargo test --no-default-features` 실행<br>2. `cargo test --features gpu` 실행<br>3. `cargo test --features local-only` 실행<br>4. 각 feature 조합에서 활성화된 테스트 수 비교<br>5. feature 전용 테스트에 `#[cfg(feature = "...")]` 적용 확인 |
| **기대 결과** | - 각 feature 조합에서 관련 테스트만 실행<br>- GPU 미활성 시 GPU 테스트 스킵<br>- local-only 시 네트워크 테스트 스킵<br>- 테스트 커버리지 > 70% (각 feature 조합) |
| **테스트 코드 위치** | `tests/nfr/maint/feature_flag_tests.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-MAINT-003: #[cfg] 플랫폼별 코드 분리 - P1

### TC-NFR-MAINT-003-01: 플랫폼별 조건부 컴파일 정합성

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-MAINT-003-01 |
| **요구사항 ID** | NFR-MAINT-003 |
| **테스트 유형** | 구조 검증 (정적 분석) |
| **사전 조건** | - 소스 코드 접근 가능<br>- 크로스 컴파일 타겟 설치 |
| **테스트 절차** | 1. `#[cfg(target_os = "...")]` 사용처 전수 조사<br>2. 플랫폼별 모듈 구조 확인:<br>  - `src/platform/macos.rs`<br>  - `src/platform/windows.rs`<br>  - `src/platform/linux.rs`<br>  - `src/platform/ios.rs`<br>  - `src/platform/android.rs`<br>3. 공통 trait 기반 추상화 확인<br>4. 플랫폼 전용 코드가 플랫폼 모듈 내에만 존재하는지 확인 |
| **기대 결과** | - 플랫폼별 코드가 전용 모듈에 격리<br>- 공통 trait (`PlatformKeychain`, `PlatformAudio` 등) 정의<br>- 핵심 로직에 `#[cfg]` 분기 최소화<br>- 새 플랫폼 추가 시 trait 구현만 필요 |
| **테스트 코드 위치** | `tests/nfr/maint/platform_cfg_structure.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | 전체 |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-MAINT-003-02: 모든 타겟 플랫폼 크로스 컴파일 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-MAINT-003-02 |
| **요구사항 ID** | NFR-MAINT-003 |
| **테스트 유형** | CI 빌드 검증 |
| **사전 조건** | - 크로스 컴파일 타겟 설치<br>  (`rustup target add` 각 플랫폼) |
| **테스트 절차** | 1. `cargo check --target aarch64-apple-darwin` (macOS ARM)<br>2. `cargo check --target x86_64-pc-windows-msvc` (Windows)<br>3. `cargo check --target x86_64-unknown-linux-gnu` (Linux)<br>4. `cargo check --target aarch64-apple-ios` (iOS)<br>5. `cargo check --target aarch64-linux-android` (Android) |
| **기대 결과** | - 모든 타겟 플랫폼 컴파일 체크 통과<br>- 플랫폼별 조건부 컴파일 에러 0건<br>- 누락된 플랫폼 구현 경고 0건 |
| **테스트 코드 위치** | `.github/workflows/ci-matrix.yml` (cross_check 스텝) |
| **자동화 여부** | 자동 (CI) |
| **플랫폼** | 전체 (크로스 컴파일) |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-MAINT-003-03: 플랫폼 추상화 인터페이스 완전성

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-MAINT-003-03 |
| **요구사항 ID** | NFR-MAINT-003 |
| **테스트 유형** | 코드 리뷰 |
| **사전 조건** | - 플랫폼 추상화 trait 정의 확인 |
| **테스트 절차** | 1. 플랫폼 추상화 trait 목록 확인<br>2. 각 trait의 모든 플랫폼 구현체 존재 확인<br>3. 구현 누락 시 컴파일 에러 발생 확인<br>4. mock 구현체 존재 여부 확인 (테스트용)<br>5. 플랫폼 감지 로직 정확성 확인 |
| **기대 결과** | - 모든 trait에 대해 5개 플랫폼 구현체 존재<br>- 구현 누락 시 `compile_error!` 매크로 발동<br>- 테스트용 mock 구현체 존재<br>- 런타임 플랫폼 감지 정확 |
| **테스트 코드 위치** | `tests/nfr/maint/platform_trait_completeness.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | 전체 |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-MAINT-004: CI/CD 매트릭스 빌드 - P1

### TC-NFR-MAINT-004-01: CI 매트릭스 전체 조합 빌드 성공

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-MAINT-004-01 |
| **요구사항 ID** | NFR-MAINT-004 |
| **테스트 유형** | CI 빌드 검증 |
| **사전 조건** | - GitHub Actions 워크플로 정의 완료<br>- 매트릭스: OS × Rust 버전 × feature set |
| **테스트 절차** | 1. CI 매트릭스 정의 확인:<br>  OS: [macos-14, windows-latest, ubuntu-22.04]<br>  Rust: [stable, nightly]<br>  Features: [default, minimal, full]<br>2. 모든 조합(3×2×3=18)에서 빌드 실행<br>3. 모든 조합에서 테스트 실행<br>4. 실패 조합 식별 및 원인 분석<br>5. 빌드 시간 매트릭스 기록 |
| **기대 결과** | - stable 빌드: 18개 조합 100% 성공<br>- nightly 빌드: 허용 실패 (allow-failure) 설정 확인<br>- 빌드 시간 < 15분 (각 조합)<br>- 테스트 통과율 100% (stable) |
| **테스트 코드 위치** | `.github/workflows/ci-matrix.yml` |
| **자동화 여부** | 자동 (CI) |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-MAINT-004-02: CI 파이프라인 자동화 완전성

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-MAINT-004-02 |
| **요구사항 ID** | NFR-MAINT-004 |
| **테스트 유형** | 코드 리뷰 |
| **사전 조건** | - CI/CD 워크플로 파일 접근 가능 |
| **테스트 절차** | 1. CI 파이프라인 단계 확인:<br>  - 린트 (clippy, fmt)<br>  - 빌드<br>  - 단위 테스트<br>  - 통합 테스트<br>  - 벤치마크<br>  - 보안 감사 (cargo-audit)<br>  - 바이너리 크기 검사<br>2. PR 생성 시 자동 실행 확인<br>3. main 브랜치 머지 시 릴리스 빌드 확인<br>4. 아티팩트 생성 및 보관 확인 |
| **기대 결과** | - 위 모든 단계가 CI에 포함<br>- PR에서 필수 체크 통과 필요 (branch protection)<br>- main 머지 시 릴리스 아티팩트 자동 생성<br>- 실패 시 Slack/이메일 알림 |
| **테스트 코드 위치** | `.github/workflows/` (모든 워크플로 파일) |
| **자동화 여부** | 자동 (CI 자체가 테스트 대상) |
| **플랫폼** | 전체 |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-MAINT-004-03: CD 릴리스 자동화 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-MAINT-004-03 |
| **요구사항 ID** | NFR-MAINT-004 |
| **테스트 유형** | Integration |
| **사전 조건** | - 릴리스 워크플로 정의 완료<br>- 테스트 태그 생성 가능 |
| **테스트 절차** | 1. 테스트 태그 `v0.0.0-test` 생성<br>2. 릴리스 워크플로 자동 트리거 확인<br>3. 각 플랫폼별 릴리스 아티팩트 생성 확인:<br>  - macOS: `.dmg` (Universal)<br>  - Windows: `.msi` / `.exe` 인스톨러<br>  - Linux: `.AppImage` / `.deb`<br>4. GitHub Release 자동 생성 확인<br>5. 변경 로그 자동 생성 확인 |
| **기대 결과** | - 태그 푸시 시 자동 릴리스 빌드 시작<br>- 3개 플랫폼 아티팩트 모두 생성<br>- GitHub Release에 아티팩트 첨부<br>- 변경 로그 자동 생성 (conventional commits)<br>- 릴리스 전체 시간 < 30분 |
| **테스트 코드 위치** | `.github/workflows/release.yml` |
| **자동화 여부** | 자동 (CD) |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |
