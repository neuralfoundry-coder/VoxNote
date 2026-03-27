# TC-TTS: 음성 합성 테스트 케이스

| 항목 | 내용 |
|------|------|
| **문서 ID** | TC-TTS |
| **SRS 참조** | VoxNote SRS v1.0 - FR-TTS (FR-TTS-001 ~ FR-TTS-004) |
| **작성일** | 2026-03-27 |
| **상태** | 초안 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/tts/` |

## 테스트 요약

| 테스트 유형 | 개수 |
|-------------|------|
| Unit | 5 |
| Integration | 4 |
| E2E (Playwright) | 2 |
| Performance | 2 |
| **합계** | **13** |

---

## FR-TTS-001: Piper TTS ONNX - P2

### TC-TTS-001-01: Piper ONNX 모델 로드 및 초기화

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-TTS-001-01 |
| **요구사항 ID** | FR-TTS-001 |
| **우선순위** | P2 |
| **테스트 유형** | Unit |
| **사전 조건** | Piper ONNX 모델 파일 및 config.json 존재 |
| **테스트 절차** | 1. `PiperTts::new("models/piper/ko-KR.onnx")` 호출<br>2. ONNX Runtime 세션 초기화 확인<br>3. 모델 메타데이터 (언어, 샘플레이트, 채널) 조회<br>4. 메모리 사용량 기록 |
| **기대 결과** | - ONNX 모델 로드 성공<br>- ONNX Runtime 세션 초기화 < 3초<br>- 메모리 사용량 < 500MB<br>- 모델 메타데이터 정상 반환 (샘플레이트: 22050Hz) |
| **테스트 코드 위치** | `crates/voxnote-core/tests/tts/test_piper_init.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-TTS-001-02: 텍스트 → 음성 기본 합성

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-TTS-001-02 |
| **요구사항 ID** | FR-TTS-001 |
| **우선순위** | P2 |
| **테스트 유형** | Integration |
| **사전 조건** | Piper TTS 모델 로드 완료 |
| **테스트 절차** | 1. 입력 텍스트: "안녕하세요, 보이스노트입니다."<br>2. `piper.synthesize(text)` 호출<br>3. 반환된 PCM 오디오 데이터 검증<br>4. WAV 파일로 저장 후 재생 가능 여부 확인 |
| **기대 결과** | - PCM 데이터 반환 (비어있지 않음)<br>- 샘플레이트: 22050Hz<br>- 오디오 길이 > 1초<br>- WAV 파일로 저장 후 정상 재생 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/tts/test_piper_synth.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-TTS-001-03: 빈 입력/특수 문자 처리

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-TTS-001-03 |
| **요구사항 ID** | FR-TTS-001 |
| **우선순위** | P2 |
| **테스트 유형** | Unit |
| **사전 조건** | Piper TTS 모델 로드 |
| **테스트 절차** | 1. 빈 문자열("") 입력 → 결과 확인<br>2. 숫자만("12345") 입력 → 결과 확인<br>3. 특수 문자("!@#$%") 입력 → 결과 확인<br>4. 매우 긴 텍스트(10000자) 입력 → 결과 확인 |
| **기대 결과** | - 빈 문자열: 빈 오디오 또는 적절한 에러 반환<br>- 숫자: 숫자 읽기 음성 생성<br>- 특수 문자: 패닉 없이 처리<br>- 긴 텍스트: 청킹 후 합성 완료 (OOM 없음) |
| **테스트 코드 위치** | `crates/voxnote-core/tests/tts/test_piper_synth.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-TTS-002: 한/영/일 음성 모델 - P2

### TC-TTS-002-01: 한국어 음성 모델 합성 품질

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-TTS-002-01 |
| **요구사항 ID** | FR-TTS-002 |
| **우선순위** | P2 |
| **테스트 유형** | Integration |
| **사전 조건** | 한국어 Piper 모델 로드 |
| **테스트 절차** | 1. 한국어 텍스트 5문장 준비<br>2. 각 문장 음성 합성<br>3. 합성 결과의 오디오 품질 측정 (SNR, MOS 추정)<br>4. 한국어 발음 정확도 수동 검증 |
| **기대 결과** | - 5문장 모두 합성 성공<br>- SNR > 20dB<br>- 한국어 발음이 자연스러움 (MOS ≥ 3.0 추정)<br>- 받침, 연음 처리 정확 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/tts/test_multilang.rs` |
| **자동화 여부** | 반자동 (음질 수동 검증) |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-TTS-002-02: 영어 음성 모델 합성

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-TTS-002-02 |
| **요구사항 ID** | FR-TTS-002 |
| **우선순위** | P2 |
| **테스트 유형** | Integration |
| **사전 조건** | 영어 Piper 모델 로드 |
| **테스트 절차** | 1. 영어 텍스트 5문장 준비<br>2. 각 문장 음성 합성<br>3. 합성 결과 검증 |
| **기대 결과** | - 5문장 모두 합성 성공<br>- 영어 발음 자연스러움<br>- 단어 강세/억양 적절<br>- 합성 시간 < 실시간 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/tts/test_multilang.rs` |
| **자동화 여부** | 반자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-TTS-002-03: 일본어 음성 모델 합성 및 모델 전환

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-TTS-002-03 |
| **요구사항 ID** | FR-TTS-002 |
| **우선순위** | P2 |
| **테스트 유형** | Unit |
| **사전 조건** | 한/영/일 3개 모델 파일 존재 |
| **테스트 절차** | 1. 일본어 모델로 일본어 텍스트 합성<br>2. 한국어 모델로 전환<br>3. 한국어 텍스트 합성<br>4. 모델 전환 시간 측정 |
| **기대 결과** | - 일본어 합성 성공 (히라가나/카타카나/한자 처리)<br>- 모델 전환 시간 < 2초<br>- 전환 후 이전 모델 메모리 해제<br>- 각 언어 모델의 음성 특성이 다름 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/tts/test_multilang.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-TTS-003: CPU 실시간 합성 - P2

### TC-TTS-003-01: CPU 실시간 합성 성능

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-TTS-003-01 |
| **요구사항 ID** | FR-TTS-003 |
| **우선순위** | P2 |
| **테스트 유형** | Performance |
| **사전 조건** | Piper TTS 모델 로드, GPU 비활성화 |
| **테스트 절차** | 1. 100자 한국어 텍스트 준비<br>2. CPU만 사용하여 합성 실행<br>3. 합성 시간 측정<br>4. 합성 결과 오디오 길이와 비교<br>5. RTF (Real-Time Factor) 계산 |
| **기대 결과** | - RTF < 1.0 (합성이 실시간보다 빠름)<br>- CPU 사용률 모니터링<br>- 단일 코어 사용 시에도 RTF < 1.0<br>- 메모리 사용량 < 500MB |
| **테스트 코드 위치** | `crates/voxnote-core/benches/tts_bench.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-TTS-003-02: 스트리밍 합성 출력

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-TTS-003-02 |
| **요구사항 ID** | FR-TTS-003 |
| **우선순위** | P2 |
| **테스트 유형** | Integration |
| **사전 조건** | Piper TTS 모델 로드 |
| **테스트 절차** | 1. 긴 텍스트(500자) 합성 요청<br>2. 문장 단위로 청킹하여 순차 합성<br>3. 첫 번째 청크의 오디오가 생성되는 시점 측정<br>4. 이전 청크 재생 중 다음 청크 합성 병렬 처리 확인 |
| **기대 결과** | - 첫 번째 오디오 청크 생성 < 500ms<br>- 청크 간 재생 끊김 없음<br>- 전체 합성 시간 < 순차 합성 대비 50%<br>- 메모리 사용량 안정적 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/tts/test_streaming_synth.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-TTS-004: 외부 TTS API - P2

### TC-TTS-004-01: 외부 TTS API 호출

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-TTS-004-01 |
| **요구사항 ID** | FR-TTS-004 |
| **우선순위** | P2 |
| **테스트 유형** | Integration (wiremock-rs) |
| **사전 조건** | wiremock-rs TTS API 모킹 서버 |
| **테스트 절차** | 1. wiremock-rs에 TTS 엔드포인트 모킹 (오디오 바이너리 응답)<br>2. `ExternalTtsProvider::synthesize(text, voice)` 호출<br>3. 응답 오디오 데이터 수신<br>4. PCM 변환 후 유효성 검증 |
| **기대 결과** | - API 호출 성공 (HTTP 200)<br>- 오디오 바이너리 데이터 수신<br>- PCM 변환 성공<br>- 음성 파라미터 (음성, 속도, 피치) 정상 전달 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/tts/test_external_tts.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-TTS-004-02: TTS Provider 전환 (로컬 ↔ 외부)

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-TTS-004-02 |
| **요구사항 ID** | FR-TTS-004 |
| **우선순위** | P2 |
| **테스트 유형** | Unit |
| **사전 조건** | 로컬 Piper 모델 및 외부 TTS API 설정 |
| **테스트 절차** | 1. 로컬 TTS로 합성 → 결과 확인<br>2. 외부 TTS API로 전환<br>3. 동일 텍스트 합성 → 결과 확인<br>4. 다시 로컬로 전환 |
| **기대 결과** | - 두 Provider 모두 정상 합성<br>- Provider 전환 시 에러 없음<br>- 출력 포맷 동일 (PCM f32)<br>- 전환 시간 < 1초 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/tts/test_external_tts.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-TTS-004-03: TTS 재생 E2E 워크플로우

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-TTS-004-03 |
| **요구사항 ID** | FR-TTS-004 |
| **우선순위** | P2 |
| **테스트 유형** | E2E (Playwright) |
| **사전 조건** | 앱 실행, TTS 모델 로드 |
| **테스트 절차** | 1. 노트 열기<br>2. 텍스트 선택 후 "음성으로 읽기" 버튼 클릭<br>3. 음성 재생 시작 확인<br>4. 일시정지/재개 버튼 테스트<br>5. 정지 버튼 테스트 |
| **기대 결과** | - 음성 재생 시작 < 1초<br>- 재생 진행률 표시<br>- 일시정지/재개 정상 동작<br>- 정지 후 리소스 해제 |
| **테스트 코드 위치** | `tests/e2e/tts/test_tts_playback.spec.ts` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-TTS-004-04: 외부 API 장애 시 로컬 폴백

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-TTS-004-04 |
| **요구사항 ID** | FR-TTS-004 |
| **우선순위** | P2 |
| **테스트 유형** | Performance |
| **사전 조건** | 외부 TTS API 장애 상태 모킹, 로컬 Piper 모델 로드 |
| **테스트 절차** | 1. wiremock-rs에 500 에러 응답 설정<br>2. 외부 TTS API로 합성 요청<br>3. 장애 감지 후 로컬 Piper로 폴백 확인<br>4. 폴백 전환 시간 측정 |
| **기대 결과** | - 외부 API 실패 감지 < 5초<br>- 로컬 Piper로 자동 전환<br>- 폴백 후 합성 결과 반환<br>- 사용자에게 폴백 알림 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/tts/test_external_tts.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |
