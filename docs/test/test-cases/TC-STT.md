# TC-STT: 음성-텍스트 변환 테스트 케이스

| 항목 | 내용 |
|------|------|
| **문서 ID** | TC-STT |
| **SRS 참조** | VoxNote SRS v1.0 - FR-STT (FR-STT-001 ~ FR-STT-010) |
| **작성일** | 2026-03-27 |
| **상태** | 초안 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/` |

## 테스트 요약

| 테스트 유형 | 개수 |
|-------------|------|
| Unit | 10 |
| Integration | 8 |
| E2E (Playwright) | 4 |
| Performance (RTF benchmark) | 4 |
| Accuracy (WER/CER) | 4 |
| **합계** | **30** |

---

## FR-STT-001: whisper.cpp 실시간 스트리밍 전사 - P0

### TC-STT-001-01: whisper.cpp 모델 로드 및 초기화

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-001-01 |
| **요구사항 ID** | FR-STT-001 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | whisper.cpp tiny 모델 파일 존재 |
| **테스트 절차** | 1. `WhisperContext::new("models/ggml-tiny.bin")` 호출<br>2. 컨텍스트 초기화 상태 확인<br>3. 모델 메타데이터 (언어, 파라미터 수) 조회<br>4. 메모리 사용량 기록 |
| **기대 결과** | - 컨텍스트 초기화 성공 (`Ok` 반환)<br>- 모델 타입: tiny 확인<br>- 초기화 시간 < 2초<br>- 메모리 사용량 < 200MB (tiny 모델) |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_whisper_init.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-STT-001-02: 실시간 스트리밍 전사 기본 동작

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-001-02 |
| **요구사항 ID** | FR-STT-001 |
| **우선순위** | P0 |
| **테스트 유형** | Integration |
| **사전 조건** | whisper.cpp 모델 로드 완료, 테스트 오디오 파일 준비 |
| **테스트 절차** | 1. 한국어 음성 테스트 파일(10초) 로드<br>2. 오디오를 2초 청크로 분할<br>3. 각 청크를 순차적으로 `whisper_full()` 에 입력<br>4. 각 청크의 전사 결과 수집<br>5. 전체 전사 결과 합산 |
| **기대 결과** | - 각 청크에 대한 전사 결과 반환<br>- 전체 전사 텍스트가 레퍼런스와 유사 (WER < 15%)<br>- 청크 간 전사 연결이 자연스러움<br>- 마지막 청크 처리 후 완료 상태 반환 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_whisper_streaming.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-STT-001-03: 스트리밍 전사 RTF 성능

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-001-03 |
| **요구사항 ID** | FR-STT-001 |
| **우선순위** | P0 |
| **테스트 유형** | Performance (RTF) |
| **사전 조건** | whisper.cpp base 모델 로드 완료 |
| **테스트 절차** | 1. 60초 분량의 테스트 오디오 준비<br>2. 스트리밍 전사 파이프라인 실행<br>3. 총 처리 시간 측정<br>4. RTF (Real-Time Factor) 계산: 처리 시간 / 오디오 길이 |
| **기대 결과** | - RTF < 0.5 (GPU 가속 시)<br>- RTF < 1.0 (CPU만 사용 시)<br>- 메모리 사용량 안정적 (시간에 따른 증가 없음)<br>- 첫 토큰 출력 레이턴시 < 500ms |
| **테스트 코드 위치** | `crates/voxnote-core/benches/stt_rtf_bench.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-STT-002: 슬라이딩 윈도우 2~3초 + 0.5초 오버랩 - P0

### TC-STT-002-01: 슬라이딩 윈도우 크기 및 오버랩 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-002-01 |
| **요구사항 ID** | FR-STT-002 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 없음 |
| **테스트 절차** | 1. `SlidingWindow::new(window_size: 3.0, overlap: 0.5)` 생성<br>2. 10초 분량의 연속 오디오 샘플을 push<br>3. 생성되는 각 윈도우의 시작/끝 타임스탬프 기록<br>4. 윈도우 간 오버랩 구간 계산 |
| **기대 결과** | - 각 윈도우 크기: 3.0초 (48000 샘플 @ 16kHz)<br>- 윈도우 간 오버랩: 0.5초 (8000 샘플)<br>- 윈도우 스트라이드: 2.5초<br>- 10초 입력 시 윈도우 4개 생성 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_sliding_window.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-STT-002-02: 오버랩 구간 중복 전사 제거

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-002-02 |
| **요구사항 ID** | FR-STT-002 |
| **우선순위** | P0 |
| **테스트 유형** | Integration |
| **사전 조건** | whisper.cpp 모델 로드, 슬라이딩 윈도우 설정 완료 |
| **테스트 절차** | 1. 명확한 문장 구분이 있는 테스트 오디오 입력<br>2. 슬라이딩 윈도우로 청크 분할<br>3. 각 청크 전사 후 오버랩 구간 결과 비교<br>4. 중복 텍스트 제거 로직 적용<br>5. 최종 전사 결과 확인 |
| **기대 결과** | - 오버랩 구간의 중복 텍스트가 제거됨<br>- 제거 후 문맥이 자연스럽게 연결됨<br>- 단어 누락 없음<br>- 원본 대비 WER 증가 < 2% |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_sliding_window.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-STT-002-03: 윈도우 크기 동적 조정 (2초~3초)

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-002-03 |
| **요구사항 ID** | FR-STT-002 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 없음 |
| **테스트 절차** | 1. 윈도우 크기 2.0초로 설정하여 테스트<br>2. 윈도우 크기 3.0초로 설정하여 테스트<br>3. VAD 기반 동적 조정 모드에서 테스트 (음성 활동에 따라 2~3초 자동 조절)<br>4. 각 설정의 전사 품질 및 레이턴시 비교 |
| **기대 결과** | - 2초 윈도우: 레이턴시 낮음, WER 약간 높음<br>- 3초 윈도우: 레이턴시 높음, WER 낮음<br>- 동적 조정: 최적 균형 달성<br>- 모든 설정에서 오버랩 0.5초 유지 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_sliding_window.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-STT-003: initial_prompt 문맥 연속성 - P0

### TC-STT-003-01: initial_prompt를 통한 문맥 전달

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-003-01 |
| **요구사항 ID** | FR-STT-003 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | whisper.cpp 모델 로드, 테스트 오디오 준비 |
| **테스트 절차** | 1. 첫 번째 청크 전사 실행 (initial_prompt 없음)<br>2. 첫 번째 전사 결과의 마지막 문장을 initial_prompt로 설정<br>3. 두 번째 청크 전사 실행 (initial_prompt 포함)<br>4. initial_prompt 없이 두 번째 청크 전사 (비교군)<br>5. 두 결과 비교 |
| **기대 결과** | - initial_prompt 사용 시 문맥 연속성 향상<br>- 동일 단어의 일관된 표기 유지<br>- initial_prompt 미사용 대비 WER 개선 ≥ 3%<br>- initial_prompt 토큰 수 < 224 (whisper 제한) |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_context.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-STT-003-02: 전문 용어 사전을 통한 문맥 힌트

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-003-02 |
| **요구사항 ID** | FR-STT-003 |
| **우선순위** | P0 |
| **테스트 유형** | Integration |
| **사전 조건** | 사용자 정의 용어 사전 파일, 전문 용어가 포함된 테스트 오디오 |
| **테스트 절차** | 1. 전문 용어 사전 로드 (예: "Kubernetes", "gRPC", "리샘플링")<br>2. 사전의 용어를 initial_prompt에 포함하여 전사<br>3. 사전 없이 동일 오디오 전사<br>4. 전문 용어 인식률 비교 |
| **기대 결과** | - 사전 적용 시 전문 용어 인식률 향상<br>- 사전 미적용 대비 CER 개선 ≥ 5%<br>- 용어 사전이 문맥에 맞게 적용됨<br>- 사전에 없는 일반 단어 전사 품질에 악영향 없음 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_context.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-STT-004: 다국어 자동감지/수동 선택 - P0

### TC-STT-004-01: 언어 자동 감지 정확도

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-004-01 |
| **요구사항 ID** | FR-STT-004 |
| **우선순위** | P0 |
| **테스트 유형** | Accuracy (WER/CER) |
| **사전 조건** | 한국어, 영어, 일본어, 중국어 테스트 오디오 각 1개 준비 |
| **테스트 절차** | 1. 한국어 오디오 → 언어 자동 감지 실행<br>2. 영어 오디오 → 언어 자동 감지 실행<br>3. 일본어 오디오 → 언어 자동 감지 실행<br>4. 중국어 오디오 → 언어 자동 감지 실행<br>5. 각 감지 결과의 언어 코드 및 신뢰도 기록 |
| **기대 결과** | - 4개 언어 모두 정확히 감지 (정확도 100%)<br>- 감지 신뢰도 ≥ 0.8<br>- 감지 소요 시간 < 1초<br>- 감지 결과에 ISO 639-1 언어 코드 포함 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_language.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-STT-004-02: 수동 언어 선택 전사

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-004-02 |
| **요구사항 ID** | FR-STT-004 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 한국어 테스트 오디오 준비 |
| **테스트 절차** | 1. 언어를 "ko"로 수동 설정하여 전사<br>2. 언어를 "en"으로 잘못 설정하여 동일 오디오 전사<br>3. 두 결과의 WER 비교 |
| **기대 결과** | - 정확한 언어 설정 시 WER < 10%<br>- 잘못된 언어 설정 시 WER > 50%<br>- 수동 선택이 자동 감지를 오버라이드함<br>- 언어 설정 값이 전사 결과 메타데이터에 포함 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_language.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-STT-004-03: 다국어 혼합 음성 처리

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-004-03 |
| **요구사항 ID** | FR-STT-004 |
| **우선순위** | P0 |
| **테스트 유형** | Accuracy (WER/CER) |
| **사전 조건** | 한-영 코드스위칭 오디오 준비 |
| **테스트 절차** | 1. 한국어와 영어가 섞인 오디오 입력<br>2. 자동 감지 모드로 전사<br>3. 전사 결과에서 각 언어의 인식 정확도 측정 |
| **기대 결과** | - 주요 언어 감지 정확<br>- 코드스위칭 전환점에서 전사 품질 유지<br>- 영어 단어가 한글이 아닌 영문으로 전사됨<br>- 혼합 음성 전체 WER < 25% |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_language.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-STT-005: Metal/CUDA GPU 가속 - P0

### TC-STT-005-01: Metal GPU 가속 전사 (macOS)

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-005-01 |
| **요구사항 ID** | FR-STT-005 |
| **우선순위** | P0 |
| **테스트 유형** | Performance (RTF) |
| **사전 조건** | macOS, Apple Silicon (M1+), Metal 지원 |
| **테스트 절차** | 1. `WhisperContext::new_with_gpu(GpuBackend::Metal)` 로 초기화<br>2. 60초 테스트 오디오로 전사 실행<br>3. GPU 사용률 모니터링<br>4. 처리 시간 및 RTF 측정<br>5. CPU 전용 모드와 성능 비교 |
| **기대 결과** | - Metal 백엔드 초기화 성공<br>- GPU 사용률 > 50% (전사 중)<br>- RTF < 0.3 (base 모델 기준)<br>- CPU 대비 속도 향상 ≥ 2배 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_gpu_accel.rs` |
| **자동화 여부** | 반자동 (Metal 지원 하드웨어 필요) |
| **플랫폼** | macOS (Apple Silicon) |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-STT-005-02: CUDA GPU 가속 전사 (Windows/Linux)

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-005-02 |
| **요구사항 ID** | FR-STT-005 |
| **우선순위** | P0 |
| **테스트 유형** | Performance (RTF) |
| **사전 조건** | NVIDIA GPU, CUDA 12.0+, cuBLAS 설치 |
| **테스트 절차** | 1. `WhisperContext::new_with_gpu(GpuBackend::Cuda)` 로 초기화<br>2. 60초 테스트 오디오로 전사 실행<br>3. nvidia-smi로 GPU 메모리/사용률 모니터링<br>4. 처리 시간 및 RTF 측정 |
| **기대 결과** | - CUDA 백엔드 초기화 성공<br>- GPU VRAM 사용량 < 2GB (base 모델)<br>- RTF < 0.2 (base 모델 기준)<br>- CPU 대비 속도 향상 ≥ 3배 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_gpu_accel.rs` |
| **자동화 여부** | 반자동 (CUDA 하드웨어 필요) |
| **플랫폼** | Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-STT-005-03: GPU 미지원 시 CPU 폴백

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-005-03 |
| **요구사항 ID** | FR-STT-005 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | GPU 없는 환경 또는 GPU 비활성화 |
| **테스트 절차** | 1. GPU 백엔드 초기화 시도<br>2. GPU 초기화 실패 시 CPU 폴백 로직 확인<br>3. CPU 모드로 정상 전사 실행<br>4. 폴백 로그 메시지 확인 |
| **기대 결과** | - GPU 초기화 실패 시 자동으로 CPU 모드 전환<br>- 에러 없이 CPU 모드 전사 성공<br>- 경고 로그에 "GPU unavailable, falling back to CPU" 메시지 포함<br>- 전사 품질은 GPU 모드와 동일 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_gpu_accel.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-STT-006: tiny~large-v3-turbo 모델 선택 - P0

### TC-STT-006-01: 모델별 전사 품질 비교

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-006-01 |
| **요구사항 ID** | FR-STT-006 |
| **우선순위** | P0 |
| **테스트 유형** | Accuracy (WER/CER) |
| **사전 조건** | tiny, base, small, medium, large-v3-turbo 모델 파일 준비 |
| **테스트 절차** | 1. 동일한 30초 한국어 테스트 오디오 준비<br>2. tiny 모델로 전사 → WER 측정<br>3. base 모델로 전사 → WER 측정<br>4. small 모델로 전사 → WER 측정<br>5. medium 모델로 전사 → WER 측정<br>6. large-v3-turbo 모델로 전사 → WER 측정 |
| **기대 결과** | - tiny WER < 30%<br>- base WER < 20%<br>- small WER < 15%<br>- medium WER < 10%<br>- large-v3-turbo WER < 8%<br>- 모델 크기 증가에 따라 WER 감소 추세 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_model_selection.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-STT-006-02: 모델 다운로드 및 관리

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-006-02 |
| **요구사항 ID** | FR-STT-006 |
| **우선순위** | P0 |
| **테스트 유형** | Integration |
| **사전 조건** | 네트워크 연결 가능, 모델 저장 디렉토리 접근 가능 |
| **테스트 절차** | 1. 모델 목록 조회 API 호출<br>2. tiny 모델 다운로드 요청<br>3. 다운로드 진행률 콜백 수신 확인<br>4. 다운로드 완료 후 GGML 파일 무결성 검증 (SHA256)<br>5. 모델 전환 (tiny → base) 실행 |
| **기대 결과** | - 모델 목록에 5종 이상 표시<br>- 다운로드 진행률 0~100% 정상 보고<br>- SHA256 체크섬 일치<br>- 모델 전환 시 이전 모델 메모리 해제 확인 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_model_selection.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-STT-007: 실시간 번역 모드 - P1

### TC-STT-007-01: 비영어 → 영어 실시간 번역

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-007-01 |
| **요구사항 ID** | FR-STT-007 |
| **우선순위** | P1 |
| **테스트 유형** | Integration |
| **사전 조건** | whisper.cpp 모델 로드, 번역 모드 활성화 |
| **테스트 절차** | 1. whisper 파라미터에 `translate: true` 설정<br>2. 한국어 오디오 입력<br>3. 전사 결과가 영어로 출력되는지 확인<br>4. 번역 품질 수동 평가 |
| **기대 결과** | - 출력 텍스트가 영어로 생성됨<br>- 원문 의미가 번역에 보존됨<br>- 번역 RTF < 1.0<br>- 실시간 스트리밍과 호환 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_translation.rs` |
| **자동화 여부** | 반자동 (번역 품질 수동 검증 필요) |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-STT-007-02: 번역 모드와 전사 모드 전환

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-007-02 |
| **요구사항 ID** | FR-STT-007 |
| **우선순위** | P1 |
| **테스트 유형** | Unit |
| **사전 조건** | whisper.cpp 모델 로드 |
| **테스트 절차** | 1. 전사 모드로 첫 번째 청크 처리<br>2. 번역 모드로 전환<br>3. 두 번째 청크를 번역 모드로 처리<br>4. 다시 전사 모드로 전환 후 세 번째 청크 처리 |
| **기대 결과** | - 모드 전환이 즉시 반영됨<br>- 전사 모드: 원어 출력<br>- 번역 모드: 영어 출력<br>- 전환 시 세션 상태 유지 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_translation.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-STT-008: Tauri event emit 스트리밍 - P0

### TC-STT-008-01: Tauri 이벤트로 전사 결과 스트리밍

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-008-01 |
| **요구사항 ID** | FR-STT-008 |
| **우선순위** | P0 |
| **테스트 유형** | Integration |
| **사전 조건** | Tauri 앱 컨텍스트, 전사 파이프라인 초기화 |
| **테스트 절차** | 1. 프론트엔드에서 `listen("stt:partial")` 이벤트 리스너 등록<br>2. `listen("stt:final")` 이벤트 리스너 등록<br>3. 녹음 시작 후 10초 동안 실행<br>4. 수신된 이벤트 목록 및 페이로드 기록 |
| **기대 결과** | - `stt:partial` 이벤트가 200ms~1초 간격으로 수신<br>- `stt:final` 이벤트에 확정된 텍스트 포함<br>- 페이로드 JSON 구조: `{ text, timestamp, is_final, language }`<br>- 이벤트 순서가 시간순으로 정렬 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_tauri_event.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-STT-008-02: 이벤트 페이로드 직렬화/역직렬화

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-008-02 |
| **요구사항 ID** | FR-STT-008 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 없음 |
| **테스트 절차** | 1. `SttEvent` 구조체 생성 (text, timestamp, is_final, language)<br>2. `serde_json::to_string()` 으로 직렬화<br>3. JSON 문자열을 `serde_json::from_str()` 로 역직렬화<br>4. 원본과 복원 데이터 비교 |
| **기대 결과** | - 직렬화/역직렬화 라운드트립 성공<br>- 모든 필드 값이 일치<br>- UTF-8 한국어 텍스트 정상 처리<br>- 타임스탬프 정밀도 유지 (ms 단위) |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_tauri_event.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-STT-009: 파일 업로드 일괄 전사 - P1

### TC-STT-009-01: 오디오 파일 업로드 및 전사

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-009-01 |
| **요구사항 ID** | FR-STT-009 |
| **우선순위** | P1 |
| **테스트 유형** | Integration |
| **사전 조건** | WAV, MP3, M4A, OGG 형식 테스트 파일 준비 |
| **테스트 절차** | 1. WAV 파일 업로드 → 전사 실행<br>2. MP3 파일 업로드 → 전사 실행<br>3. M4A 파일 업로드 → 전사 실행<br>4. OGG 파일 업로드 → 전사 실행<br>5. 각 결과의 전사 품질 확인 |
| **기대 결과** | - 4개 포맷 모두 전사 성공<br>- 전사 결과 텍스트가 비어있지 않음<br>- 파일 디코딩 에러 없음<br>- 처리 진행률이 0~100% 정상 보고 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_batch_transcribe.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-STT-009-02: 대용량 파일 일괄 전사

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-009-02 |
| **요구사항 ID** | FR-STT-009 |
| **우선순위** | P1 |
| **테스트 유형** | Performance |
| **사전 조건** | 60분 분량의 오디오 파일 준비 |
| **테스트 절차** | 1. 60분 오디오 파일 업로드<br>2. 일괄 전사 실행<br>3. 처리 시간 측정<br>4. 메모리 사용량 모니터링<br>5. 전사 결과 완전성 확인 |
| **기대 결과** | - 60분 파일 전사 완료 (OOM 없음)<br>- 메모리 사용량 < 2GB<br>- RTF < 0.5 (GPU 가속 시)<br>- 전사 결과가 파일 전체를 커버 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_batch_transcribe.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-STT-009-03: 일괄 전사 E2E 워크플로우

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-009-03 |
| **요구사항 ID** | FR-STT-009 |
| **우선순위** | P1 |
| **테스트 유형** | E2E (Playwright) |
| **사전 조건** | 앱 실행 중, 테스트 오디오 파일 준비 |
| **테스트 절차** | 1. Playwright로 파일 업로드 버튼 클릭<br>2. 파일 선택 다이얼로그에서 테스트 파일 선택<br>3. 업로드 진행률 바 표시 확인<br>4. 전사 완료 후 결과 텍스트 표시 확인<br>5. 결과 내보내기 기능 테스트 |
| **기대 결과** | - 파일 업로드 UI 정상 동작<br>- 진행률 바 0~100% 표시<br>- 전사 완료 알림 표시<br>- 결과 텍스트가 편집 가능한 영역에 표시 |
| **테스트 코드 위치** | `tests/e2e/stt/test_batch_upload.spec.ts` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-STT-010: 외부 STT API 연동 - P1

### TC-STT-010-01: 외부 STT API 호출 및 결과 수신

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-010-01 |
| **요구사항 ID** | FR-STT-010 |
| **우선순위** | P1 |
| **테스트 유형** | Integration |
| **사전 조건** | 외부 STT API 키 설정, wiremock-rs 모킹 서버 준비 |
| **테스트 절차** | 1. wiremock-rs로 STT API 모킹 서버 시작<br>2. 테스트 오디오를 외부 API에 전송<br>3. 응답 수신 및 파싱<br>4. 로컬 전사 결과와 비교 |
| **기대 결과** | - API 호출 성공 (HTTP 200)<br>- 응답 JSON 파싱 성공<br>- 전사 결과 텍스트 수신<br>- 타임아웃 처리 정상 동작 (30초) |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_external_api.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-STT-010-02: 외부 API 장애 시 로컬 폴백

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-STT-010-02 |
| **요구사항 ID** | FR-STT-010 |
| **우선순위** | P1 |
| **테스트 유형** | Unit |
| **사전 조건** | 로컬 whisper.cpp 모델 로드, 외부 API 설정 |
| **테스트 절차** | 1. wiremock-rs에서 API 응답을 500 에러로 설정<br>2. 외부 API로 전사 요청<br>3. 실패 감지 후 로컬 엔진으로 자동 전환되는지 확인<br>4. 로컬 전사 결과 반환 확인 |
| **기대 결과** | - API 실패 후 3초 이내 로컬 폴백<br>- 로컬 전사 결과 정상 반환<br>- 사용자에게 폴백 알림<br>- 재시도 로직: 최대 3회 후 폴백 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/stt/test_external_api.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |
