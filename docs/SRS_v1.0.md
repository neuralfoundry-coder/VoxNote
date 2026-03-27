# VoxNote 소프트웨어 요구사항 명세서 (SRS)

**버전:** 1.0
**작성일:** 2026-03-27
**기준 문서:** 로컬 AI 회의록 서비스 — 아키텍처 설계 및 구현 방법론 v1.0
**표준:** IEEE 830 / ISO/IEC/IEEE 29148 기반

---

## 목차

1. [소개](#1-소개)
2. [전체 설명](#2-전체-설명)
3. [기능 요구사항](#3-기능-요구사항)
4. [비기능 요구사항](#4-비기능-요구사항)
5. [인터페이스 요구사항](#5-인터페이스-요구사항)
6. [데이터 요구사항](#6-데이터-요구사항)
7. [추적 매트릭스](#7-추적-매트릭스)

---

## 1. 소개

### 1.1 목적

본 문서는 VoxNote 프로젝트의 소프트웨어 요구사항을 체계적으로 정의한다. 개발팀, 테스트팀, 이해관계자가 시스템의 기능과 제약사항을 명확히 이해하고, 개발 진행 및 검증의 기준으로 활용하는 것을 목적으로 한다.

### 1.2 범위

VoxNote는 **로컬 AI 기반 차세대 회의록 서비스**로, 음성을 텍스트로 변환(STT)하고 AI를 활용하여 회의록을 자동 요약·문서화하는 크로스플랫폼 애플리케이션이다. 핵심 AI 추론은 사용자 디바이스에서 로컬로 수행되며, 사용자 설정에 따라 외부 LLM Provider API도 자유롭게 활용할 수 있다.

### 1.3 용어 정의

| 용어 | 정의 |
|------|------|
| STT | Speech-to-Text, 음성을 텍스트로 변환하는 기술 |
| LLM | Large Language Model, 대규모 언어 모델 |
| TTS | Text-to-Speech, 텍스트를 음성으로 변환하는 기술 |
| VAD | Voice Activity Detection, 음성 구간 감지 |
| GGUF | GPT-Generated Unified Format, 양자화 모델 파일 형식 |
| CRDT | Conflict-free Replicated Data Type, 충돌 없는 분산 데이터 타입 |
| E2EE | End-to-End Encryption, 종단간 암호화 |
| RAG | Retrieval-Augmented Generation, 검색 증강 생성 |
| IPC | Inter-Process Communication, 프로세스 간 통신 |
| Provider | AI 모델 제공자 (로컬 모델 또는 외부 API 서비스) |
| 화자 분리 | Speaker Diarization, 음성에서 화자를 구분하는 기술 |
| FTS | Full-Text Search, 전문 검색 |

### 1.4 참고 문서

- VoxNote 아키텍처 설계 및 구현 방법론 v1.0
- Tauri 2.0 공식 문서
- whisper.cpp / llama.cpp 프로젝트 문서
- IEEE 830-1998 소프트웨어 요구사항 명세서 권장 관례

---

## 2. 전체 설명

### 2.1 제품 개요

VoxNote는 기존 클라우드 의존형 회의록 서비스(예: 티로)의 한계를 극복하기 위해 설계된 Privacy-First 회의록 서비스이다.

| 기존 서비스 한계 | VoxNote 해결 전략 |
|-----------------|------------------|
| 클라우드 의존 (AssemblyAI, OpenAI) | 100% 로컬 추론 기본 + 클라우드 API 선택적 사용 (opt-in) |
| 오프라인 시 기능 제한 | 완전한 오프라인 동작 (로컬 모델 기준) |
| 월 $7~29 구독 비용 | 일회성 구매 or 오픈소스 (서버 비용 ≈ 0) |
| AI 학습 금지 약속 (신뢰 기반) | 기술적으로 불가능한 구조 (로컬 모드 시 데이터 전송 자체 없음) |
| 웹/데스크탑/모바일 별도 개발 | 단일 Rust 코어 + Tauri 2.0 크로스플랫폼 |
| 제한된 언어 지원 | GGUF 양자화 모델 교체로 언어 무제한 확장 |
| 단일 AI Provider에 종속 | 로컬 모델 + 다수 외부 Provider 자유 선택 |

### 2.2 사용자 유형

| 사용자 유형 | 설명 |
|------------|------|
| **개인 사용자** | 개인 회의/강의 녹음 및 메모 용도. 로컬 모델 또는 클라우드 API 선택 사용 |
| **팀 사용자** | 팀 단위 회의록 공유, 공유 템플릿, 팀 폴더 활용 |
| **관리자** | 팀 설정 관리, 공유 템플릿 관리, 팀원 권한 관리 |

### 2.3 운영 환경

| 플랫폼 | 타겟 | GPU 가속 | 패키징 |
|--------|------|---------|--------|
| macOS (Apple Silicon) | `aarch64-apple-darwin` | Metal | `.dmg`, `.app` |
| macOS (Intel) | `x86_64-apple-darwin` | CPU only | `.dmg` |
| Windows | `x86_64-pc-windows-msvc` | CUDA / Vulkan | `.msi`, `.exe` |
| Linux | `x86_64-unknown-linux-gnu` | CUDA / Vulkan | `.AppImage`, `.deb`, `.rpm` |
| iOS | `aarch64-apple-ios` | CoreML | `.ipa` |
| Android | `aarch64-linux-android` | NNAPI / Vulkan | `.apk`, `.aab` |
| Web | `wasm32-unknown-unknown` | WebGPU | 정적 호스팅 |

### 2.4 설계 제약사항

| 원칙 | 설명 |
|------|------|
| LOCAL-FIRST | 모든 AI 추론은 기본적으로 디바이스 내에서 수행 |
| RUST-NATIVE | 코어 로직 100% Rust 구현 |
| ZERO-TRUST | 서버는 사용자 데이터에 접근 불가 |
| PLUGIN-ARCH | 모델 및 Provider 핫스왑 가능 |
| CROSS-PLAT | 단일 코드베이스, 7개 타겟 (데스크탑 3 + 모바일 2 + 웹 1 + WASM 1) |
| PRIVACY-HARD | 프라이버시를 기술적으로 보장 (로컬 모드 시) |
| PROVIDER-AGNOSTIC | 로컬 모델과 외부 API를 동일한 추상화 계층에서 처리 |

### 2.5 가정 및 의존성

- 데스크탑 사용자는 최소 8GB RAM을 보유한다고 가정
- GPU 가속은 선택사항이며, CPU만으로도 모든 기능이 동작해야 함
- 외부 LLM API 사용 시 사용자가 직접 API 키를 발급·관리함
- 네트워크가 없는 환경에서도 로컬 모델 기반 핵심 기능은 100% 동작
- Tauri 2.0의 모바일 지원이 안정적인 수준이라 가정

---

## 3. 기능 요구사항

> **우선순위 정의:**
> - **P0** — 필수 (MVP에 포함)
> - **P1** — 중요 (초기 릴리즈에 포함)
> - **P2** — 개선 (후속 릴리즈에 포함)

### 3.1 FR-AUD: 오디오 파이프라인

| ID | 요구사항 | 우선순위 | Phase |
|----|---------|---------|-------|
| FR-AUD-001 | 시스템은 마이크 입력을 실시간으로 캡처할 수 있어야 한다 (cpal 기반) | P0 | 1 |
| FR-AUD-002 | 캡처된 오디오를 16kHz mono f32 형식으로 리샘플링해야 한다 (rubato) | P0 | 1 |
| FR-AUD-003 | VAD(Voice Activity Detection)를 통해 음성 구간만 필터링해야 한다 | P0 | 1 |
| FR-AUD-004 | lock-free 링버퍼를 사용하여 캡처 스레드와 처리 스레드 간 데이터를 전달해야 한다 | P0 | 1 |
| FR-AUD-005 | macOS에서 ScreenCaptureKit/CoreAudio를 통한 시스템 사운드 캡처를 지원해야 한다 | P1 | 4 |
| FR-AUD-006 | Windows에서 WASAPI Loopback을 통한 시스템 사운드 캡처를 지원해야 한다 | P1 | 4 |
| FR-AUD-007 | Linux에서 PulseAudio/PipeWire Monitor를 통한 시스템 사운드 캡처를 지원해야 한다 | P1 | 4 |
| FR-AUD-008 | iOS에서 AVAudioEngine을 통한 오디오 캡처를 지원해야 한다 (Swift bridge) | P1 | 3 |
| FR-AUD-009 | Android에서 AudioRecord를 통한 오디오 캡처를 지원해야 한다 (Kotlin bridge) | P1 | 3 |
| FR-AUD-010 | 웹에서 MediaDevices.getUserMedia() + AudioWorklet을 통한 캡처를 지원해야 한다 | P2 | 3 |

### 3.2 FR-STT: 음성-텍스트 변환

| ID | 요구사항 | 우선순위 | Phase |
|----|---------|---------|-------|
| FR-STT-001 | whisper.cpp 기반으로 실시간 스트리밍 전사를 수행해야 한다 | P0 | 1 |
| FR-STT-002 | 2~3초 단위 슬라이딩 윈도우 + 0.5초 오버랩으로 연속성을 확보해야 한다 | P0 | 1 |
| FR-STT-003 | 직전 전사 결과를 initial_prompt로 주입하여 문맥 연속성을 유지해야 한다 | P0 | 1 |
| FR-STT-004 | 다국어 자동 감지 및 수동 언어 선택을 지원해야 한다 | P0 | 1 |
| FR-STT-005 | Metal(macOS), CUDA(Windows/Linux) GPU 가속을 지원해야 한다 | P0 | 1 |
| FR-STT-006 | tiny(40MB)부터 large-v3-turbo(850MB)까지 모델 선택이 가능해야 한다 | P0 | 1 |
| FR-STT-007 | 실시간 번역 모드를 지원해야 한다 (소스 언어 → 영어 등) | P1 | 2 |
| FR-STT-008 | 전사 결과를 Tauri event emit으로 프론트엔드에 실시간 스트리밍해야 한다 | P0 | 1 |
| FR-STT-009 | 녹음 파일(WAV 등) 업로드를 통한 일괄 전사를 지원해야 한다 | P1 | 2 |
| FR-STT-010 | 외부 STT API(OpenAI Whisper API, Google STT 등)를 대안으로 사용 가능해야 한다 | P1 | 2 |

### 3.3 FR-LLM: LLM 엔진

| ID | 요구사항 | 우선순위 | Phase |
|----|---------|---------|-------|
| FR-LLM-001 | llama.cpp 기반으로 로컬 LLM 추론을 수행해야 한다 | P0 | 2 |
| FR-LLM-002 | 1~2분 단위 전사 텍스트를 구조화된 실시간 요약으로 변환해야 한다 | P0 | 2 |
| FR-LLM-003 | 사용자 정의 템플릿을 기반으로 회의록 문서를 자동 생성해야 한다 | P0 | 2 |
| FR-LLM-004 | GBNF Grammar를 사용하여 JSON/Markdown 출력 형식을 강제할 수 있어야 한다 | P1 | 2 |
| FR-LLM-005 | RAG 방식으로 전사 텍스트에 대한 질의응답(Ask VoxNote)을 지원해야 한다 | P1 | 2 |
| FR-LLM-006 | 전사 텍스트의 다국어 번역을 지원해야 한다 | P1 | 2 |
| FR-LLM-007 | Streaming token output으로 생성 과정을 실시간 표시해야 한다 | P0 | 2 |
| FR-LLM-008 | GGUF 양자화 모델(Q4_K_M 등)을 지원하여 메모리 효율을 확보해야 한다 | P0 | 2 |
| FR-LLM-009 | LLM 세션 풀링으로 반복 호출 시 성능을 최적화해야 한다 | P1 | 2 |
| FR-LLM-010 | 전사 결과의 오탈자 교정을 LLM 경량 패스로 처리할 수 있어야 한다 | P1 | 2 |

### 3.4 FR-API: 외부 LLM Provider API 연동

| ID | 요구사항 | 우선순위 | Phase |
|----|---------|---------|-------|
| FR-API-001 | 설정 UI에서 AI Provider를 선택하고 API 키를 관리할 수 있어야 한다 | P0 | 2 |
| FR-API-002 | OpenAI API (GPT-4o, Whisper API, TTS 등)를 지원해야 한다 | P0 | 2 |
| FR-API-003 | Anthropic API (Claude 모델)를 지원해야 한다 | P0 | 2 |
| FR-API-004 | Google Gemini API를 지원해야 한다 | P1 | 2 |
| FR-API-005 | Groq API (고속 추론)를 지원해야 한다 | P2 | 3 |
| FR-API-006 | Ollama (사용자 자체 서버)를 OpenAI-compatible 엔드포인트로 지원해야 한다 | P1 | 2 |
| FR-API-007 | STT, LLM, TTS 각 엔진별로 독립적으로 Provider를 선택할 수 있어야 한다 | P0 | 2 |
| FR-API-008 | 로컬 모델과 클라우드 API를 동일한 내부 추상화 인터페이스(trait)로 통합해야 한다 | P0 | 2 |
| FR-API-009 | API 키는 OS 키체인(macOS Keychain, Windows Credential Manager, Linux Secret Service)에 암호화 보관해야 한다 | P0 | 2 |
| FR-API-010 | 네트워크 불가 시 자동으로 로컬 모델로 폴백해야 한다 | P1 | 2 |
| FR-API-011 | 사용자 정의 OpenAI-compatible 엔드포인트 URL을 설정할 수 있어야 한다 | P1 | 2 |
| FR-API-012 | API 사용량(토큰 수, 비용 추정)을 모니터링하고 표시해야 한다 | P2 | 3 |
| FR-API-013 | 새로운 Provider 추가가 용이한 플러그인 구조(trait 기반)로 설계해야 한다 | P0 | 2 |

### 3.5 FR-TTS: 음성 합성

| ID | 요구사항 | 우선순위 | Phase |
|----|---------|---------|-------|
| FR-TTS-001 | Piper TTS(ONNX 기반)로 로컬 음성 합성을 지원해야 한다 | P2 | 3 |
| FR-TTS-002 | 한국어, 영어, 일본어 음성 모델을 제공해야 한다 | P2 | 3 |
| FR-TTS-003 | CPU에서도 실시간 합성이 가능해야 한다 | P2 | 3 |
| FR-TTS-004 | 외부 TTS API(OpenAI TTS, Google TTS 등)를 대안으로 사용 가능해야 한다 | P2 | 3 |

### 3.6 FR-DIA: 화자 분리

| ID | 요구사항 | 우선순위 | Phase |
|----|---------|---------|-------|
| FR-DIA-001 | ECAPA-TDNN(ONNX) 기반 화자 임베딩 추출을 수행해야 한다 | P1 | 2 |
| FR-DIA-002 | 온라인 Agglomerative 클러스터링으로 실시간 화자 구분을 지원해야 한다 | P1 | 2 |
| FR-DIA-003 | 코사인 유사도 임계값(0.65~0.75)을 사용자 조정 가능해야 한다 | P2 | 2 |
| FR-DIA-004 | 화자 분리 결과를 전사 세그먼트에 태깅해야 한다 | P1 | 2 |
| FR-DIA-005 | 사용자가 화자에 이름을 할당할 수 있어야 한다 | P1 | 2 |

### 3.7 FR-STO: 로컬 저장소

| ID | 요구사항 | 우선순위 | Phase |
|----|---------|---------|-------|
| FR-STO-001 | SQLite(rusqlite, FTS5)를 사용하여 노트, 전사, 요약 데이터를 저장해야 한다 | P0 | 1 |
| FR-STO-002 | SQLite FTS5를 활용한 전문검색을 지원해야 한다 | P0 | 1 |
| FR-STO-003 | redb를 사용하여 오디오 청크 캐시 및 세션 상태를 저장해야 한다 | P1 | 1 |
| FR-STO-004 | age(X25519) 기반 파일 레벨 E2EE 암호화를 지원해야 한다 | P0 | 1 |
| FR-STO-005 | 사용자 단어장(고유명사, 전문 용어)을 관리할 수 있어야 한다 | P1 | 2 |
| FR-STO-006 | RAG용 텍스트 청크 임베딩을 BLOB으로 저장하고 검색할 수 있어야 한다 | P1 | 2 |
| FR-STO-007 | 노트를 폴더 구조로 정리할 수 있어야 한다 | P0 | 1 |

### 3.8 FR-SYN: 디바이스 간 동기화

| ID | 요구사항 | 우선순위 | Phase |
|----|---------|---------|-------|
| FR-SYN-001 | y-crdt 기반으로 디바이스 간 노트 동기화를 지원해야 한다 | P1 | 3 |
| FR-SYN-002 | 동기화 시 CRDT 델타를 age(X25519)로 암호화하여 전송해야 한다 | P1 | 3 |
| FR-SYN-003 | 서버는 암호화된 바이너리 델타만 중계하며, 평문 접근이 불가해야 한다 | P1 | 3 |
| FR-SYN-004 | 오프라인 디바이스를 위한 임시 버퍼링(TTL 30일)을 지원해야 한다 | P1 | 3 |
| FR-SYN-005 | WebSocket 기반 실시간 동기화를 지원해야 한다 | P1 | 3 |

### 3.9 FR-MDL: 모델 관리

| ID | 요구사항 | 우선순위 | Phase |
|----|---------|---------|-------|
| FR-MDL-001 | 모델 카탈로그(registry.toml)를 조회하고 모델을 선택할 수 있어야 한다 | P0 | 1 |
| FR-MDL-002 | CDN에서 모델을 청크 단위로 다운로드할 수 있어야 한다 (재개 가능) | P0 | 1 |
| FR-MDL-003 | 다운로드 진행률을 UI에 실시간 표시해야 한다 | P0 | 1 |
| FR-MDL-004 | SHA-256 기반 모델 무결성 검증을 수행해야 한다 | P0 | 1 |
| FR-MDL-005 | 사용자 VRAM/RAM에 맞는 모델을 자동 추천해야 한다 | P1 | 2 |
| FR-MDL-006 | 실행 중 모델 핫스왑(기존 모델 언로드 → 새 모델 로드)을 지원해야 한다 | P1 | 2 |
| FR-MDL-007 | 사용자가 커스텀 GGUF 모델을 직접 로드할 수 있어야 한다 | P2 | 2 |

### 3.10 FR-SRV: 서버 기능

| ID | 요구사항 | 우선순위 | Phase |
|----|---------|---------|-------|
| FR-SRV-001 | OAuth2 OIDC (Google/Apple) 기반 로그인을 지원해야 한다 | P1 | 3 |
| FR-SRV-002 | JWT(RS256) 기반 인증 토큰 발급 및 갱신을 지원해야 한다 | P1 | 3 |
| FR-SRV-003 | 라이선스 키 검증 및 디바이스 활성화/비활성화를 지원해야 한다 | P1 | 3 |
| FR-SRV-004 | FCM/APNs 기반 푸시 알림 등록 및 전송을 지원해야 한다 | P2 | 4 |
| FR-SRV-005 | 모델 파일을 Cloudflare R2/S3에서 CDN으로 배포해야 한다 | P0 | 1 |
| FR-SRV-006 | CRDT 동기화 릴레이 서버(암호화 델타 중계)를 제공해야 한다 | P1 | 3 |
| FR-SRV-007 | Rate limiting 미들웨어를 적용해야 한다 | P1 | 3 |
| FR-SRV-008 | 사용자 프로필 조회/수정/삭제를 지원해야 한다 | P1 | 3 |

### 3.11 FR-UI: 사용자 인터페이스

| ID | 요구사항 | 우선순위 | Phase |
|----|---------|---------|-------|
| FR-UI-001 | 실시간 전사 뷰에서 전사 텍스트가 스트리밍으로 표시되어야 한다 | P0 | 1 |
| FR-UI-002 | 노트 에디터에서 전사 텍스트를 편집할 수 있어야 한다 | P0 | 1 |
| FR-UI-003 | 요약 패널에서 AI 요약 결과를 확인하고 편집할 수 있어야 한다 | P0 | 2 |
| FR-UI-004 | 모델 매니저에서 모델 다운로드/삭제/교체를 관리할 수 있어야 한다 | P0 | 2 |
| FR-UI-005 | Ask VoxNote에서 회의 내용에 대한 질의응답을 수행할 수 있어야 한다 | P1 | 2 |
| FR-UI-006 | 설정 화면에서 앱 전반의 설정을 관리할 수 있어야 한다 | P0 | 1 |
| FR-UI-007 | **AI Provider 설정 패널에서 Provider 선택, API 키 입력, 엔진별 Provider 매핑을 관리할 수 있어야 한다** | P0 | 2 |
| FR-UI-008 | 녹음 시작/중지 컨트롤과 상태 표시를 제공해야 한다 | P0 | 1 |
| FR-UI-009 | 화자별 전사 텍스트를 색상/라벨로 구분 표시해야 한다 | P1 | 2 |
| FR-UI-010 | 시스템 트레이 아이콘 및 빠른 녹음 시작을 지원해야 한다 | P1 | 1 |
| FR-UI-011 | React/Solid + TypeScript + TailwindCSS 기반 프론트엔드를 구현해야 한다 | P0 | 1 |

### 3.12 FR-MOB: 모바일 전략

| ID | 요구사항 | 우선순위 | Phase |
|----|---------|---------|-------|
| FR-MOB-001 | **Tier 1 (온디바이스):** Whisper tiny/base + Qwen2.5-1.5B로 기본 기능을 제공해야 한다 | P1 | 3 |
| FR-MOB-002 | **Tier 2 (LAN 위임):** 같은 네트워크의 VoxNote 데스크탑에 mDNS 자동 디스커버리 후 E2EE 터널로 대형 모델을 위임할 수 있어야 한다 | P2 | 3 |
| FR-MOB-003 | **Tier 3 (클라우드 opt-in):** 사용자가 직접 설정한 API 엔드포인트(예: Ollama 인스턴스)를 사용할 수 있어야 한다 | P1 | 3 |
| FR-MOB-004 | **클라우드 API 우선 옵션:** 모바일에서 외부 LLM Provider API를 기본 엔진으로 설정할 수 있어야 한다 | P1 | 3 |
| FR-MOB-005 | iOS/Android에서 Tauri Mobile 기반 네이티브 앱을 제공해야 한다 | P1 | 3 |

### 3.13 FR-EXT: 외부 연동

| ID | 요구사항 | 우선순위 | Phase |
|----|---------|---------|-------|
| FR-EXT-001 | Notion으로 회의록 내보내기를 지원해야 한다 | P2 | 4 |
| FR-EXT-002 | Slack으로 회의 요약 공유를 지원해야 한다 | P2 | 4 |
| FR-EXT-003 | Confluence로 회의록 내보내기를 지원해야 한다 | P2 | 4 |
| FR-EXT-004 | Markdown/PDF/DOCX 형식으로 문서 내보내기를 지원해야 한다 | P1 | 2 |

---

## 4. 비기능 요구사항

### 4.1 NFR-PERF: 성능

| ID | 요구사항 | 목표치 | 조건 |
|----|---------|--------|------|
| NFR-PERF-001 | 전사 지연 (발화 → 텍스트 표시) | < 300ms | 로컬 GPU 활용 시 |
| NFR-PERF-002 | 앱 시작 시간 | < 1초 | 네이티브 Tauri |
| NFR-PERF-003 | 메모리 사용량 (기본 상태) | < 500MB | STT 모델 미로드 시 |
| NFR-PERF-004 | 바이너리 크기 (모델 미포함) | < 30MB | Tauri 경량성 |
| NFR-PERF-005 | 1시간 회의 STT (파일 업로드) | < 8분 (CPU) / < 2분 (GPU) | large-v3-turbo 기준 |
| NFR-PERF-006 | 요약 생성 속도 | < 10초 | 7B 로컬 모델 기준 |
| NFR-PERF-007 | 외부 API 응답 시간 | Provider 의존 | 네트워크 상태에 따라 상이 |

### 4.2 NFR-SEC: 보안 및 프라이버시

| ID | 요구사항 |
|----|---------|
| NFR-SEC-001 | 로컬 모드에서는 어떤 사용자 데이터도 디바이스 외부로 전송되지 않아야 한다 |
| NFR-SEC-002 | 모든 저장 데이터는 age(X25519) 기반 E2EE로 암호화되어야 한다 |
| NFR-SEC-003 | 동기화 시 서버는 평문 데이터에 접근할 수 없어야 한다 |
| NFR-SEC-004 | API 키는 OS 키체인에 암호화 저장되어야 한다 (평문 저장 금지) |
| NFR-SEC-005 | 키 파생에 Argon2를 사용해야 한다 |
| NFR-SEC-006 | 스트리밍 암호화에 ChaCha20-Poly1305를 사용해야 한다 |
| NFR-SEC-007 | 외부 API 사용 시 전송 데이터 범위를 사용자에게 명확히 고지해야 한다 |
| NFR-SEC-008 | 릴리즈 전 보안 감사 및 퍼징 테스트를 수행해야 한다 |

### 4.3 NFR-PLAT: 크로스플랫폼 호환성

| ID | 플랫폼 | 최소 CPU | 최소 RAM | 최소 저장공간 | GPU (선택) |
|----|--------|---------|---------|------------|-----------|
| NFR-PLAT-001 | macOS | Apple M1+ | 8GB | 5GB | Metal (내장) |
| NFR-PLAT-002 | Windows | 4코어 이상 | 8GB | 5GB | CUDA (GTX 1060+) |
| NFR-PLAT-003 | Linux | 4코어 이상 | 8GB | 5GB | CUDA / Vulkan |
| NFR-PLAT-004 | iOS | A15+ (iPhone 13+) | — | 2GB | CoreML |
| NFR-PLAT-005 | Android | Snapdragon 8 Gen 1+ | 6GB | 2GB | NNAPI |

### 4.4 NFR-REL: 신뢰성

| ID | 요구사항 |
|----|---------|
| NFR-REL-001 | 오프라인 환경에서 로컬 모델 기반 핵심 기능(녹음, 전사, 요약)이 100% 동작해야 한다 |
| NFR-REL-002 | 녹음 중 앱 강제 종료 시 기존 전사 데이터가 유실되지 않아야 한다 |
| NFR-REL-003 | CRDT 기반 동기화로 디바이스 간 충돌이 자동 해결되어야 한다 |
| NFR-REL-004 | 모델 다운로드 중 네트워크 단절 시 재개(resume)가 가능해야 한다 |
| NFR-REL-005 | 외부 API 장애 시 로컬 모델로 자동 폴백해야 한다 |

### 4.5 NFR-USAB: 사용성

| ID | 요구사항 |
|----|---------|
| NFR-USAB-001 | 최초 실행 시 사용자 시스템 사양에 맞는 모델을 자동 추천해야 한다 |
| NFR-USAB-002 | 녹음 시작까지 3회 이내 클릭으로 도달 가능해야 한다 |
| NFR-USAB-003 | 회의 내용 음성 재생(TTS)으로 접근성을 지원해야 한다 |
| NFR-USAB-004 | Provider 설정이 직관적이어야 하며, 설정 미완료 시 로컬 모델이 기본값이어야 한다 |

### 4.6 NFR-MAINT: 유지보수성

| ID | 요구사항 |
|----|---------|
| NFR-MAINT-001 | Cargo 워크스페이스로 모듈을 분리해야 한다 (core, tauri, server, wasm) |
| NFR-MAINT-002 | feature flag를 통해 기능별 조건부 컴파일을 지원해야 한다 |
| NFR-MAINT-003 | 플랫폼별 코드는 `#[cfg]` 조건부 컴파일로 분리해야 한다 |
| NFR-MAINT-004 | CI/CD에서 macOS/Windows/Linux/iOS/Android 매트릭스 빌드를 자동화해야 한다 |

### 4.7 NFR-EXT: 확장성

| ID | 요구사항 |
|----|---------|
| NFR-EXT-001 | LLM Provider는 Rust trait 기반 플러그인 구조로 설계하여 새 Provider 추가가 용이해야 한다 |
| NFR-EXT-002 | STT/LLM/TTS 엔진은 동일한 추상화 인터페이스를 공유하여 로컬/클라우드 구현을 교체 가능해야 한다 |
| NFR-EXT-003 | 모델 레지스트리(registry.toml)에 새 모델을 추가하는 것만으로 모델 확장이 가능해야 한다 |
| NFR-EXT-004 | OpenAI-compatible API 엔드포인트를 지원하는 모든 서비스와 호환 가능해야 한다 |

---

## 5. 인터페이스 요구사항

### 5.1 Tauri IPC 인터페이스

프론트엔드(WebView)와 Rust 코어 간 통신은 Tauri IPC(`invoke`/`listen`)를 사용한다.

| 커맨드 | 방향 | 설명 |
|--------|------|------|
| `start_recording` | Frontend → Rust | 녹음 시작, session_id 반환 |
| `stop_recording` | Frontend → Rust | 녹음 종료, note_id 반환 |
| `generate_summary` | Frontend → Rust | 요약 생성 요청 |
| `stt:segment` | Rust → Frontend | 실시간 전사 세그먼트 이벤트 |
| `list_models` | Frontend → Rust | 사용 가능한 모델 목록 조회 |
| `download_model` | Frontend → Rust | 모델 다운로드 시작 |
| `switch_provider` | Frontend → Rust | AI Provider 전환 |
| `get_provider_config` | Frontend → Rust | 현재 Provider 설정 조회 |
| `set_provider_config` | Frontend → Rust | Provider 설정 저장 |
| `ask_voxnote` | Frontend → Rust | RAG 기반 질의응답 |

### 5.2 서버 REST API

| 메서드 | 경로 | 설명 |
|--------|------|------|
| POST | `/api/v1/auth/login` | OAuth2 OIDC 로그인 |
| POST | `/api/v1/auth/refresh` | JWT 갱신 |
| DELETE | `/api/v1/auth/logout` | 세션 무효화 |
| GET | `/api/v1/license/verify` | 라이선스 검증 |
| POST | `/api/v1/license/activate` | 디바이스 활성화 |
| DELETE | `/api/v1/license/deactivate` | 디바이스 비활성화 |
| POST | `/api/v1/notifications/register` | 푸시 토큰 등록 |
| PUT | `/api/v1/notifications/settings` | 알림 설정 |
| WS | `/api/v1/sync/connect` | CRDT 동기화 WebSocket |
| GET | `/api/v1/sync/status` | 동기화 상태 확인 |
| GET | `/api/v1/models/catalog` | 모델 카탈로그 조회 |
| GET | `/api/v1/models/:id/download` | 모델 다운로드 (Signed URL) |
| GET | `/api/v1/user/profile` | 프로필 조회 |
| PUT | `/api/v1/user/profile` | 프로필 수정 |
| DELETE | `/api/v1/user/account` | 계정 삭제 |

### 5.3 플랫폼 네이티브 브릿지

| 플랫폼 | 브릿지 | 용도 |
|--------|--------|------|
| iOS | Swift (Tauri mobile plugin) | AVAudioEngine 오디오 캡처 |
| Android | Kotlin (Tauri mobile plugin) | AudioRecord 오디오 캡처 |
| Web | JS → Tauri IPC | MediaDevices API 오디오 캡처 |

### 5.4 LLM Provider API 인터페이스

시스템은 다음 외부 API 형식을 지원해야 한다:

| Provider | API 형식 | 지원 엔진 |
|----------|---------|----------|
| OpenAI | OpenAI Chat Completions / Whisper / TTS API | STT, LLM, TTS |
| Anthropic | Anthropic Messages API | LLM |
| Google Gemini | Gemini API | LLM |
| Groq | OpenAI-compatible API | LLM |
| Ollama | OpenAI-compatible API (로컬 서버) | LLM |
| 사용자 정의 | OpenAI-compatible 엔드포인트 | LLM, STT |

---

## 6. 데이터 요구사항

### 6.1 SQLite 스키마

```sql
-- 노트
CREATE TABLE notes (
    id          TEXT PRIMARY KEY,
    title       TEXT NOT NULL,
    created_at  TEXT NOT NULL,
    updated_at  TEXT NOT NULL,
    folder_id   TEXT REFERENCES folders(id)
);

-- 전사 텍스트
CREATE TABLE transcripts (
    id            TEXT PRIMARY KEY,
    note_id       TEXT NOT NULL REFERENCES notes(id),
    timestamp_ms  INTEGER NOT NULL,
    text          TEXT NOT NULL,
    speaker_id    TEXT,
    confidence    REAL
);

-- 요약
CREATE TABLE summaries (
    id          TEXT PRIMARY KEY,
    note_id     TEXT NOT NULL REFERENCES notes(id),
    template_id TEXT,
    content     TEXT NOT NULL,
    model_used  TEXT NOT NULL          -- 사용된 모델/Provider 식별
);

-- 사용자 단어장
CREATE TABLE vocabulary (
    id          TEXT PRIMARY KEY,
    term        TEXT NOT NULL,
    replacement TEXT NOT NULL,
    domain      TEXT
);

-- 임베딩 (RAG용)
CREATE TABLE embeddings (
    id        TEXT PRIMARY KEY,
    note_id   TEXT NOT NULL REFERENCES notes(id),
    chunk_idx INTEGER NOT NULL,
    vector    BLOB NOT NULL
);

-- 전문검색 인덱스
CREATE VIRTUAL TABLE transcript_fts USING fts5(text, content=transcripts);

-- Provider 설정
CREATE TABLE provider_config (
    id          TEXT PRIMARY KEY,
    engine_type TEXT NOT NULL,         -- 'stt' | 'llm' | 'tts'
    provider    TEXT NOT NULL,         -- 'local' | 'openai' | 'anthropic' | ...
    model_id    TEXT NOT NULL,
    endpoint    TEXT,                  -- 사용자 정의 엔드포인트 (nullable)
    is_active   INTEGER DEFAULT 1
);
```

### 6.2 모델 레지스트리 구조

```toml
[[models]]
id = "whisper-large-v3-turbo-q5"
name = "Whisper Large V3 Turbo"
type = "stt"                           # stt | llm | tts | diarization
size_bytes = 891289600
quantization = "Q5_0"
languages = ["auto", "ko", "en", "ja", "zh", "es", "fr", "de"]
min_ram_mb = 3072
gpu_recommended = true
download_url = "https://cdn.voxnote.app/models/whisper/..."
sha256 = "abc123..."
```

---

## 7. 추적 매트릭스

### 요구사항 ID ↔ 로드맵 Phase 매핑

| Phase | 기간 | 핵심 요구사항 |
|-------|------|-------------|
| **Phase 1: Foundation** | Month 1~3 | FR-AUD-001~004, FR-STT-001~006/008, FR-STO-001~004/007, FR-MDL-001~004, FR-UI-001/002/006/008/010/011, FR-SRV-005 |
| **Phase 2: Intelligence** | Month 4~6 | FR-LLM-001~010, FR-API-001~013, FR-DIA-001~005, FR-STT-007/009/010, FR-STO-005/006, FR-MDL-005~007, FR-UI-003~005/007/009, FR-EXT-004 |
| **Phase 3: Platform** | Month 7~9 | FR-AUD-008~010, FR-MOB-001~005, FR-SYN-001~005, FR-SRV-001~003/006~008, FR-TTS-001~004, FR-API-005 |
| **Phase 4: Polish** | Month 10~12 | FR-AUD-005~007, FR-SRV-004, FR-EXT-001~003, FR-API-012 |

### 우선순위 요약

| 우선순위 | 요구사항 수 | 설명 |
|----------|-----------|------|
| P0 | 38 | MVP 필수 기능 |
| P1 | 39 | 초기 릴리즈 포함 |
| P2 | 16 | 후속 릴리즈 |

---

*본 문서는 VoxNote 프로젝트의 아키텍처 설계서 v1.0을 기반으로 작성되었으며, 개발 진행에 따라 업데이트된다.*
