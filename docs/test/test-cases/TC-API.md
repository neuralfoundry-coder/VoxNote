# TC-API: 외부 Provider 연동 테스트 케이스

| 항목 | 내용 |
|------|------|
| **문서 ID** | TC-API |
| **SRS 참조** | VoxNote SRS v1.0 - FR-API (FR-API-001 ~ FR-API-013) |
| **작성일** | 2026-03-27 |
| **상태** | 초안 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/` |

## 테스트 요약

| 테스트 유형 | 개수 |
|-------------|------|
| Unit | 14 |
| Integration (wiremock-rs) | 12 |
| E2E (Playwright) | 6 |
| Performance | 2 |
| **합계** | **34** |

---

## FR-API-001: Provider 설정 UI - P0

### TC-API-001-01: Provider 설정 화면 표시

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-001-01 |
| **요구사항 ID** | FR-API-001 |
| **우선순위** | P0 |
| **테스트 유형** | E2E (Playwright) |
| **사전 조건** | 앱 실행 |
| **테스트 절차** | 1. 설정 메뉴 클릭<br>2. "Provider 설정" 탭 선택<br>3. Provider 목록 표시 확인<br>4. 각 Provider의 설정 항목 확인 |
| **기대 결과** | - Provider 목록에 OpenAI, Anthropic, Google, Groq, Ollama 표시<br>- 각 Provider의 활성화/비활성화 토글 존재<br>- API 키 입력 필드 존재<br>- 모델 선택 드롭다운 존재 |
| **테스트 코드 위치** | `tests/e2e/provider/test_provider_settings.spec.ts` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-API-001-02: Provider 설정 저장 및 로드

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-001-02 |
| **요구사항 ID** | FR-API-001 |
| **우선순위** | P0 |
| **테스트 유형** | E2E (Playwright) |
| **사전 조건** | 앱 실행, Provider 설정 화면 열림 |
| **테스트 절차** | 1. OpenAI Provider 활성화<br>2. API 키 입력<br>3. 모델 "gpt-4o" 선택<br>4. 저장 버튼 클릭<br>5. 앱 재시작 후 설정 유지 확인 |
| **기대 결과** | - 설정 저장 성공 알림<br>- 앱 재시작 후 설정 값 유지<br>- API 키가 마스킹되어 표시 (sk-****)<br>- 설정 파일에 암호화 저장 |
| **테스트 코드 위치** | `tests/e2e/provider/test_provider_settings.spec.ts` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-API-002: OpenAI API - P0

### TC-API-002-01: OpenAI Chat Completion API 호출

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-002-01 |
| **요구사항 ID** | FR-API-002 |
| **우선순위** | P0 |
| **테스트 유형** | Integration (wiremock-rs) |
| **사전 조건** | wiremock-rs 모킹 서버 시작 |
| **테스트 절차** | 1. wiremock-rs에 `/v1/chat/completions` 엔드포인트 모킹<br>2. `OpenAiProvider::chat_completion(messages)` 호출<br>3. 요청 헤더 검증 (Authorization: Bearer sk-xxx)<br>4. 응답 파싱 검증 |
| **기대 결과** | - HTTP POST 요청 정상 전송<br>- Authorization 헤더에 API 키 포함<br>- 응답 JSON 파싱 성공<br>- `choices[0].message.content` 추출 성공 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_openai.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-API-002-02: OpenAI 스트리밍 응답 처리

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-002-02 |
| **요구사항 ID** | FR-API-002 |
| **우선순위** | P0 |
| **테스트 유형** | Integration (wiremock-rs) |
| **사전 조건** | wiremock-rs SSE 스트리밍 모킹 |
| **테스트 절차** | 1. wiremock-rs에 SSE 스트리밍 응답 설정<br>2. `stream: true` 옵션으로 API 호출<br>3. `data: {"choices": [{"delta": {"content": "..."}}]}` 청크 수신<br>4. `data: [DONE]` 수신 시 완료 처리 |
| **기대 결과** | - SSE 스트리밍 연결 성공<br>- 각 청크의 delta content 파싱 성공<br>- `[DONE]` 메시지로 스트림 종료 감지<br>- 전체 응답 = 모든 delta 합산 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_openai.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-API-002-03: OpenAI API 에러 처리

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-002-03 |
| **요구사항 ID** | FR-API-002 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | wiremock-rs 에러 응답 모킹 |
| **테스트 절차** | 1. 401 Unauthorized 응답 모킹 → API 호출<br>2. 429 Rate Limit 응답 모킹 → API 호출<br>3. 500 Server Error 응답 모킹 → API 호출<br>4. 각 에러의 처리 결과 확인 |
| **기대 결과** | - 401: `ApiError::Unauthorized` 반환<br>- 429: 자동 재시도 (exponential backoff, 최대 3회)<br>- 500: `ApiError::ServerError` 반환<br>- 모든 에러에 대한 사용자 친화적 메시지 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_openai.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-API-003: Anthropic API - P0

### TC-API-003-01: Anthropic Messages API 호출

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-003-01 |
| **요구사항 ID** | FR-API-003 |
| **우선순위** | P0 |
| **테스트 유형** | Integration (wiremock-rs) |
| **사전 조건** | wiremock-rs 모킹 서버 |
| **테스트 절차** | 1. wiremock-rs에 `/v1/messages` 엔드포인트 모킹<br>2. `AnthropicProvider::create_message(messages)` 호출<br>3. 요청 헤더 검증 (x-api-key, anthropic-version)<br>4. 응답 파싱 검증 |
| **기대 결과** | - `x-api-key` 헤더에 API 키 포함<br>- `anthropic-version` 헤더 포함<br>- 응답의 `content[0].text` 추출 성공<br>- 입력/출력 토큰 수 파싱 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_anthropic.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-API-003-02: Anthropic 스트리밍 SSE 처리

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-003-02 |
| **요구사항 ID** | FR-API-003 |
| **우선순위** | P0 |
| **테스트 유형** | Integration (wiremock-rs) |
| **사전 조건** | SSE 스트리밍 모킹 설정 |
| **테스트 절차** | 1. `stream: true` 로 Anthropic API 호출<br>2. `event: content_block_delta` 이벤트 수신<br>3. `event: message_stop` 이벤트로 종료 감지<br>4. 전체 텍스트 조합 |
| **기대 결과** | - SSE 이벤트 타입별 정상 파싱<br>- `content_block_delta.delta.text` 추출 성공<br>- `message_stop` 이벤트로 스트림 종료<br>- 전체 응답 텍스트 일관성 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_anthropic.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-API-004: Google Gemini - P1

### TC-API-004-01: Gemini generateContent API 호출

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-004-01 |
| **요구사항 ID** | FR-API-004 |
| **우선순위** | P1 |
| **테스트 유형** | Integration (wiremock-rs) |
| **사전 조건** | wiremock-rs 모킹 서버 |
| **테스트 절차** | 1. Gemini generateContent 엔드포인트 모킹<br>2. `GeminiProvider::generate_content(contents)` 호출<br>3. 요청 URL에 API 키 포함 확인<br>4. 응답 파싱 검증 |
| **기대 결과** | - API 키가 URL 파라미터로 전달<br>- `candidates[0].content.parts[0].text` 추출 성공<br>- Safety 필터링 정보 파싱<br>- 사용량 메타데이터 수신 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_gemini.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-API-004-02: Gemini 스트리밍 응답

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-004-02 |
| **요구사항 ID** | FR-API-004 |
| **우선순위** | P1 |
| **테스트 유형** | Integration (wiremock-rs) |
| **사전 조건** | 스트리밍 모킹 |
| **테스트 절차** | 1. `streamGenerateContent` 엔드포인트 모킹<br>2. 스트리밍 호출<br>3. JSON 라인별 파싱<br>4. 전체 결과 조합 |
| **기대 결과** | - 스트리밍 연결 성공<br>- 각 청크 JSON 파싱 성공<br>- 텍스트 조각 순서 보장<br>- 완료 시 finish_reason 확인 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_gemini.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-API-005: Groq - P2

### TC-API-005-01: Groq API 호출 (OpenAI 호환)

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-005-01 |
| **요구사항 ID** | FR-API-005 |
| **우선순위** | P2 |
| **테스트 유형** | Integration (wiremock-rs) |
| **사전 조건** | wiremock-rs 모킹 서버 |
| **테스트 절차** | 1. Groq API 엔드포인트 (`api.groq.com`) 모킹<br>2. OpenAI 호환 형식으로 요청 전송<br>3. 응답 파싱 (OpenAI 형식) |
| **기대 결과** | - OpenAI 호환 요청/응답 정상 처리<br>- base_url이 Groq 엔드포인트로 설정<br>- 모델명 (llama, mixtral 등) 정상 전달<br>- 응답 시간 기록 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_groq.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-API-005-02: Groq Rate Limit 처리

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-005-02 |
| **요구사항 ID** | FR-API-005 |
| **우선순위** | P2 |
| **테스트 유형** | Unit |
| **사전 조건** | wiremock-rs 429 응답 모킹 |
| **테스트 절차** | 1. Groq API에 연속 요청 전송<br>2. 429 응답 수신 시 재시도 로직 확인<br>3. `x-ratelimit-*` 헤더 파싱<br>4. 재시도 간격이 점진적으로 증가하는지 확인 |
| **기대 결과** | - Rate limit 헤더 정상 파싱<br>- Exponential backoff 적용<br>- 최대 3회 재시도 후 에러 반환<br>- 재시도 간격: 1초 → 2초 → 4초 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_groq.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-API-006: Ollama - P1

### TC-API-006-01: Ollama 로컬 API 연동

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-006-01 |
| **요구사항 ID** | FR-API-006 |
| **우선순위** | P1 |
| **테스트 유형** | Integration (wiremock-rs) |
| **사전 조건** | wiremock-rs로 Ollama API 모킹 (localhost:11434) |
| **테스트 절차** | 1. `/api/generate` 엔드포인트 모킹<br>2. `OllamaProvider::generate(prompt, model)` 호출<br>3. 응답 NDJSON 라인별 파싱<br>4. `done: true` 플래그로 완료 감지 |
| **기대 결과** | - localhost:11434 연결 성공<br>- NDJSON 스트리밍 파싱 성공<br>- 각 라인의 `response` 필드 추출<br>- `done: true` 시 스트림 종료 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_ollama.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-API-006-02: Ollama 모델 목록 조회

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-006-02 |
| **요구사항 ID** | FR-API-006 |
| **우선순위** | P1 |
| **테스트 유형** | Unit |
| **사전 조건** | wiremock-rs 모킹 |
| **테스트 절차** | 1. `/api/tags` 엔드포인트 모킹 (모델 목록 반환)<br>2. `OllamaProvider::list_models()` 호출<br>3. 반환된 모델 목록 검증 |
| **기대 결과** | - 모델 목록 배열 반환<br>- 각 모델의 이름, 크기, 양자화 정보 포함<br>- 빈 목록 시 적절한 메시지 반환<br>- Ollama 미실행 시 연결 에러 처리 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_ollama.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-API-007: 엔진별 독립 Provider 선택 - P0

### TC-API-007-01: STT/LLM/TTS 별도 Provider 설정

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-007-01 |
| **요구사항 ID** | FR-API-007 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 다수의 Provider 설정 완료 |
| **테스트 절차** | 1. STT Provider를 "로컬 (whisper.cpp)"로 설정<br>2. LLM Provider를 "OpenAI (gpt-4o)"로 설정<br>3. TTS Provider를 "로컬 (Piper)"로 설정<br>4. 각 엔진이 올바른 Provider를 사용하는지 확인 |
| **기대 결과** | - STT: 로컬 whisper.cpp 엔진 사용<br>- LLM: OpenAI API 호출<br>- TTS: 로컬 Piper 사용<br>- 엔진 간 Provider 설정이 독립적 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_engine_provider.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-API-007-02: Provider 동적 전환

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-007-02 |
| **요구사항 ID** | FR-API-007 |
| **우선순위** | P0 |
| **테스트 유형** | Integration |
| **사전 조건** | 2개 이상의 LLM Provider 설정 |
| **테스트 절차** | 1. LLM Provider를 OpenAI로 설정<br>2. 요약 생성 실행<br>3. 실행 중 Provider를 Anthropic으로 전환<br>4. 다음 요약 생성 시 Anthropic 사용 확인 |
| **기대 결과** | - Provider 전환 시 진행 중 작업 완료 후 전환<br>- 전환 후 새 Provider로 정상 동작<br>- 설정 즉시 반영<br>- 이전 Provider 리소스 정리 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_engine_provider.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-API-008: 통합 추상화 인터페이스 trait - P0

### TC-API-008-01: Provider trait 구현 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-008-01 |
| **요구사항 ID** | FR-API-008 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 없음 |
| **테스트 절차** | 1. `LlmProvider` trait의 필수 메서드 목록 확인<br>2. OpenAI, Anthropic, Gemini, Ollama 구현체가 trait를 만족하는지 컴파일 타임 검증<br>3. trait 객체 (`Box<dyn LlmProvider>`) 로 동적 디스패치 테스트<br>4. 각 구현체의 메서드 호출 결과 확인 |
| **기대 결과** | - 모든 구현체가 `LlmProvider` trait 만족<br>- `chat()`, `stream_chat()`, `list_models()` 메서드 존재<br>- 동적 디스패치 정상 동작<br>- 컴파일 에러 없음 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_trait.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-API-008-02: Provider 전환 시 인터페이스 일관성

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-008-02 |
| **요구사항 ID** | FR-API-008 |
| **우선순위** | P0 |
| **테스트 유형** | Integration (wiremock-rs) |
| **사전 조건** | 각 Provider의 모킹 서버 설정 |
| **테스트 절차** | 1. 동일 프롬프트로 OpenAI Provider 호출<br>2. 동일 프롬프트로 Anthropic Provider 호출<br>3. 두 응답의 반환 타입이 동일한지 확인<br>4. `ChatResponse` 구조체 필드 일관성 검증 |
| **기대 결과** | - 모든 Provider가 동일한 `ChatResponse` 타입 반환<br>- `text`, `model`, `usage`, `finish_reason` 필드 공통<br>- Provider별 차이는 내부 구현에만 존재<br>- 상위 레이어에서 Provider 무관하게 사용 가능 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_trait.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-API-009: API 키 키체인 보관 - P0

### TC-API-009-01: macOS 키체인 API 키 저장/조회

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-009-01 |
| **요구사항 ID** | FR-API-009 |
| **우선순위** | P0 |
| **테스트 유형** | Integration |
| **사전 조건** | macOS 키체인 접근 권한 |
| **테스트 절차** | 1. `Keychain::store("voxnote.openai", "sk-test-key-123")` 호출<br>2. `Keychain::retrieve("voxnote.openai")` 호출<br>3. 반환된 키 값 검증<br>4. `Keychain::delete("voxnote.openai")` 호출 후 조회 시 에러 확인 |
| **기대 결과** | - 키체인 저장 성공<br>- 저장된 키 정확히 조회<br>- 삭제 후 조회 시 `NotFound` 에러<br>- 키가 평문으로 디스크에 저장되지 않음 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_keychain.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-API-009-02: Windows Credential Manager 키 저장

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-009-02 |
| **요구사항 ID** | FR-API-009 |
| **우선순위** | P0 |
| **테스트 유형** | Integration |
| **사전 조건** | Windows Credential Manager 접근 가능 |
| **테스트 절차** | 1. Windows Credential Manager에 API 키 저장<br>2. 저장된 키 조회<br>3. 키 삭제 후 확인 |
| **기대 결과** | - Credential Manager 저장 성공<br>- 키 조회 값 일치<br>- 삭제 후 조회 실패<br>- DPAPI로 암호화 저장 확인 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_keychain.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | Windows |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-API-009-03: Linux Secret Service 키 저장

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-009-03 |
| **요구사항 ID** | FR-API-009 |
| **우선순위** | P0 |
| **테스트 유형** | Integration |
| **사전 조건** | GNOME Keyring 또는 KWallet 사용 가능 |
| **테스트 절차** | 1. Secret Service API로 API 키 저장<br>2. 저장된 키 조회<br>3. 키 삭제 후 확인 |
| **기대 결과** | - Secret Service 연결 성공<br>- 키 저장/조회/삭제 정상 동작<br>- D-Bus 통신 에러 처리<br>- Secret Service 미설치 시 폴백 (파일 기반 암호화) |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_keychain.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-API-010: 네트워크 장애 로컬 폴백 - P1

### TC-API-010-01: 네트워크 타임아웃 시 로컬 폴백

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-010-01 |
| **요구사항 ID** | FR-API-010 |
| **우선순위** | P1 |
| **테스트 유형** | Integration (wiremock-rs) |
| **사전 조건** | 로컬 LLM 모델 로드, wiremock-rs 지연 응답 설정 |
| **테스트 절차** | 1. wiremock-rs에 30초 지연 응답 설정 (타임아웃 유발)<br>2. 외부 API로 요약 요청<br>3. 10초 타임아웃 후 로컬 폴백 동작 확인<br>4. 로컬 LLM으로 요약 생성 확인 |
| **기대 결과** | - 10초 타임아웃 후 자동으로 로컬 모델 전환<br>- 로컬 모델로 요약 결과 반환<br>- 사용자에게 폴백 알림<br>- 네트워크 복구 후 자동 외부 API 복귀 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_fallback.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-API-010-02: DNS 해석 실패 시 폴백

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-010-02 |
| **요구사항 ID** | FR-API-010 |
| **우선순위** | P1 |
| **테스트 유형** | Unit |
| **사전 조건** | 존재하지 않는 호스트명 설정 |
| **테스트 절차** | 1. Provider 엔드포인트를 잘못된 호스트로 설정<br>2. API 호출 시도<br>3. DNS 실패 감지 후 폴백 동작 확인 |
| **기대 결과** | - DNS 해석 실패 감지<br>- 2초 이내 로컬 폴백<br>- `NetworkError::DnsResolutionFailed` 로그 기록<br>- 사용자 알림: "네트워크 연결을 확인해주세요" |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_fallback.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-API-011: 사용자 정의 엔드포인트 - P1

### TC-API-011-01: 커스텀 엔드포인트 설정

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-011-01 |
| **요구사항 ID** | FR-API-011 |
| **우선순위** | P1 |
| **테스트 유형** | Unit |
| **사전 조건** | 없음 |
| **테스트 절차** | 1. `ProviderConfig::new().base_url("https://my-proxy.example.com/v1")` 설정<br>2. API 호출 시 요청 URL 확인<br>3. 기존 Provider 프로토콜 (OpenAI 호환) 유지 확인 |
| **기대 결과** | - 커스텀 URL로 요청 전송<br>- 기존 인증 헤더 유지<br>- 응답 파싱 정상 동작<br>- URL 유효성 검사 (HTTP/HTTPS만 허용) |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_custom_endpoint.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-API-011-02: 프록시 서버 경유

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-011-02 |
| **요구사항 ID** | FR-API-011 |
| **우선순위** | P1 |
| **테스트 유형** | Integration (wiremock-rs) |
| **사전 조건** | wiremock-rs 프록시 모킹 |
| **테스트 절차** | 1. HTTP 프록시 서버 주소 설정<br>2. API 호출이 프록시를 경유하는지 확인<br>3. 프록시 인증 (username/password) 설정 테스트 |
| **기대 결과** | - 프록시 경유 API 호출 성공<br>- 프록시 인증 정상 처리<br>- 프록시 미응답 시 직접 연결 폴백 옵션<br>- HTTPS 프록시 (CONNECT 메서드) 지원 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_custom_endpoint.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-API-012: API 사용량 모니터링 - P2

### TC-API-012-01: 토큰 사용량 기록

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-012-01 |
| **요구사항 ID** | FR-API-012 |
| **우선순위** | P2 |
| **테스트 유형** | Unit |
| **사전 조건** | Provider 설정 완료, 사용량 DB 초기화 |
| **테스트 절차** | 1. OpenAI API 호출 (모킹)<br>2. 응답의 `usage.prompt_tokens`, `usage.completion_tokens` 파싱<br>3. 사용량 DB에 기록<br>4. 사용량 조회 API로 누적 토큰 수 확인 |
| **기대 결과** | - 입력/출력 토큰 수 정확히 기록<br>- Provider별 사용량 분리 기록<br>- 일별/월별 집계 가능<br>- 예상 비용 계산 (토큰 단가 기반) |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_usage.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-API-012-02: 사용량 대시보드 E2E

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-012-02 |
| **요구사항 ID** | FR-API-012 |
| **우선순위** | P2 |
| **테스트 유형** | E2E (Playwright) |
| **사전 조건** | 사용량 데이터 존재 |
| **테스트 절차** | 1. 설정 > 사용량 대시보드 열기<br>2. Provider별 사용량 차트 표시 확인<br>3. 기간 필터 (일/주/월) 변경 테스트<br>4. 비용 추정값 표시 확인 |
| **기대 결과** | - 대시보드 정상 렌더링<br>- 차트 데이터와 DB 데이터 일치<br>- 기간 필터 정상 동작<br>- 비용 추정값 표시 (USD) |
| **테스트 코드 위치** | `tests/e2e/provider/test_usage_dashboard.spec.ts` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-API-013: Provider 플러그인 구조 - P0

### TC-API-013-01: 플러그인 동적 로드

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-013-01 |
| **요구사항 ID** | FR-API-013 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 테스트용 플러그인 .dylib/.dll/.so 파일 준비 |
| **테스트 절차** | 1. `PluginManager::load("plugins/test-provider.dylib")` 호출<br>2. 플러그인이 `LlmProvider` trait를 구현하는지 확인<br>3. 플러그인을 통한 API 호출 테스트<br>4. 플러그인 언로드 |
| **기대 결과** | - 플러그인 동적 로드 성공<br>- trait 구현 확인<br>- 플러그인을 통한 API 호출 정상<br>- 언로드 후 메모리 해제 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_plugin.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-API-013-02: 플러그인 검증 및 샌드박싱

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-013-02 |
| **요구사항 ID** | FR-API-013 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 유효/무효 플러그인 파일 준비 |
| **테스트 절차** | 1. 유효한 플러그인 로드 → 성공 확인<br>2. 잘못된 ABI 버전 플러그인 로드 시도<br>3. 악성 플러그인 (파일 시스템 접근) 로드 시도<br>4. 플러그인 버전 호환성 확인 |
| **기대 결과** | - 유효 플러그인: 로드 성공<br>- ABI 불일치: `PluginError::IncompatibleAbi` 반환<br>- 보안 위반: 플러그인 격리 또는 로드 거부<br>- 버전 확인: semver 호환성 검사 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/provider/test_plugin.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-API-013-03: 플러그인 등록 E2E

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-API-013-03 |
| **요구사항 ID** | FR-API-013 |
| **우선순위** | P0 |
| **테스트 유형** | E2E (Playwright) |
| **사전 조건** | 앱 실행, 테스트 플러그인 파일 준비 |
| **테스트 절차** | 1. 설정 > 플러그인 관리 열기<br>2. "플러그인 설치" 버튼 클릭<br>3. 플러그인 파일 선택<br>4. 플러그인 목록에 추가되었는지 확인<br>5. Provider 목록에 새 플러그인 Provider 표시 확인 |
| **기대 결과** | - 플러그인 설치 성공 알림<br>- 플러그인 목록에 표시 (이름, 버전, 설명)<br>- Provider 설정에 새 Provider 선택 가능<br>- 플러그인 비활성화/삭제 버튼 동작 |
| **테스트 코드 위치** | `tests/e2e/provider/test_plugin_ui.spec.ts` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |
