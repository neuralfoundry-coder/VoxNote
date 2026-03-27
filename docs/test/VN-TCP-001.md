# VoxNote 마스터 테스트 계획서 (Master Test Plan)

**문서 ID:** VN-TCP-001
**버전:** 1.0
**작성일:** 2026-03-27
**상태:** 초안 (Draft)
**작성 기준:** SRS v1.0 (2026-03-27)

---

## 목차

1. [개요](#1-개요)
2. [테스트 전략](#2-테스트-전략)
3. [테스트 환경](#3-테스트-환경)
4. [테스트 케이스 ID 체계](#4-테스트-케이스-id-체계)
5. [테스트 도구 스택](#5-테스트-도구-스택)
6. [테스트 실행 방법](#6-테스트-실행-방법)
7. [리포트 생성 및 관리](#7-리포트-생성-및-관리)
8. [CI/CD 통합](#8-cicd-통합)
9. [테스트 카테고리 목록](#9-테스트-카테고리-목록)
10. [테스트 일정](#10-테스트-일정)
11. [부록](#부록)

---

## 1. 개요

### 1.1 목적

본 문서는 VoxNote 프로젝트의 **소프트웨어 요구사항 명세서(SRS) v1.0**에 정의된 **130개 요구사항**(기능 93개 + 비기능 37개)의 체계적 검증을 위한 마스터 테스트 계획을 수립한다. 모든 요구사항이 설계 의도대로 구현되었음을 보증하고, 품질 기준을 충족하는지 확인하는 것을 목적으로 한다.

### 1.2 범위

| 구분 | 항목 수 | 설명 |
|------|---------|------|
| 기능 테스트 (FR) | 93 | 13개 카테고리별 기능 요구사항 검증 |
| 비기능 테스트 (NFR) | 37 | 7개 카테고리별 성능/보안/호환성 등 검증 |
| UI 테스트 | — | 접근성(a11y), 반응형, 사용성 검증 |

### 1.3 참조 문서

| 문서 | 버전 | 비고 |
|------|------|------|
| VoxNote SRS (SRS_v1.0.md) | 1.0 | 요구사항 정의 기준 |
| 로컬 AI 회의록 서비스 — 아키텍처 설계 및 구현 방법론 | 1.0 | 아키텍처 참조 |
| Tauri 2.0 공식 문서 | — | 플랫폼 제약사항 참조 |
| whisper.cpp / llama.cpp 프로젝트 문서 | — | AI 엔진 동작 참조 |

### 1.4 문서 정보

| 항목 | 값 |
|------|-----|
| 문서 ID | VN-TCP-001 |
| 버전 | 1.0 |
| 표준 | IEEE 829 기반 |
| 승인 상태 | 검토 대기 |

---

## 2. 테스트 전략

### 2.1 테스트 레벨

VoxNote는 Rust + Tauri 2.0 + React/TypeScript 기반 크로스플랫폼 애플리케이션으로, 다음 6개 테스트 레벨을 적용한다.

| 레벨 | 프레임워크 | 대상 | 설명 |
|------|-----------|------|------|
| **단위 테스트 (Rust)** | `#[test]` + cargo-nextest | Rust 코어 로직 | 오디오 파이프라인, STT 바인딩, 스토리지, 암호화, Provider 추상화 등 |
| **단위 테스트 (TS)** | vitest + @testing-library/react | React 컴포넌트, 훅, 유틸 | 전사 뷰, 노트 에디터, Provider 설정 패널 등 |
| **통합 테스트** | cargo-nextest + wiremock-rs | Rust 모듈 간 연동, IPC, API 연동 | Tauri IPC 커맨드, 외부 API mock 통합 |
| **E2E/UI 테스트** | Playwright + tauri-driver | 전체 애플리케이션 흐름 | 녹음~전사~요약~내보내기 시나리오 |
| **성능 테스트** | criterion | Rust 핵심 경로 벤치마크 | 전사 지연, 메모리 사용량, 모델 로딩 시간 |
| **보안 테스트** | cargo-fuzz | 파서, 암호화, 입력 검증 | 퍼징 기반 취약점 탐지 |

### 2.2 우선순위별 전략

SRS에 정의된 우선순위(P0/P1/P2)에 따라 차별화된 테스트 전략을 적용한다.

| 우선순위 | 요구사항 수 | 자동화 목표 | 수동 테스트 | 회귀 테스트 |
|----------|-----------|------------|------------|------------|
| **P0 (필수)** | 38 | **100%** 자동화 | 불허 | 매 빌드 실행 |
| **P1 (중요)** | 39 | **80% 이상** 자동화 | 탐색적 테스트 허용 | PR 단위 실행 |
| **P2 (개선)** | 16 | **50% 이상** 자동화 | 수동 테스트 허용 | 릴리즈 전 실행 |

### 2.3 Phase별 테스트 전략

| Phase | 기간 | 테스트 초점 | 완료 기준 |
|-------|------|-----------|----------|
| Phase 1: Foundation | Month 1~3 | 단위 테스트 + 기본 통합 테스트 | P0 기능 단위 테스트 커버리지 80% 이상 |
| Phase 2: Intelligence | Month 4~6 | AI 엔진 통합 테스트 + Provider API mock 테스트 | LLM/API 통합 시나리오 검증 완료 |
| Phase 3: Platform | Month 7~9 | 크로스플랫폼 E2E + 모바일 호환성 테스트 | 3개 데스크탑 + 2개 모바일 플랫폼 E2E 통과 |
| Phase 4: Polish | Month 10~12 | 전체 회귀 + 성능/보안 테스트 | 전체 TC Pass율 95% 이상, NFR 목표치 달성 |

---

## 3. 테스트 환경

### 3.1 하드웨어

| 환경 | 사양 | 용도 |
|------|------|------|
| macOS (Apple Silicon) | M1+ / 8GB RAM / 256GB SSD | 주 개발 및 테스트 환경, Metal GPU 가속 검증 |
| Windows | 4코어 이상 / 8GB RAM / NVIDIA GTX 1060+ | CUDA 가속, WASAPI Loopback 테스트 |
| Linux (Ubuntu 22.04+) | 4코어 이상 / 8GB RAM | CUDA/Vulkan 가속, PipeWire 테스트 |
| iOS Simulator | Xcode 내장 (iPhone 15 Pro 시뮬레이터) | iOS 기본 기능 검증 |
| Android Emulator | Android Studio (Pixel 7, API 34) | Android 기본 기능 검증 |

### 3.2 소프트웨어

| 구분 | 도구/버전 | 용도 |
|------|----------|------|
| 언어 런타임 | Rust stable (latest), Node.js 20+ | 빌드 및 테스트 실행 |
| 테스트 러너 (Rust) | cargo-nextest | Rust 단위/통합 테스트 실행 |
| 테스트 러너 (TS) | vitest | Frontend 단위 테스트 실행 |
| E2E 프레임워크 | Playwright + tauri-driver | E2E/UI 테스트 |
| 벤치마크 | criterion | Rust 성능 벤치마크 |
| 퍼징 | cargo-fuzz | 보안 퍼징 테스트 |
| 커버리지 | cargo-tarpaulin | Rust 코드 커버리지 측정 |
| API Mock | wiremock-rs | 외부 API(OpenAI, Anthropic 등) mock |
| 접근성 | axe-core (Playwright 통합) | WCAG 2.1 접근성 검증 |
| 리포트 | Allure | 통합 테스트 리포트 |

### 3.3 테스트 데이터

| 구분 | 내용 | 위치 |
|------|------|------|
| 샘플 오디오 (한국어) | 5분 회의 녹음, 2명 화자, 16kHz mono WAV | `test-data/audio/ko_meeting_5min.wav` |
| 샘플 오디오 (영어) | 5분 회의 녹음, 3명 화자, 16kHz mono WAV | `test-data/audio/en_meeting_5min.wav` |
| 샘플 오디오 (일본어) | 5분 회의 녹음, 2명 화자, 16kHz mono WAV | `test-data/audio/ja_meeting_5min.wav` |
| 참조 전사문 (한국어) | 수동 검증된 전사 텍스트 (WER 측정 기준) | `test-data/reference/ko_transcript.json` |
| 참조 전사문 (영어) | 수동 검증된 전사 텍스트 (WER 측정 기준) | `test-data/reference/en_transcript.json` |
| 참조 전사문 (일본어) | 수동 검증된 전사 텍스트 (WER 측정 기준) | `test-data/reference/ja_transcript.json` |
| Mock API 응답 (OpenAI) | Chat Completions, Whisper, TTS 응답 JSON | `test-data/mocks/openai/` |
| Mock API 응답 (Anthropic) | Messages API 응답 JSON | `test-data/mocks/anthropic/` |
| Mock API 응답 (Gemini) | Gemini API 응답 JSON | `test-data/mocks/gemini/` |
| 테스트용 GGUF 모델 | Whisper tiny (40MB), 경량 LLM | `test-data/models/` |
| 테스트 DB 스냅샷 | 사전 구성된 SQLite 데이터베이스 | `test-data/db/` |

---

## 4. 테스트 케이스 ID 체계

### 4.1 명명 규칙

```
TC-{카테고리}-{요구사항번호}-{순번}
```

| 요소 | 설명 | 예시 |
|------|------|------|
| `TC` | Test Case 접두어 | — |
| `{카테고리}` | SRS 요구사항 카테고리 코드 | `AUD`, `STT`, `LLM`, `API`, `PERF` 등 |
| `{요구사항번호}` | SRS 요구사항 일련번호 (3자리) | `001`, `002`, ..., `013` |
| `{순번}` | 동일 요구사항 내 테스트 케이스 순번 (2자리) | `01`, `02`, ..., `99` |

**예시:**
- `TC-AUD-001-01` : FR-AUD-001 (마이크 캡처)의 첫 번째 테스트 케이스
- `TC-STT-004-03` : FR-STT-004 (다국어 감지)의 세 번째 테스트 케이스
- `TC-PERF-001-01` : NFR-PERF-001 (전사 지연)의 첫 번째 테스트 케이스

### 4.2 매핑 원칙

| 원칙 | 설명 |
|------|------|
| **1:1 매핑** | 모든 SRS 요구사항은 최소 1개 이상의 TC를 가져야 한다 |
| **인수 조건 기반** | 각 요구사항의 인수 조건(Acceptance Criteria)당 최소 1개 TC를 작성한다 |
| **경계값 포함** | 성능 관련 NFR은 목표치의 경계값 테스트를 반드시 포함한다 |
| **네거티브 테스트** | P0 요구사항은 정상 경로 외 실패/예외 경로 TC를 반드시 포함한다 |
| **크로스플랫폼** | 플랫폼 의존 기능은 각 타겟 플랫폼별로 별도 TC를 작성한다 |

### 4.3 비기능 테스트 케이스 ID

비기능 요구사항은 SRS 카테고리 코드를 그대로 사용한다.

| SRS 카테고리 | TC 접두어 | 예시 |
|-------------|----------|------|
| NFR-PERF | `TC-PERF` | `TC-PERF-001-01` |
| NFR-SEC | `TC-SEC` | `TC-SEC-001-01` |
| NFR-PLAT | `TC-PLAT` | `TC-PLAT-001-01` |
| NFR-REL | `TC-REL` | `TC-REL-001-01` |
| NFR-USAB | `TC-USAB` | `TC-USAB-001-01` |
| NFR-MAINT | `TC-MAINT` | `TC-MAINT-001-01` |
| NFR-EXT (확장성) | `TC-NEXT` | `TC-NEXT-001-01` |

---

## 5. 테스트 도구 스택

| 레벨 | 도구 | 대상 | 출력 |
|------|------|------|------|
| Rust 단위/통합 테스트 | cargo-nextest | Rust 코어 모듈 (`voxnote-core`, `voxnote-tauri`, `voxnote-server`) | JUnit XML (`test-results/rust/junit.xml`) |
| Rust 성능 벤치마크 | criterion | 오디오 파이프라인, STT 추론, 암호화 | HTML 리포트 (`target/criterion/`) |
| Rust 코드 커버리지 | cargo-tarpaulin | 전체 Rust 코드베이스 | LCOV (`test-results/coverage/lcov.info`) + HTML (`test-results/coverage/html/`) |
| Rust 퍼징 테스트 | cargo-fuzz | 파서, 암호화, IPC 직렬화 | Crash artifacts (`fuzz/artifacts/`) |
| API Mock | wiremock-rs | 외부 LLM Provider API (OpenAI, Anthropic, Gemini 등) | 테스트 내 사용 (별도 출력 없음) |
| Frontend 단위 테스트 | vitest + @testing-library/react | React 컴포넌트, 커스텀 훅, 유틸리티 | JUnit XML (`test-results/frontend/junit.xml`) |
| E2E/UI 테스트 | Playwright + tauri-driver | 전체 애플리케이션 (데스크탑) | JUnit XML (`test-results/e2e/junit.xml`) + 스크린샷 (`test-results/e2e/screenshots/`) |
| 접근성 테스트 | axe-core (Playwright 통합) | WebView UI | WCAG 위반 리포트 (`test-results/a11y/`) |
| 통합 리포트 | Allure | 전체 테스트 결과 통합 | HTML 대시보드 (`test-results/allure-report/`) |

---

## 6. 테스트 실행 방법

### 6.1 통합 실행 스크립트

`scripts/test-all.sh` 스크립트를 통해 모든 테스트를 일관성 있게 실행한다.

```bash
#!/usr/bin/env bash
# scripts/test-all.sh — VoxNote 통합 테스트 실행 스크립트
#
# 사용법:
#   ./scripts/test-all.sh --all          # 전체 테스트 실행
#   ./scripts/test-all.sh --unit         # Rust + TS 단위 테스트
#   ./scripts/test-all.sh --integration  # 통합 테스트
#   ./scripts/test-all.sh --e2e          # E2E/UI 테스트
#   ./scripts/test-all.sh --bench        # 성능 벤치마크
#   ./scripts/test-all.sh --fuzz         # 퍼징 테스트 (시간 제한)
#   ./scripts/test-all.sh --coverage     # 커버리지 측정
#   ./scripts/test-all.sh --report       # Allure 리포트 생성
#
# 옵션 조합 가능:
#   ./scripts/test-all.sh --unit --integration --report

set -euo pipefail

RESULTS_DIR="test-results"
ALLURE_RESULTS="$RESULTS_DIR/allure-results"

# --- 옵션 파싱 ---
RUN_UNIT=false
RUN_INTEGRATION=false
RUN_E2E=false
RUN_BENCH=false
RUN_FUZZ=false
RUN_COVERAGE=false
GEN_REPORT=false

for arg in "$@"; do
  case $arg in
    --unit)         RUN_UNIT=true ;;
    --integration)  RUN_INTEGRATION=true ;;
    --e2e)          RUN_E2E=true ;;
    --bench)        RUN_BENCH=true ;;
    --fuzz)         RUN_FUZZ=true ;;
    --coverage)     RUN_COVERAGE=true ;;
    --report)       GEN_REPORT=true ;;
    --all)
      RUN_UNIT=true
      RUN_INTEGRATION=true
      RUN_E2E=true
      RUN_BENCH=true
      RUN_COVERAGE=true
      GEN_REPORT=true
      ;;
  esac
done

mkdir -p "$RESULTS_DIR" "$ALLURE_RESULTS"

# --- Rust 단위 테스트 ---
if $RUN_UNIT; then
  echo "=== Rust 단위 테스트 실행 ==="
  cargo nextest run --workspace \
    --profile ci \
    --junit-file "$RESULTS_DIR/rust/junit.xml" \
    --filter-expr 'not test(integration::)'

  echo "=== Frontend 단위 테스트 실행 ==="
  cd frontend && npx vitest run \
    --reporter=junit \
    --outputFile="../$RESULTS_DIR/frontend/junit.xml" \
    && cd ..
fi

# --- 통합 테스트 ---
if $RUN_INTEGRATION; then
  echo "=== 통합 테스트 실행 ==="
  cargo nextest run --workspace \
    --profile ci \
    --junit-file "$RESULTS_DIR/integration/junit.xml" \
    --filter-expr 'test(integration::)'
fi

# --- E2E/UI 테스트 ---
if $RUN_E2E; then
  echo "=== E2E/UI 테스트 실행 ==="
  npx playwright test \
    --reporter=junit \
    --output "$RESULTS_DIR/e2e/" \
    2>&1 | tee "$RESULTS_DIR/e2e/playwright.log"
fi

# --- 성능 벤치마크 ---
if $RUN_BENCH; then
  echo "=== 성능 벤치마크 실행 ==="
  cargo bench --workspace -- --output-format=bencher \
    | tee "$RESULTS_DIR/bench/results.txt"
fi

# --- 퍼징 테스트 ---
if $RUN_FUZZ; then
  echo "=== 퍼징 테스트 실행 (300초 제한) ==="
  for target in $(cargo fuzz list 2>/dev/null); do
    cargo fuzz run "$target" -- -max_total_time=300
  done
fi

# --- 커버리지 ---
if $RUN_COVERAGE; then
  echo "=== 코드 커버리지 측정 ==="
  cargo tarpaulin --workspace \
    --out Lcov Html \
    --output-dir "$RESULTS_DIR/coverage/"
fi

# --- Allure 리포트 ---
if $GEN_REPORT; then
  echo "=== Allure 리포트 생성 ==="
  cp "$RESULTS_DIR"/rust/*.xml "$ALLURE_RESULTS/" 2>/dev/null || true
  cp "$RESULTS_DIR"/frontend/*.xml "$ALLURE_RESULTS/" 2>/dev/null || true
  cp "$RESULTS_DIR"/integration/*.xml "$ALLURE_RESULTS/" 2>/dev/null || true
  cp "$RESULTS_DIR"/e2e/*.xml "$ALLURE_RESULTS/" 2>/dev/null || true

  allure generate "$ALLURE_RESULTS" \
    -o "$RESULTS_DIR/allure-report" \
    --clean

  echo "=== Markdown 요약 생성 ==="
  # 자동 요약 스크립트 호출
  node scripts/generate-test-summary.js \
    --input "$RESULTS_DIR/allure-report" \
    --output "$RESULTS_DIR/SUMMARY.md"

  echo "리포트: $RESULTS_DIR/allure-report/index.html"
fi

echo "=== 테스트 완료 ==="
```

### 6.2 개별 실행 명령어

#### Rust 단위 테스트

```bash
# 전체 Rust 단위 테스트
cargo nextest run --workspace

# 특정 모듈만 실행
cargo nextest run -p voxnote-core --filter-expr 'test(audio::)'

# 특정 테스트만 실행
cargo nextest run -p voxnote-core --filter-expr 'test(test_resample_16khz)'
```

#### Frontend 단위 테스트

```bash
# 전체 Frontend 테스트
cd frontend && npx vitest run

# 감시 모드
cd frontend && npx vitest

# 특정 파일
cd frontend && npx vitest run src/components/TranscriptView.test.tsx
```

#### 통합 테스트

```bash
# 전체 통합 테스트
cargo nextest run --workspace --filter-expr 'test(integration::)'

# wiremock 기반 API 통합 테스트
cargo nextest run -p voxnote-core --filter-expr 'test(integration::api::)'
```

#### E2E/UI 테스트

```bash
# 전체 E2E 테스트
npx playwright test

# 특정 시나리오
npx playwright test tests/e2e/recording-flow.spec.ts

# UI 모드 (디버그)
npx playwright test --ui

# 특정 브라우저/플랫폼
npx playwright test --project=desktop-macos
```

#### 성능 벤치마크

```bash
# 전체 벤치마크
cargo bench --workspace

# 특정 벤치마크
cargo bench --bench audio_pipeline

# 기준선 대비 비교
cargo bench --workspace -- --save-baseline main
cargo bench --workspace -- --baseline main
```

#### 커버리지

```bash
# Rust 커버리지
cargo tarpaulin --workspace --out Html Lcov

# Frontend 커버리지
cd frontend && npx vitest run --coverage
```

#### 퍼징

```bash
# 퍼징 대상 목록
cargo fuzz list

# 특정 대상 퍼징 (10분)
cargo fuzz run fuzz_audio_parser -- -max_total_time=600

# 크래시 재현
cargo fuzz run fuzz_audio_parser fuzz/artifacts/fuzz_audio_parser/<artifact>
```

---

## 7. 리포트 생성 및 관리

### 7.1 Allure 통합 리포트

모든 테스트 프레임워크의 결과를 JUnit XML 형식으로 출력하고, Allure로 통합하여 단일 HTML 대시보드를 생성한다.

```
[cargo-nextest] ──→ JUnit XML ──┐
[vitest]        ──→ JUnit XML ──┼──→ [Allure Generate] ──→ HTML Dashboard
[Playwright]    ──→ JUnit XML ──┘
```

**Allure 대시보드 제공 정보:**
- 전체 Pass/Fail/Skip 현황
- 카테고리별 테스트 결과
- 실행 시간 추이 (히스토리)
- 실패 테스트 상세 (스택 트레이스, 스크린샷)
- 요구사항별 커버리지 매핑

### 7.2 Markdown 요약 리포트

각 테스트 실행 후 `test-results/SUMMARY.md`에 자동 요약을 생성한다.

```markdown
# VoxNote 테스트 요약 — {날짜}

| 항목 | 값 |
|------|-----|
| 실행 일시 | 2026-03-27 14:30:00 KST |
| 커밋 | abc1234 |
| 총 TC | 450 |
| Pass | 432 (96.0%) |
| Fail | 12 (2.7%) |
| Skip | 6 (1.3%) |
| Rust 커버리지 | 82.3% |
| TS 커버리지 | 78.5% |
```

### 7.3 디렉토리 구조

```
test-results/
├── rust/
│   └── junit.xml                  # Rust 단위 테스트 결과
├── frontend/
│   └── junit.xml                  # Frontend 단위 테스트 결과
├── integration/
│   └── junit.xml                  # 통합 테스트 결과
├── e2e/
│   ├── junit.xml                  # E2E 테스트 결과
│   ├── screenshots/               # 실패 시 스크린샷
│   │   └── recording-flow-001.png
│   └── playwright.log             # 실행 로그
├── bench/
│   └── results.txt                # 벤치마크 결과
├── coverage/
│   ├── lcov.info                  # LCOV 커버리지 데이터
│   └── html/
│       └── index.html             # HTML 커버리지 리포트
├── a11y/
│   └── axe-results.json           # 접근성 검사 결과
├── allure-results/                # Allure 입력 (JUnit XML 집계)
│   ├── rust-junit.xml
│   ├── frontend-junit.xml
│   ├── integration-junit.xml
│   └── e2e-junit.xml
├── allure-report/                 # Allure HTML 대시보드
│   └── index.html
├── history/                       # 히스토리 데이터
│   └── 2026-03-27/
│       ├── SUMMARY.md
│       └── allure-report/
└── SUMMARY.md                     # 최신 요약 리포트
```

### 7.4 리포트 히스토리 관리

| 항목 | 방안 |
|------|------|
| **히스토리 보존** | 각 실행 결과를 `test-results/history/{날짜}/`에 아카이브 |
| **Allure 트렌드** | `allure-report/history/` 디렉토리를 실행 간 복사하여 추이 그래프 유지 |
| **보존 기간** | 로컬: 최근 30일 / CI: 최근 90일 또는 100회 실행 |
| **용량 관리** | 스크린샷은 실패 케이스만 보존, 성공 케이스는 자동 삭제 |
| **Git 연동** | `test-results/` 디렉토리는 `.gitignore`에 추가, CI artifact로만 관리 |

---

## 8. CI/CD 통합

### 8.1 GitHub Actions 매트릭스 빌드

```yaml
# .github/workflows/test.yml (구조 설계)
name: VoxNote Test Suite

on:
  push:
    branches: [main, develop]
  pull_request:
    branches: [main, develop]

jobs:
  test:
    strategy:
      matrix:
        os: [macos-14, windows-latest, ubuntu-22.04]
        include:
          - os: macos-14
            rust-target: aarch64-apple-darwin
          - os: windows-latest
            rust-target: x86_64-pc-windows-msvc
          - os: ubuntu-22.04
            rust-target: x86_64-unknown-linux-gnu

    runs-on: ${{ matrix.os }}

    steps:
      # 1. 체크아웃 + 도구 설치
      # 2. Rust 단위/통합 테스트 (cargo-nextest)
      # 3. Frontend 단위 테스트 (vitest)
      # 4. E2E 테스트 (Playwright, ubuntu만)
      # 5. 커버리지 측정 (cargo-tarpaulin, ubuntu만)
      # 6. Allure 리포트 생성
      # 7. Artifact 업로드
```

### 8.2 PR 코멘트 자동 게시

PR 생성/업데이트 시 테스트 결과 요약을 자동으로 코멘트에 게시한다.

```
## VoxNote 테스트 결과

| 플랫폼 | 단위(Rust) | 단위(TS) | 통합 | E2E | 상태 |
|--------|-----------|---------|------|-----|------|
| macOS  | 234/234   | 89/89   | 45/45| —   | Pass |
| Windows| 230/234   | 89/89   | 44/45| —   | Fail |
| Linux  | 234/234   | 89/89   | 45/45| 32/32| Pass |

**Rust 커버리지:** 82.3% (+1.2%)
**TS 커버리지:** 78.5% (-0.3%)

> 4건 실패 — [상세 리포트](링크)
```

**구현 방안:**
- `actions/github-script` 또는 별도 Action을 사용하여 JUnit XML 파싱 후 코멘트 작성
- 이전 코멘트를 업데이트 (중복 코멘트 방지)
- 실패 시 PR에 `test-failure` 라벨 자동 부착

### 8.3 커버리지 뱃지

| 뱃지 | 도구 | 기준 |
|------|------|------|
| Rust 커버리지 | cargo-tarpaulin + Codecov/Coveralls | 최소 75%, 목표 85% |
| TS 커버리지 | vitest --coverage + Codecov | 최소 70%, 목표 80% |
| 빌드 상태 | GitHub Actions | Pass/Fail |
| 테스트 결과 | GitHub Actions | Total/Pass/Fail |

`README.md`에 다음 뱃지를 표시한다:

```markdown
![Build](https://img.shields.io/github/actions/workflow/status/...)
![Rust Coverage](https://img.shields.io/codecov/c/github/.../rust)
![TS Coverage](https://img.shields.io/codecov/c/github/.../typescript)
![Tests](https://img.shields.io/badge/tests-450%20passed-brightgreen)
```

---

## 9. 테스트 카테고리 목록

### 9.1 기능 테스트 카테고리 (13개)

| 카테고리 | SRS 참조 | 요구사항 수 | TC 파일 | 설명 |
|----------|---------|-----------|---------|------|
| TC-AUD | FR-AUD-001 ~ 010 | 10 | [TC-AUD.md](test-cases/TC-AUD.md) | 오디오 파이프라인 (마이크 캡처, 리샘플링, VAD, 시스템 사운드) |
| TC-STT | FR-STT-001 ~ 010 | 10 | [TC-STT.md](test-cases/TC-STT.md) | 음성-텍스트 변환 (실시간 전사, 다국어, GPU 가속, 파일 전사) |
| TC-LLM | FR-LLM-001 ~ 010 | 10 | [TC-LLM.md](test-cases/TC-LLM.md) | LLM 엔진 (로컬 추론, 요약, 문서 생성, RAG, 스트리밍) |
| TC-API | FR-API-001 ~ 013 | 13 | [TC-API.md](test-cases/TC-API.md) | 외부 LLM Provider API 연동 (OpenAI, Anthropic, Gemini, 폴백) |
| TC-TTS | FR-TTS-001 ~ 004 | 4 | [TC-TTS.md](test-cases/TC-TTS.md) | 음성 합성 (Piper TTS, 다국어, 외부 TTS API) |
| TC-DIA | FR-DIA-001 ~ 005 | 5 | [TC-DIA.md](test-cases/TC-DIA.md) | 화자 분리 (임베딩 추출, 클러스터링, 화자 태깅) |
| TC-STO | FR-STO-001 ~ 007 | 7 | [TC-STO.md](test-cases/TC-STO.md) | 로컬 저장소 (SQLite, FTS5, E2EE, 폴더 관리) |
| TC-SYN | FR-SYN-001 ~ 005 | 5 | [TC-SYN.md](test-cases/TC-SYN.md) | 디바이스 간 동기화 (CRDT, 암호화 전송, WebSocket) |
| TC-MDL | FR-MDL-001 ~ 007 | 7 | [TC-MDL.md](test-cases/TC-MDL.md) | 모델 관리 (카탈로그, 다운로드, 무결성 검증, 핫스왑) |
| TC-SRV | FR-SRV-001 ~ 008 | 8 | [TC-SRV.md](test-cases/TC-SRV.md) | 서버 기능 (인증, 라이선스, 동기화 릴레이, CDN) |
| TC-UI | FR-UI-001 ~ 011 | 11 | [TC-UI.md](test-cases/TC-UI.md) | 사용자 인터페이스 (전사 뷰, 에디터, 설정, Provider 패널) |
| TC-MOB | FR-MOB-001 ~ 005 | 5 | [TC-MOB.md](test-cases/TC-MOB.md) | 모바일 전략 (온디바이스, LAN 위임, 클라우드 opt-in) |
| TC-EXT | FR-EXT-001 ~ 004 | 4 | [TC-EXT.md](test-cases/TC-EXT.md) | 외부 연동 (Notion, Slack, Confluence, 문서 내보내기) |

### 9.2 비기능 테스트 카테고리 (7개)

| 카테고리 | SRS 참조 | 요구사항 수 | TC 파일 | 설명 |
|----------|---------|-----------|---------|------|
| TC-PERF | NFR-PERF-001 ~ 007 | 7 | [TC-PERF.md](test-cases/TC-PERF.md) | 성능 (전사 지연, 앱 시작 시간, 메모리, 바이너리 크기) |
| TC-SEC | NFR-SEC-001 ~ 008 | 8 | [TC-SEC.md](test-cases/TC-SEC.md) | 보안 및 프라이버시 (E2EE, 키체인, 퍼징, 데이터 유출 방지) |
| TC-PLAT | NFR-PLAT-001 ~ 005 | 5 | [TC-PLAT.md](test-cases/TC-PLAT.md) | 크로스플랫폼 호환성 (macOS, Windows, Linux, iOS, Android) |
| TC-REL | NFR-REL-001 ~ 005 | 5 | [TC-REL.md](test-cases/TC-REL.md) | 신뢰성 (오프라인 동작, 데이터 유실 방지, 충돌 해결, 폴백) |
| TC-USAB | NFR-USAB-001 ~ 004 | 4 | [TC-USAB.md](test-cases/TC-USAB.md) | 사용성 (모델 자동 추천, 클릭 횟수, 접근성, Provider UX) |
| TC-MAINT | NFR-MAINT-001 ~ 004 | 4 | [TC-MAINT.md](test-cases/TC-MAINT.md) | 유지보수성 (워크스페이스 구조, feature flag, 조건부 컴파일, CI/CD) |
| TC-NEXT | NFR-EXT-001 ~ 004 | 4 | [TC-NEXT.md](test-cases/TC-NEXT.md) | 확장성 (플러그인 구조, 추상화 인터페이스, 모델 레지스트리, API 호환) |

### 9.3 요구사항 수 요약

| 구분 | 카테고리 수 | 요구사항 수 |
|------|-----------|-----------|
| 기능 (FR) | 13 | 93 |
| 비기능 (NFR) | 7 | 37 |
| **합계** | **20** | **130** |

---

## 10. 테스트 일정

### 10.1 Phase별 상세 일정

#### Phase 1: Foundation (Month 1~3)

| 월 | 대상 카테고리 | 주요 활동 | 산출물 |
|----|-------------|----------|--------|
| Month 1 | TC-AUD, TC-STO | 오디오 캡처/리샘플링/VAD 단위 테스트, SQLite CRUD 단위 테스트 | 단위 TC 작성 및 자동화 |
| Month 2 | TC-STT, TC-MDL | STT 스트리밍 전사 통합 테스트, 모델 다운로드/검증 통합 테스트 | 통합 TC 작성 및 자동화 |
| Month 3 | TC-UI (기본), TC-SRV (CDN) | 전사 뷰/에디터/설정 UI E2E 테스트, CDN 모델 배포 테스트 | E2E TC 작성, Phase 1 리포트 |

**Phase 1 완료 기준:**
- TC-AUD-001~004 전체 Pass
- TC-STT-001~006, 008 전체 Pass
- TC-STO-001~004, 007 전체 Pass
- TC-MDL-001~004 전체 Pass
- P0 Rust 커버리지 80% 이상

#### Phase 2: Intelligence (Month 4~6)

| 월 | 대상 카테고리 | 주요 활동 | 산출물 |
|----|-------------|----------|--------|
| Month 4 | TC-LLM | 로컬 LLM 추론, 요약 생성, GBNF Grammar 테스트 | LLM 단위/통합 TC |
| Month 5 | TC-API, TC-DIA | Provider API mock 통합 테스트, 화자 분리 정확도 테스트 | API/DIA TC, wiremock 시나리오 |
| Month 6 | TC-EXT (문서) | Markdown/PDF/DOCX 내보내기 테스트, Phase 2 전체 회귀 | 내보내기 TC, Phase 2 리포트 |

**Phase 2 완료 기준:**
- TC-LLM-001~003, 007~008 전체 Pass
- TC-API-001~003, 007~009, 013 전체 Pass
- TC-DIA-001~002, 004 전체 Pass
- Provider 폴백 시나리오 검증 완료

#### Phase 3: Platform (Month 7~9)

| 월 | 대상 카테고리 | 주요 활동 | 산출물 |
|----|-------------|----------|--------|
| Month 7 | TC-MOB, TC-SYN | 모바일 온디바이스 STT 테스트, CRDT 동기화 통합 테스트 | 모바일 TC, 동기화 TC |
| Month 8 | TC-TTS, TC-SRV (인증) | TTS 음성 합성 테스트, OAuth2/JWT 인증 플로우 테스트 | TTS TC, 인증 TC |
| Month 9 | TC-PLAT | 3개 데스크탑 + 2개 모바일 크로스플랫폼 E2E 테스트 | 플랫폼별 E2E 리포트, Phase 3 리포트 |

**Phase 3 완료 기준:**
- TC-MOB-001, 003~005 전체 Pass
- TC-SYN-001~005 전체 Pass
- TC-SRV-001~003, 006~008 전체 Pass
- 5개 플랫폼 E2E 시나리오 Pass

#### Phase 4: Polish (Month 10~12)

| 월 | 대상 카테고리 | 주요 활동 | 산출물 |
|----|-------------|----------|--------|
| Month 10 | TC-AUD (시스템 사운드), TC-SRV (알림) | 시스템 사운드 캡처 플랫폼별 테스트, 푸시 알림 테스트 | 시스템 사운드 TC, 알림 TC |
| Month 11 | TC-EXT (외부 연동), TC-NFR 전체 | Notion/Slack/Confluence 연동, 전체 NFR 검증 | 외부 연동 TC, NFR TC |
| Month 12 | 전체 회귀 | 130개 전체 요구사항 회귀 테스트, 성능/보안 최종 검증 | 최종 테스트 리포트, 릴리즈 판정 |

**Phase 4 완료 기준:**
- 전체 TC Pass율 95% 이상
- NFR-PERF 전 항목 목표치 달성
- NFR-SEC 보안 감사 및 퍼징 완료
- 릴리즈 후보(RC) 승인

### 10.2 마일스톤 요약

```
Month 1  ──── Month 3  ──── Month 6  ──── Month 9  ──── Month 12
  │              │              │              │              │
  ├── Phase 1 ──┤              │              │              │
  │  Foundation  │              │              │              │
  │              ├── Phase 2 ──┤              │              │
  │              │ Intelligence │              │              │
  │              │              ├── Phase 3 ──┤              │
  │              │              │  Platform    │              │
  │              │              │              ├── Phase 4 ──┤
  │              │              │              │   Polish     │
  │              │              │              │              │
  ▼              ▼              ▼              ▼              ▼
 개발착수     Phase 1 완료  Phase 2 완료  Phase 3 완료    GA 릴리즈
             P0 검증완료   AI 통합검증   크로스플랫폼    전체 검증완료
                           완료          검증완료
```

---

## 부록

### 부록 A. 테스트 결과 판정 기준

| 판정 | 코드 | 기준 | 설명 |
|------|------|------|------|
| **Pass** | `PASS` | 기대 결과와 실제 결과가 일치 | 테스트 케이스의 모든 검증 항목을 충족 |
| **Fail** | `FAIL` | 기대 결과와 실제 결과가 불일치 | 하나 이상의 검증 항목 미충족. 결함 보고서 작성 필수 |
| **Skip** | `SKIP` | 실행 전제 조건 미충족으로 실행 불가 | 환경 미구성, 의존 기능 미구현 등. 사유 기록 필수 |
| **N/A** | `N/A` | 해당 테스트 케이스가 현재 빌드/환경에 적용 불가 | 플랫폼 미지원, 기능 삭제 등. 적용 제외 사유 기록 필수 |

**판정 규칙:**
- 하나의 TC 내에 다수 검증 항목이 있을 경우, **모든 항목이 Pass**여야 해당 TC가 Pass이다
- 성능 테스트는 목표치의 **110% 이내**를 Pass로 판정한다 (예: 목표 300ms → 330ms까지 Pass)
- 퍼징 테스트는 **크래시 미발견** 시 Pass로 판정한다
- Skip과 N/A가 전체 TC의 **10%를 초과**할 경우 테스트 계획을 재검토한다

### 부록 B. 결함 심각도 분류

| 심각도 | 코드 | 기준 | 대응 시한 | 예시 |
|--------|------|------|----------|------|
| **Critical** | `S1` | 시스템 장애, 데이터 유실, 보안 취약점 | **24시간 이내** 수정 착수 | 녹음 데이터 유실, E2EE 우회, 앱 크래시 |
| **Major** | `S2` | 핵심 기능 불능, 우회 경로 없음 | **3일 이내** 수정 착수 | 전사 기능 미동작, 모델 다운로드 실패, Provider 전환 불가 |
| **Minor** | `S3` | 기능 동작하나 불편, 우회 경로 존재 | **1주 이내** 수정 착수 | UI 정렬 깨짐, 느린 응답, 부정확한 화자 라벨 |
| **Trivial** | `S4` | 미관, 오탈자, 개선 사항 | **다음 릴리즈** 반영 | 오탈자, 아이콘 해상도, 로그 메시지 문구 |

**결함 관리 규칙:**
- 모든 Fail 판정 TC에 대해 결함 보고서를 작성하고 GitHub Issues에 등록한다
- Critical(S1) 결함은 발견 즉시 팀 채널에 공유하고, 릴리즈를 차단(blocker)한다
- Major(S2) 결함은 해당 Phase 종료 전 반드시 해결한다
- Minor(S3) 이상 결함이 잔존하는 상태에서는 릴리즈 판정을 보류한다
- 결함 보고서에는 **재현 절차**, **기대 결과**, **실제 결과**, **환경 정보**, **스크린샷/로그**를 필수 포함한다

---

*본 문서는 VoxNote 프로젝트 SRS v1.0을 기반으로 작성되었으며, 요구사항 변경 시 함께 갱신된다.*
