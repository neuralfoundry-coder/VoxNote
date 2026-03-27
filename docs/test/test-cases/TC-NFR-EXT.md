# TC-NFR-EXT: 확장성 비기능 요구사항 테스트 케이스

| 항목 | 내용 |
|------|------|
| **문서 ID** | TC-NFR-EXT |
| **SRS 참조** | VoxNote SRS v1.0 - NFR-EXT (NFR-EXT-001 ~ NFR-EXT-004) |
| **작성일** | 2026-03-27 |
| **상태** | 초안 |
| **테스트 코드 위치** | `tests/nfr/ext/`, `crates/voxnote-core/tests/extensibility/` |

## 테스트 요약

| 테스트 유형 | 개수 |
|-------------|------|
| Integration | 5 |
| 구조 검증 (정적 분석) | 3 |
| Unit | 2 |
| E2E | 2 |
| **합계** | **12** |

---

## NFR-EXT-001: trait 기반 Provider 플러그인 아키텍처 - P1

### TC-NFR-EXT-001-01: Provider trait 인터페이스 정의 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-EXT-001-01 |
| **요구사항 ID** | NFR-EXT-001 |
| **테스트 유형** | 구조 검증 (정적 분석) |
| **사전 조건** | - 소스 코드 접근 가능<br>- Provider trait 정의 확인 |
| **테스트 절차** | 1. `SttProvider` trait 정의 확인:<br>  - `fn transcribe(&self, audio: &AudioChunk) -> Result<Transcript>`<br>  - `fn supported_languages(&self) -> Vec<Language>`<br>  - `fn name(&self) -> &str`<br>2. `LlmProvider` trait 정의 확인:<br>  - `fn summarize(&self, text: &str) -> Result<Summary>`<br>  - `fn stream_generate(&self, prompt: &str) -> Result<TokenStream>`<br>3. `TtsProvider` trait 정의 확인<br>4. trait 객체 안전성 (object safety) 확인<br>5. `async` trait 지원 확인 |
| **기대 결과** | - STT/LLM/TTS 각각 독립 trait 정의<br>- trait이 object-safe (동적 디스패치 가능)<br>- 모든 Provider 메서드에 `Result` 반환 (에러 처리)<br>- async 메서드 지원 (`async-trait` 또는 네이티브)<br>- trait 문서화 (rustdoc) 완비 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/extensibility/test_provider_trait.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | 전체 |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-EXT-001-02: 커스텀 Provider 구현 및 등록

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-EXT-001-02 |
| **요구사항 ID** | NFR-EXT-001 |
| **테스트 유형** | Integration |
| **사전 조건** | - Provider trait 정의 확인<br>- 테스트용 Mock Provider 구현 준비 |
| **테스트 절차** | 1. `MockSttProvider` 구현 (SttProvider trait 구현체)<br>2. Provider 레지스트리에 Mock Provider 등록<br>3. 등록된 Provider를 이름으로 조회<br>4. Mock Provider를 통한 전사 요청 실행<br>5. Provider 해제 및 교체 동작 확인 |
| **기대 결과** | - Mock Provider trait 구현 컴파일 성공<br>- 레지스트리 등록/조회 정상<br>- Mock Provider를 통한 전사 정상 동작<br>- 런타임 Provider 교체 가능<br>- Provider 등록 코드 < 50줄 (간결한 인터페이스) |
| **테스트 코드 위치** | `crates/voxnote-core/tests/extensibility/test_custom_provider.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-EXT-001-03: 다중 Provider 동시 등록 및 선택

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-EXT-001-03 |
| **요구사항 ID** | NFR-EXT-001 |
| **테스트 유형** | Integration |
| **사전 조건** | - 여러 Provider 구현체 준비<br>  (WhisperLocal, OpenAiStt, GroqStt 등) |
| **테스트 절차** | 1. 3개 이상의 STT Provider 동시 등록<br>2. 이름 기반 Provider 선택 동작 확인<br>3. 기본 Provider 설정 및 폴백 체인 구성<br>4. Provider A 실패 시 Provider B 자동 전환 확인<br>5. UI에서 Provider 목록 표시 및 전환 확인 |
| **기대 결과** | - 다중 Provider 동시 등록 정상<br>- 이름 기반 정확한 선택<br>- 폴백 체인: A → B → C (로컬) 순서 동작<br>- Provider 전환 시 진행 중 작업 안전 중단<br>- UI Provider 목록에 등록된 모든 Provider 표시 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/extensibility/test_multi_provider.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-EXT-002: 엔진 추상화 인터페이스 - P1

### TC-NFR-EXT-002-01: STT 엔진 추상화 레이어 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-EXT-002-01 |
| **요구사항 ID** | NFR-EXT-002 |
| **테스트 유형** | 구조 검증 (정적 분석) |
| **사전 조건** | - STT 엔진 추상화 코드 접근 가능 |
| **테스트 절차** | 1. `SttEngine` 추상화 trait 정의 확인<br>2. 구현체 확인:<br>  - `WhisperCppEngine` (whisper.cpp 바인딩)<br>  - `WhisperRsEngine` (whisper-rs, 향후)<br>  - `ExternalApiEngine` (외부 API 래퍼)<br>3. 엔진 초기화 인터페이스 통일성 확인<br>4. 엔진 교체 시 상위 코드 변경 없음 확인<br>5. 엔진별 설정 파라미터 추상화 확인 |
| **기대 결과** | - 통일된 `SttEngine` 인터페이스<br>- 엔진 교체 시 설정 변경만으로 가능 (코드 변경 불필요)<br>- 엔진별 고유 설정은 `EngineConfig` enum으로 처리<br>- 새 엔진 추가: trait 구현 + 레지스트리 등록만 필요 |
| **테스트 코드 위치** | `crates/voxnote-stt/tests/test_engine_abstraction.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | 전체 |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-EXT-002-02: LLM 엔진 교체 테스트

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-EXT-002-02 |
| **요구사항 ID** | NFR-EXT-002 |
| **테스트 유형** | Integration |
| **사전 조건** | - 로컬 LLM 엔진 (llama.cpp 기반) 준비<br>- 외부 API 엔진 (OpenAI, Anthropic) 모의 서버 준비 |
| **테스트 절차** | 1. 로컬 LLM 엔진으로 요약 생성<br>2. 외부 API 엔진으로 동일 텍스트 요약 생성<br>3. 두 엔진의 인터페이스 호환성 확인 (호출 코드 동일)<br>4. 런타임 엔진 교체 후 즉시 사용 가능 확인<br>5. 엔진 교체 시 기존 설정/기록 보존 확인 |
| **기대 결과** | - 두 엔진 모두 동일한 `LlmEngine` trait 구현<br>- 호출 코드 변경 없이 엔진 교체 가능<br>- 런타임 교체 < 1초 (모델 로드 제외)<br>- 이전 엔진의 결과 기록 보존 |
| **테스트 코드 위치** | `crates/voxnote-llm/tests/test_engine_swap.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-EXT-002-03: 엔진 추상화를 통한 A/B 테스트 지원

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-EXT-002-03 |
| **요구사항 ID** | NFR-EXT-002 |
| **테스트 유형** | Integration |
| **사전 조건** | - 2개 이상의 STT 엔진 등록<br>- A/B 테스트 프레임워크 준비 |
| **테스트 절차** | 1. 동일 오디오를 엔진 A (whisper.cpp tiny)와 엔진 B (whisper.cpp base)로 전사<br>2. 각 엔진의 결과를 독립적으로 수집<br>3. WER/CER 비교<br>4. 처리 시간 비교<br>5. 메모리 사용량 비교 |
| **기대 결과** | - 두 엔진 결과 독립 수집 성공<br>- 비교 리포트 자동 생성<br>- 엔진 추상화 덕분에 비교 코드 최소화<br>- 사용자가 결과 기반 엔진 선택 가능 |
| **테스트 코드 위치** | `crates/voxnote-stt/tests/test_engine_ab_test.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-EXT-003: registry.toml 모델 확장 - P1

### TC-NFR-EXT-003-01: registry.toml 모델 정의 및 파싱 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-EXT-003-01 |
| **요구사항 ID** | NFR-EXT-003 |
| **테스트 유형** | Unit |
| **사전 조건** | - `registry.toml` 파일 존재<br>- TOML 파서 준비 |
| **테스트 절차** | 1. 기본 `registry.toml` 파싱 성공 확인<br>2. 모델 항목 필수 필드 확인:<br>  - `name`: 모델 이름<br>  - `url`: 다운로드 URL<br>  - `sha256`: 체크섬<br>  - `size`: 파일 크기<br>  - `engine`: 호환 엔진<br>  - `languages`: 지원 언어<br>3. 선택 필드 확인 (description, min_ram, gpu_required)<br>4. 잘못된 TOML 구문 에러 처리 확인<br>5. 필수 필드 누락 시 에러 메시지 확인 |
| **기대 결과** | - 유효한 `registry.toml` 파싱 성공<br>- 모든 필수 필드 검증 통과<br>- 잘못된 구문 시 명확한 에러 메시지 (줄 번호 포함)<br>- 필수 필드 누락 시 구체적인 안내 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/extensibility/test_registry_parse.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | 전체 |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-EXT-003-02: 사용자 정의 모델 추가 (registry.toml 확장)

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-EXT-003-02 |
| **요구사항 ID** | NFR-EXT-003 |
| **테스트 유형** | Integration |
| **사전 조건** | - 기본 `registry.toml` 로드 상태<br>- 커스텀 GGUF 모델 파일 준비 |
| **테스트 절차** | 1. 사용자 `registry.local.toml` 파일 작성:<br>  ```toml<br>  [[models]]<br>  name = "my-custom-whisper"<br>  path = "/path/to/custom-model.bin"<br>  engine = "whisper-cpp"<br>  languages = ["ko", "en"]<br>  ```<br>2. 앱 재시작 없이 모델 목록 갱신 확인<br>3. 커스텀 모델 선택 및 전사 실행<br>4. 기본 레지스트리 + 사용자 레지스트리 병합 확인<br>5. 이름 충돌 시 사용자 설정 우선 확인 |
| **기대 결과** | - 사용자 레지스트리 자동 인식 및 로드<br>- 커스텀 모델이 모델 목록에 표시<br>- 커스텀 모델로 전사 정상 동작<br>- 이름 충돌 시 사용자 정의 우선<br>- 잘못된 경로 시 친절한 에러 메시지 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/extensibility/test_registry_custom.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-EXT-003-03: 원격 레지스트리 업데이트 및 새 모델 알림

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-EXT-003-03 |
| **요구사항 ID** | NFR-EXT-003 |
| **테스트 유형** | E2E |
| **사전 조건** | - 원격 레지스트리 서버 (또는 Mock) 준비<br>- 현재 로컬 레지스트리 버전 기록 |
| **테스트 절차** | 1. 앱 시작 시 원격 레지스트리 버전 확인<br>2. 새 모델이 추가된 원격 레지스트리 감지<br>3. "새 모델 사용 가능" 알림 표시 확인<br>4. 원격 레지스트리 다운로드 및 로컬 병합<br>5. 새 모델 다운로드 및 사용 가능 확인 |
| **기대 결과** | - 원격 레지스트리 업데이트 자동 확인 (1일 1회)<br>- 새 모델 알림 비침입적 표시<br>- 레지스트리 병합 후 기존 설정 보존<br>- 오프라인 시 로컬 레지스트리만 사용 (에러 없음) |
| **테스트 코드 위치** | `tests/nfr/ext/registry_remote_update.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-EXT-004: OpenAI-compatible API 호환 - P1

### TC-NFR-EXT-004-01: OpenAI API 호환 엔드포인트 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-EXT-004-01 |
| **요구사항 ID** | NFR-EXT-004 |
| **테스트 유형** | Integration |
| **사전 조건** | - OpenAI 호환 API 클라이언트 구현 완료<br>- 다양한 OpenAI-compatible 서버 준비<br>  (Ollama, LM Studio, vLLM, LocalAI) |
| **테스트 절차** | 1. OpenAI 공식 API 엔드포인트로 요청 테스트<br>  - `POST /v1/chat/completions`<br>  - `POST /v1/audio/transcriptions`<br>2. Ollama 호환 엔드포인트로 동일 요청<br>3. LM Studio 호환 엔드포인트로 동일 요청<br>4. 각 서버의 응답 포맷 파싱 확인<br>5. 스트리밍 응답 (SSE) 처리 확인 |
| **기대 결과** | - OpenAI 공식 API 정상 동작<br>- Ollama 호환 정상 동작<br>- LM Studio 호환 정상 동작<br>- 응답 JSON 스키마 일관되게 파싱<br>- SSE 스트리밍 정상 처리 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/extensibility/test_openai_compat.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-EXT-004-02: 커스텀 base_url 설정 및 인증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-EXT-004-02 |
| **요구사항 ID** | NFR-EXT-004 |
| **테스트 유형** | Unit |
| **사전 조건** | - OpenAI 호환 클라이언트 구현 완료 |
| **테스트 절차** | 1. 기본 base_url (`https://api.openai.com/v1`) 테스트<br>2. 커스텀 base_url (`http://localhost:11434/v1`) 설정 테스트<br>3. API 키 인증 헤더 (`Authorization: Bearer ...`) 확인<br>4. API 키 없는 로컬 서버 (Ollama) 접속 테스트<br>5. 잘못된 base_url 시 에러 처리 확인 |
| **기대 결과** | - 커스텀 base_url 정상 동작<br>- API 키 있는/없는 경우 모두 처리<br>- 잘못된 URL 시 명확한 에러 메시지<br>- URL 끝의 `/` 유무 자동 처리<br>- HTTP/HTTPS 모두 지원 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/extensibility/test_openai_config.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | 전체 |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-EXT-004-03: OpenAI-compatible 서버 자동 감지

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-EXT-004-03 |
| **요구사항 ID** | NFR-EXT-004 |
| **테스트 유형** | E2E |
| **사전 조건** | - 로컬 네트워크에 Ollama 서버 실행 중<br>- 네트워크 스캔 기능 활성화 |
| **테스트 절차** | 1. 앱 설정에서 "로컬 서버 자동 감지" 활성화<br>2. 로컬 네트워크에서 OpenAI-compatible 서버 스캔<br>3. 발견된 서버 목록 표시 확인<br>4. 발견된 서버의 사용 가능 모델 조회 (`GET /v1/models`)<br>5. 사용자가 서버 선택 후 즉시 사용 가능 확인 |
| **기대 결과** | - 로컬 Ollama 서버 자동 감지<br>- 서버 목록에 이름/주소/모델 수 표시<br>- 모델 목록 조회 성공<br>- 서버 선택 후 즉시 전사/요약 사용 가능<br>- 감지 실패 시 수동 입력 옵션 제공 |
| **테스트 코드 위치** | `tests/nfr/ext/openai_server_discovery.spec.ts` |
| **자동화 여부** | 반자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |
