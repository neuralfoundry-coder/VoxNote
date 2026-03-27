# VoxNote 빌드 & 배포 가이드

**버전:** 0.1.0
**작성일:** 2026-03-27
**대상:** 개발자, DevOps, 릴리즈 담당자

---

## 목차

1. [사전 요구사항](#1-사전-요구사항)
2. [프로젝트 구조](#2-프로젝트-구조)
3. [개발 환경 설정](#3-개발-환경-설정)
4. [데스크탑 빌드](#4-데스크탑-빌드)
5. [서버 빌드](#5-서버-빌드)
6. [모바일 빌드](#6-모바일-빌드)
7. [WASM 빌드](#7-wasm-빌드)
8. [AI 모델 관리](#8-ai-모델-관리)
9. [Feature Flags](#9-feature-flags)
10. [테스트](#10-테스트)
11. [CI/CD 파이프라인](#11-cicd-파이프라인)
12. [릴리즈 배포](#12-릴리즈-배포)
13. [문제 해결](#13-문제-해결)

---

## 1. 사전 요구사항

### 필수 도구

| 도구 | 최소 버전 | 설치 |
|------|----------|------|
| Rust (stable) | 1.80+ | `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \| sh` |
| Node.js | 20+ | [nodejs.org](https://nodejs.org) 또는 `brew install node` |
| pnpm | 9+ | `npm install -g pnpm` |
| Tauri CLI | 2.0+ | `cargo install tauri-cli --version "^2"` |

### 플랫폼별 추가 요구사항

**macOS:**
```bash
xcode-select --install   # Xcode Command Line Tools
```

**Windows:**
- Visual Studio Build Tools (MSVC)
- WebView2 Runtime (Windows 10/11 기본 포함)
- CUDA Toolkit 12+ (GPU 가속 사용 시)

**Linux (Debian/Ubuntu):**
```bash
sudo apt install -y \
  build-essential pkg-config \
  libssl-dev libgtk-3-dev libwebkit2gtk-4.1-dev \
  libasound2-dev libayatana-appindicator3-dev librsvg2-dev
```

**Linux (Fedora):**
```bash
sudo dnf install -y \
  gcc-c++ pkg-config openssl-devel \
  gtk3-devel webkit2gtk4.1-devel \
  alsa-lib-devel libappindicator-gtk3-devel librsvg2-devel
```

### 버전 확인

```bash
rustc --version          # rustc 1.80.0+
cargo --version          # cargo 1.80.0+
cargo tauri --version    # tauri-cli 2.x
node --version           # v20+
pnpm --version           # 9+
```

---

## 2. 프로젝트 구조

```
voxnote/
├── Cargo.toml                    # 워크스페이스 루트
├── rust-toolchain.toml           # Rust stable 고정
├── crates/
│   ├── voxnote-core/             # 핵심 비즈니스 로직 (순수 Rust)
│   ├── voxnote-tauri/            # 데스크탑/모바일 앱 (Tauri 2)
│   │   ├── tauri.conf.json       # Tauri 설정
│   │   ├── capabilities/         # 권한 설정
│   │   └── icons/                # 앱 아이콘
│   ├── voxnote-server/           # 동기화 서버 (Axum)
│   └── voxnote-wasm/             # 웹 브라우저 타겟
├── frontend/                     # React + TypeScript
│   ├── package.json
│   ├── vite.config.ts
│   └── src/
├── models/
│   └── registry.toml             # AI 모델 카탈로그
├── scripts/
│   └── test-all.sh               # 전체 테스트 스크립트
└── docs/                         # 문서
```

### 의존성 방향

```
voxnote-core (프레임워크 무관, 순수 Rust)
    ↑
    ├── voxnote-tauri   (Tauri 의존)
    ├── voxnote-server  (Axum 의존)
    └── voxnote-wasm    (wasm-bindgen 의존)
```

> `voxnote-core`는 Tauri, Axum, wasm-bindgen에 절대 의존하지 않습니다.

---

## 3. 개발 환경 설정

### 3.1 저장소 클론

```bash
git clone https://github.com/example/voxnote.git
cd voxnote
```

### 3.2 Frontend 의존성 설치

```bash
cd frontend
pnpm install
cd ..
```

### 3.3 STT 모델 다운로드

```bash
mkdir -p ~/.voxnote/models

# Whisper Tiny (74MB) — 개발/테스트용
curl -L -o ~/.voxnote/models/ggml-tiny.bin \
  "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"

# Whisper Base (148MB) — 기본 품질
curl -L -o ~/.voxnote/models/ggml-base.bin \
  "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-base.bin"
```

### 3.4 개발 서버 실행

```bash
# 터미널 1: Frontend 개발 서버
cd frontend && pnpm dev

# 터미널 2: Tauri 앱 실행 (핫 리로드)
cargo tauri dev
```

앱이 `http://localhost:1420`에서 Frontend를 로드하고 네이티브 윈도우에 표시됩니다.

---

## 4. 데스크탑 빌드

### 4.1 개발 빌드 (디버그)

```bash
# Frontend 빌드
cd frontend && pnpm build && cd ..

# Tauri 앱 빌드 (디버그, 빠르지만 최적화 없음)
cargo build -p voxnote-tauri --features stt,desktop
```

### 4.2 릴리즈 빌드

```bash
# Frontend 빌드
cd frontend && pnpm build && cd ..

# macOS (Apple Silicon + Metal GPU)
cargo tauri build --no-bundle
# 또는 번들 포함 (.dmg)
cargo tauri build
```

### 4.3 플랫폼별 릴리즈 빌드

| 플랫폼 | 명령어 | 출력 |
|--------|--------|------|
| **macOS (ARM)** | `cargo tauri build` | `.dmg`, `.app` |
| **macOS (Intel)** | `cargo tauri build --target x86_64-apple-darwin` | `.dmg` |
| **Windows** | `cargo tauri build` | `.msi`, `.exe` |
| **Linux** | `cargo tauri build` | `.AppImage`, `.deb` |

### 4.4 GPU 가속 빌드

```bash
# macOS Metal (Apple Silicon)
cargo build -p voxnote-tauri --features stt,desktop,metal --release

# Windows/Linux CUDA
cargo build -p voxnote-tauri --features stt,desktop,cuda --release

# Windows/Linux Vulkan
cargo build -p voxnote-tauri --features stt,desktop,vulkan --release
```

### 4.5 빌드 결과

```
target/release/voxnote-tauri          # 네이티브 바이너리 (~9.4MB)
target/release/bundle/
├── dmg/VoxNote_0.1.0_aarch64.dmg     # macOS 디스크 이미지
├── macos/VoxNote.app/                # macOS 앱 번들
├── msi/VoxNote_0.1.0_x64.msi         # Windows 설치 파일
├── nsis/VoxNote_0.1.0_x64-setup.exe  # Windows NSIS 설치 파일
├── appimage/VoxNote_0.1.0.AppImage   # Linux AppImage
└── deb/voxnote_0.1.0_amd64.deb      # Debian 패키지
```

---

## 5. 서버 빌드

### 5.1 로컬 빌드

```bash
cargo build -p voxnote-server --release
```

### 5.2 Docker 빌드 (권장)

```dockerfile
# Dockerfile
FROM rust:1.80-slim AS builder
WORKDIR /app
COPY . .
RUN cargo build -p voxnote-server --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/voxnote-server /usr/local/bin/
EXPOSE 8080
CMD ["voxnote-server"]
```

```bash
docker build -t voxnote-server .
docker run -p 8080:8080 voxnote-server
```

### 5.3 서버 환경 변수

| 변수 | 설명 | 기본값 |
|------|------|--------|
| `LISTEN_ADDR` | 바인드 주소 | `0.0.0.0:8080` |
| `DATABASE_URL` | PostgreSQL URL | (필수) |
| `REDIS_URL` | Redis URL | (선택) |
| `JWT_SECRET` | JWT 서명 비밀키 | (필수) |
| `RUST_LOG` | 로그 레벨 | `info` |

### 5.4 서버 API 엔드포인트

| 메서드 | 경로 | 설명 |
|--------|------|------|
| GET | `/health` | 헬스체크 |
| POST | `/api/v1/auth/login` | OAuth2 로그인 |
| POST | `/api/v1/auth/refresh` | 토큰 갱신 |
| WS | `/api/v1/sync/connect` | CRDT 동기화 |
| GET | `/api/v1/license/verify` | 라이선스 확인 |
| GET | `/api/v1/models/catalog` | 모델 카탈로그 |

---

## 6. 모바일 빌드

### 6.1 사전 준비

**iOS:**
```bash
# Xcode 15+ 설치 필요
rustup target add aarch64-apple-ios
cargo tauri ios init
```

**Android:**
```bash
# Android Studio + NDK 설치 필요
rustup target add aarch64-linux-android
cargo tauri android init
```

### 6.2 빌드

```bash
# iOS
cargo tauri ios build

# Android
cargo tauri android build
```

### 6.3 모바일 빌드 Feature Flags

```bash
# 모바일은 desktop 대신 mobile feature 사용
cargo build -p voxnote-tauri --features stt,mobile,cloud-providers
```

---

## 7. WASM 빌드

```bash
# WASM 타겟 추가
rustup target add wasm32-unknown-unknown

# 빌드
cargo build -p voxnote-wasm --target wasm32-unknown-unknown --release

# wasm-pack 사용 시
cargo install wasm-pack
wasm-pack build crates/voxnote-wasm --target web
```

> WASM 빌드에서는 로컬 AI 추론 없이 `cloud-providers` feature만 사용합니다.

---

## 8. AI 모델 관리

### 8.1 모델 저장 경로

| OS | 경로 |
|----|------|
| macOS/Linux | `~/.voxnote/models/` |
| Windows | `%APPDATA%\.voxnote\models\` |

### 8.2 사용 가능한 모델

| 모델 | 크기 | RAM | GPU | 용도 |
|------|------|-----|-----|------|
| `ggml-tiny.bin` | 74 MB | 512 MB | 불필요 | 개발/테스트 |
| `ggml-base.bin` | 148 MB | 1 GB | 선택 | 기본 품질 |
| `ggml-small.bin` | 488 MB | 2 GB | 권장 | 균형 |
| `ggml-large-v3-turbo-q5_0.bin` | 850 MB | 3 GB | 필수 | 최고 품질 |

### 8.3 모델 다운로드 스크립트

```bash
#!/bin/bash
# scripts/download-models.sh

MODELS_DIR="${HOME}/.voxnote/models"
mkdir -p "$MODELS_DIR"

BASE_URL="https://huggingface.co/ggerganov/whisper.cpp/resolve/main"

download_model() {
    local name=$1
    local dest="${MODELS_DIR}/${name}"
    if [ -f "$dest" ]; then
        echo "✓ ${name} already exists"
        return
    fi
    echo "↓ Downloading ${name}..."
    curl -L -o "$dest" "${BASE_URL}/${name}" --progress-bar
    echo "✓ ${name} downloaded"
}

case "${1:-tiny}" in
    tiny)  download_model "ggml-tiny.bin" ;;
    base)  download_model "ggml-base.bin" ;;
    small) download_model "ggml-small.bin" ;;
    all)
        download_model "ggml-tiny.bin"
        download_model "ggml-base.bin"
        download_model "ggml-small.bin"
        ;;
    *)
        echo "Usage: $0 [tiny|base|small|all]"
        exit 1
        ;;
esac
```

### 8.4 모델 레지스트리

`models/registry.toml`에 모델 메타데이터가 정의되어 있습니다. 새 모델 추가 시 이 파일에 엔트리를 추가합니다.

---

## 9. Feature Flags

### 9.1 전체 Feature 목록

| Feature | 기본값 | 설명 | 의존성 |
|---------|-------|------|--------|
| `stt` | ON | whisper.cpp STT | `whisper-rs` |
| `llm` | OFF | llama.cpp LLM | — |
| `tts` | OFF | Piper TTS | `ort` |
| `diarize` | OFF | 화자 분리 | `ort` |
| `metal` | OFF | Apple Metal GPU | `whisper-rs/metal` |
| `cuda` | OFF | NVIDIA CUDA GPU | `whisper-rs/cuda` |
| `vulkan` | OFF | Vulkan GPU | — |
| `desktop` | ON | 데스크탑 오디오 캡처 | `cpal` |
| `mobile` | OFF | 모바일 최적화 | — |
| `wasm` | OFF | WebAssembly | `wasm-bindgen` |
| `sync` | OFF | CRDT 디바이스 동기화 | `yrs`, `tokio-tungstenite` |
| `cloud-providers` | OFF | 클라우드 API | `reqwest` |

### 9.2 빌드 프로필별 Feature 조합

```bash
# 개발 (기본)
--features stt,desktop

# 프로덕션 데스크탑 (macOS)
--features stt,llm,desktop,metal,sync,cloud-providers

# 프로덕션 데스크탑 (Windows/Linux CUDA)
--features stt,llm,desktop,cuda,sync,cloud-providers

# 모바일
--features stt,mobile,cloud-providers

# 서버
--features sync

# WASM
--features wasm,cloud-providers
```

---

## 10. 테스트

### 10.1 전체 테스트 실행

```bash
./scripts/test-all.sh --all
```

### 10.2 개별 실행

```bash
# Rust Core (feature 없이)
cargo test -p voxnote-core --no-default-features

# Rust Core (모든 feature)
cargo test -p voxnote-core --features stt,desktop

# Rust Server
cargo test -p voxnote-server

# Frontend
cd frontend && pnpm test

# TypeScript 타입 체크
cd frontend && npx tsc --noEmit

# Lint
cargo clippy --workspace --all-features -- -D warnings
cargo fmt --check
```

### 10.3 실제 모델 추론 테스트

```bash
# Whisper 모델이 ~/.voxnote/models/ggml-tiny.bin에 있어야 함
cargo test -p voxnote-core --features stt,desktop --test real_whisper_inference
cargo test -p voxnote-core --features stt,desktop --test real_e2e_full_pipeline
```

### 10.4 현재 테스트 현황

| 영역 | 테스트 수 | 상태 |
|------|----------|------|
| Rust Core (단위+통합) | 126 | 전체 통과 |
| Rust Core (실제 모델) | 12 | 전체 통과 |
| Rust Server | 13 | 전체 통과 |
| Frontend | 29 | 전체 통과 |
| **합계** | **168** | **0 failures** |

---

## 11. CI/CD 파이프라인

### 11.1 GitHub Actions 구성

```yaml
# .github/workflows/ci.yml
name: CI

on:
  push:
    branches: [main]
  pull_request:

jobs:
  test:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: pnpm/action-setup@v2
        with:
          version: 10
      - uses: actions/setup-node@v4
        with:
          node-version: 20

      # 의존성 캐시
      - uses: Swatinem/rust-cache@v2

      # Frontend
      - run: cd frontend && pnpm install && pnpm test && npx tsc --noEmit

      # Rust
      - run: cargo test -p voxnote-core --no-default-features
      - run: cargo test -p voxnote-server

  lint:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
        with:
          components: clippy, rustfmt
      - run: cargo clippy --workspace --all-features -- -D warnings
      - run: cargo fmt --check

  build:
    needs: [test, lint]
    strategy:
      matrix:
        include:
          - os: macos-14
            target: aarch64-apple-darwin
            features: stt,desktop,metal
          - os: macos-13
            target: x86_64-apple-darwin
            features: stt,desktop
          - os: windows-latest
            target: x86_64-pc-windows-msvc
            features: stt,desktop
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
            features: stt,desktop
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: pnpm/action-setup@v2
        with:
          version: 10
      - run: cd frontend && pnpm install && pnpm build
      - run: cargo tauri build --no-bundle
      - uses: actions/upload-artifact@v4
        with:
          name: voxnote-${{ matrix.target }}
          path: target/release/voxnote-tauri*
```

### 11.2 릴리즈 파이프라인

```yaml
# .github/workflows/release.yml
name: Release

on:
  push:
    tags: ['v*']

jobs:
  release:
    strategy:
      matrix:
        include:
          - os: macos-14
            target: aarch64-apple-darwin
          - os: macos-13
            target: x86_64-apple-darwin
          - os: windows-latest
            target: x86_64-pc-windows-msvc
          - os: ubuntu-latest
            target: x86_64-unknown-linux-gnu
    runs-on: ${{ matrix.os }}
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      - uses: pnpm/action-setup@v2
        with:
          version: 10
      - run: cd frontend && pnpm install && pnpm build
      - run: cargo tauri build

      - uses: softprops/action-gh-release@v1
        with:
          files: |
            target/release/bundle/**/*
        env:
          GITHUB_TOKEN: ${{ secrets.GITHUB_TOKEN }}
```

---

## 12. 릴리즈 배포

### 12.1 버전 관리

```bash
# 버전 업데이트 위치:
# 1. Cargo.toml (workspace.package.version)
# 2. crates/voxnote-tauri/tauri.conf.json (version)
# 3. frontend/package.json (version)
```

### 12.2 릴리즈 체크리스트

```
□ 모든 테스트 통과 (168/168)
□ clippy 경고 0개
□ cargo fmt 통과
□ TypeScript 타입 체크 통과
□ CHANGELOG 업데이트
□ 버전 번호 업데이트 (3개 파일)
□ git tag 생성
□ CI 빌드 성공 (4 플랫폼)
□ 바이너리 크기 < 30MB 확인
□ 모델 다운로드 정상 확인
□ 실제 녹음→전사 동작 확인
```

### 12.3 배포 산출물

| 플랫폼 | 파일 | 배포 채널 |
|--------|------|----------|
| macOS (ARM) | `VoxNote_x.x.x_aarch64.dmg` | GitHub Releases, 홈페이지 |
| macOS (Intel) | `VoxNote_x.x.x_x64.dmg` | GitHub Releases |
| Windows | `VoxNote_x.x.x_x64-setup.exe` | GitHub Releases |
| Windows | `VoxNote_x.x.x_x64.msi` | GitHub Releases |
| Linux | `VoxNote_x.x.x_amd64.AppImage` | GitHub Releases |
| Linux | `voxnote_x.x.x_amd64.deb` | GitHub Releases |

### 12.4 자동 업데이트

Tauri의 내장 업데이터를 사용합니다. `tauri.conf.json`에 업데이트 엔드포인트를 설정합니다:

```json
{
  "plugins": {
    "updater": {
      "endpoints": [
        "https://releases.voxnote.app/{{target}}/{{arch}}/{{current_version}}"
      ],
      "pubkey": "YOUR_PUBLIC_KEY"
    }
  }
}
```

---

## 13. 문제 해결

### 13.1 빌드 에러

**whisper-rs 컴파일 실패:**
```bash
# macOS: Accelerate.framework 필요 (Xcode 포함)
xcode-select --install

# Linux: cmake 필요
sudo apt install cmake
```

**Tauri 빌드 시 WebView 에러 (Linux):**
```bash
sudo apt install libwebkit2gtk-4.1-dev
```

**CUDA 빌드 실패:**
```bash
# CUDA_PATH 환경 변수 확인
echo $CUDA_PATH
# nvcc 확인
nvcc --version
```

### 13.2 런타임 에러

**"No audio device found":**
- 마이크 권한 확인 (macOS: 시스템 설정 → 개인 정보 보호 → 마이크)
- 오디오 디바이스 연결 확인

**"Model not found":**
```bash
# 모델 다운로드
curl -L -o ~/.voxnote/models/ggml-tiny.bin \
  "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"
```

**DB 초기화 실패:**
```bash
# 데이터 디렉토리 권한 확인
ls -la ~/.voxnote/data/
# 수동 초기화
mkdir -p ~/.voxnote/data
```

### 13.3 성능 참고

| 항목 | 목표 | 실측값 |
|------|------|--------|
| 바이너리 크기 | < 30MB | 9.4MB |
| 앱 시작 시간 | < 1초 | < 1초 |
| 모델 로드 (tiny) | < 5초 | < 1초 |
| 3초 오디오 추론 (tiny, CPU) | < 5초 | < 700ms |
| 100 노트 조회 | < 100ms | < 10ms |
| FTS5 검색 | < 50ms | < 5ms |
| 1MB 암호화+복호화 | — | ~100ms |

---

## 부록: 빠른 시작 요약

```bash
# 1. 의존성 설치
cargo install tauri-cli --version "^2"
cd frontend && pnpm install && cd ..

# 2. 모델 다운로드
mkdir -p ~/.voxnote/models
curl -L -o ~/.voxnote/models/ggml-tiny.bin \
  "https://huggingface.co/ggerganov/whisper.cpp/resolve/main/ggml-tiny.bin"

# 3. 테스트
cargo test -p voxnote-core --features stt,desktop
cd frontend && pnpm test && cd ..

# 4. 개발 실행
cd frontend && pnpm dev &
cargo tauri dev

# 5. 릴리즈 빌드
cd frontend && pnpm build && cd ..
cargo tauri build
```
