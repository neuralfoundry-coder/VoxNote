# TC-AUD: 오디오 파이프라인 테스트 케이스

| 항목 | 내용 |
|------|------|
| **문서 ID** | TC-AUD |
| **SRS 참조** | VoxNote SRS v1.0 - FR-AUD (FR-AUD-001 ~ FR-AUD-010) |
| **작성일** | 2026-03-27 |
| **상태** | 초안 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/` |

## 테스트 요약

| 테스트 유형 | 개수 |
|-------------|------|
| Unit (Rust cargo test) | 14 |
| Integration (pipeline) | 8 |
| E2E (Playwright recording UI) | 4 |
| Performance (criterion resampling latency) | 4 |
| **합계** | **30** |

---

## FR-AUD-001: 마이크 캡처 (cpal) - P0 Phase 1

### TC-AUD-001-01: 기본 마이크 장치 열거 및 선택

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-001-01 |
| **요구사항 ID** | FR-AUD-001 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 시스템에 1개 이상의 오디오 입력 장치가 연결되어 있음 |
| **테스트 절차** | 1. `cpal::default_host()` 호출하여 호스트 초기화<br>2. `host.input_devices()` 호출하여 입력 장치 목록 열거<br>3. `host.default_input_device()` 호출하여 기본 장치 확인<br>4. 반환된 장치의 이름과 지원 포맷 확인 |
| **기대 결과** | - 장치 목록이 1개 이상 반환됨<br>- 기본 입력 장치가 `None`이 아님<br>- 장치 이름이 빈 문자열이 아님<br>- 지원 포맷에 f32 샘플 타입 포함 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_mic_capture.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-AUD-001-02: 마이크 오디오 스트림 캡처 시작/중지

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-001-02 |
| **요구사항 ID** | FR-AUD-001 |
| **우선순위** | P0 |
| **테스트 유형** | Integration |
| **사전 조건** | 기본 마이크 장치 사용 가능, 마이크 권한 허용 |
| **테스트 절차** | 1. `AudioCapture::new()` 로 캡처 인스턴스 생성<br>2. `capture.start()` 호출하여 녹음 시작<br>3. 2초간 대기하며 콜백으로 수신되는 샘플 버퍼 수집<br>4. `capture.stop()` 호출하여 녹음 중지<br>5. 수집된 샘플 데이터 검증 |
| **기대 결과** | - `start()` 호출 후 100ms 이내에 첫 번째 콜백 수신<br>- 2초간 수집된 샘플 수 > 0<br>- `stop()` 호출 후 추가 콜백 없음<br>- 샘플 값 범위: -1.0 ~ 1.0 (f32) |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_mic_capture.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-AUD-001-03: 마이크 권한 거부 시 에러 처리

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-001-03 |
| **요구사항 ID** | FR-AUD-001 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 마이크 접근 권한이 거부된 상태 |
| **테스트 절차** | 1. 마이크 접근 권한이 없는 상태에서 `AudioCapture::new()` 호출<br>2. 반환된 에러 타입 확인<br>3. 에러 메시지가 사용자에게 권한 요청을 안내하는지 확인 |
| **기대 결과** | - `Err(AudioError::PermissionDenied)` 반환<br>- 에러 메시지에 권한 관련 안내 포함<br>- 패닉 없이 정상적으로 에러 처리됨 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_mic_capture.rs` |
| **자동화 여부** | 수동 (권한 상태 변경 필요) |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-AUD-002: 리샘플링 16kHz mono f32 (rubato) - P0 Phase 1

### TC-AUD-002-01: 44.1kHz → 16kHz 리샘플링 정확도

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-002-01 |
| **요구사항 ID** | FR-AUD-002 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 44.1kHz 스테레오 테스트 PCM 파일 준비 |
| **테스트 절차** | 1. 44.1kHz 스테레오 f32 샘플 데이터 로드<br>2. `Resampler::new(44100, 16000)` 생성<br>3. `resampler.process(&input_samples)` 호출<br>4. 출력 샘플레이트 및 채널 수 확인<br>5. 1kHz 사인파 입력 시 출력 주파수 스펙트럼 검증 |
| **기대 결과** | - 출력 샘플레이트: 16000Hz<br>- 출력 채널: mono (1채널)<br>- 출력 포맷: f32<br>- SNR ≥ 90dB (사인파 테스트 기준)<br>- 출력 샘플 수 = 입력 샘플 수 × (16000/44100) ± 1 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_resampling.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-AUD-002-02: 48kHz → 16kHz 리샘플링

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-002-02 |
| **요구사항 ID** | FR-AUD-002 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 48kHz 모노 테스트 PCM 파일 준비 |
| **테스트 절차** | 1. 48kHz 모노 f32 샘플 데이터 로드<br>2. `Resampler::new(48000, 16000)` 생성<br>3. `resampler.process(&input_samples)` 호출<br>4. 출력 검증 |
| **기대 결과** | - 출력 샘플레이트: 16000Hz<br>- 출력 채널: mono<br>- 리샘플링 비율: 정확히 1/3<br>- SNR ≥ 90dB |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_resampling.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-AUD-002-03: 리샘플링 레이턴시 성능 벤치마크

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-002-03 |
| **요구사항 ID** | FR-AUD-002 |
| **우선순위** | P0 |
| **테스트 유형** | Performance (criterion) |
| **사전 조건** | criterion 벤치마크 환경 설정 완료 |
| **테스트 절차** | 1. 44.1kHz 1초 분량의 오디오 청크 생성<br>2. criterion 벤치마크로 `resampler.process()` 100회 반복 측정<br>3. 평균 처리 시간 및 표준편차 기록<br>4. 48kHz 입력으로도 동일 벤치마크 수행 |
| **기대 결과** | - 1초 청크 처리 시간 < 10ms (실시간 비율 < 0.01)<br>- 표준편차 < 평균의 10%<br>- 메모리 할당 안정적 (힙 할당 증가 없음) |
| **테스트 코드 위치** | `crates/voxnote-core/benches/resampling_bench.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-AUD-003: VAD 필터링 - P0 Phase 1

### TC-AUD-003-01: 음성 구간 감지 정확도

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-003-01 |
| **요구사항 ID** | FR-AUD-003 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 음성+무음 구간이 명확한 테스트 오디오 파일 준비 |
| **테스트 절차** | 1. 레이블링된 테스트 오디오 로드 (음성 3초 → 무음 2초 → 음성 4초)<br>2. `VadFilter::new(threshold: 0.5)` 생성<br>3. 오디오를 30ms 프레임 단위로 VAD에 입력<br>4. 각 프레임의 음성/비음성 판정 결과를 레이블과 비교 |
| **기대 결과** | - 음성 구간 감지율 (recall) ≥ 95%<br>- 비음성 구간 정확도 (precision) ≥ 90%<br>- 음성 시작 감지 지연 < 100ms<br>- 음성 종료 감지 지연 < 300ms |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_vad.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-AUD-003-02: VAD 무음 구간 필터링

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-003-02 |
| **요구사항 ID** | FR-AUD-003 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 무음(silence) 전용 테스트 오디오 준비 |
| **테스트 절차** | 1. 5초 분량의 무음(-60dB 이하) 오디오 생성<br>2. VAD 필터에 입력<br>3. 필터 출력 확인 |
| **기대 결과** | - 모든 프레임이 비음성으로 판정<br>- STT 엔진으로 전달되는 데이터 없음<br>- CPU 사용률 < 1% (무음 상태) |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_vad.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-AUD-003-03: VAD 임계값 조정

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-003-03 |
| **요구사항 ID** | FR-AUD-003 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 잡음이 포함된 테스트 오디오 준비 |
| **테스트 절차** | 1. 배경 잡음이 포함된 오디오 로드<br>2. threshold=0.3으로 VAD 실행 → 감지된 음성 구간 기록<br>3. threshold=0.7로 VAD 실행 → 감지된 음성 구간 기록<br>4. 두 결과 비교 |
| **기대 결과** | - 낮은 임계값(0.3): 더 많은 프레임이 음성으로 감지됨<br>- 높은 임계값(0.7): 더 적은 프레임이 음성으로 감지됨<br>- 두 결과 간 유의미한 차이 존재 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_vad.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-AUD-004: lock-free 링버퍼 - P0 Phase 1

### TC-AUD-004-01: 단일 생산자-단일 소비자 링버퍼 동작

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-004-01 |
| **요구사항 ID** | FR-AUD-004 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 없음 |
| **테스트 절차** | 1. `RingBuffer::<f32>::new(capacity: 4096)` 생성<br>2. producer 핸들로 1024 샘플 쓰기<br>3. consumer 핸들로 1024 샘플 읽기<br>4. 읽은 데이터와 쓴 데이터 비교 |
| **기대 결과** | - 읽은 데이터가 쓴 데이터와 동일<br>- 데이터 손실 없음<br>- lock-free 동작 (뮤텍스 사용 없음) |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_ringbuffer.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-AUD-004-02: 링버퍼 오버플로우 처리

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-004-02 |
| **요구사항 ID** | FR-AUD-004 |
| **우선순위** | P0 |
| **테스트 유형** | Unit |
| **사전 조건** | 없음 |
| **테스트 절차** | 1. 용량 1024의 링버퍼 생성<br>2. consumer가 읽지 않는 상태에서 2048 샘플 쓰기 시도<br>3. 쓰기 결과 및 버퍼 상태 확인 |
| **기대 결과** | - 오버플로우 시 오래된 데이터 덮어쓰기 또는 쓰기 실패 반환<br>- 패닉 없이 정상 동작<br>- 버퍼 내 데이터 일관성 유지 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_ringbuffer.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-AUD-004-03: 멀티스레드 동시 읽기/쓰기 스트레스 테스트

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-004-03 |
| **요구사항 ID** | FR-AUD-004 |
| **우선순위** | P0 |
| **테스트 유형** | Performance |
| **사전 조건** | 없음 |
| **테스트 절차** | 1. 용량 8192의 링버퍼 생성<br>2. 생산자 스레드: 100만 샘플을 256 청크로 연속 쓰기<br>3. 소비자 스레드: 동시에 256 청크로 연속 읽기<br>4. 전체 처리 시간 및 데이터 무결성 확인 |
| **기대 결과** | - 100만 샘플 전송 데이터 무결성 100%<br>- 데드락 발생 없음<br>- 처리량 > 1M samples/sec<br>- CPU 캐시 라인 경합 최소화 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_ringbuffer.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-AUD-005: macOS ScreenCaptureKit - P1 Phase 4

### TC-AUD-005-01: macOS 시스템 오디오 캡처

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-005-01 |
| **요구사항 ID** | FR-AUD-005 |
| **우선순위** | P1 |
| **테스트 유형** | Integration |
| **사전 조건** | macOS 13.0+, ScreenCaptureKit 권한 허용, 시스템 오디오 재생 중 |
| **테스트 절차** | 1. `SystemAudioCapture::new_macos()` 호출<br>2. `SCShareableContent` 로 캡처 가능한 오디오 소스 열거<br>3. 시스템 오디오 캡처 시작<br>4. 3초간 캡처된 오디오 데이터 수집<br>5. 캡처 중지 및 데이터 검증 |
| **기대 결과** | - ScreenCaptureKit API 정상 초기화<br>- 캡처된 오디오 샘플 수 > 0<br>- 샘플레이트가 시스템 설정과 일치<br>- 레이턴시 < 50ms |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_system_audio_macos.rs` |
| **자동화 여부** | 반자동 (macOS CI 환경 필요) |
| **플랫폼** | macOS |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-AUD-005-02: 마이크 + 시스템 오디오 동시 캡처

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-005-02 |
| **요구사항 ID** | FR-AUD-005 |
| **우선순위** | P1 |
| **테스트 유형** | Integration |
| **사전 조건** | macOS 13.0+, 마이크 및 ScreenCaptureKit 권한 허용 |
| **테스트 절차** | 1. 마이크 캡처 인스턴스 생성 및 시작<br>2. 시스템 오디오 캡처 인스턴스 생성 및 시작<br>3. 두 스트림을 동시에 3초간 캡처<br>4. 각 스트림의 데이터 독립성 확인 |
| **기대 결과** | - 두 스트림 모두 독립적으로 데이터 수집<br>- 스트림 간 간섭 없음<br>- 각 스트림 타임스탬프 정확<br>- 메모리 사용량 < 100MB |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_system_audio_macos.rs` |
| **자동화 여부** | 반자동 |
| **플랫폼** | macOS |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-AUD-006: Windows WASAPI Loopback - P1 Phase 4

### TC-AUD-006-01: Windows WASAPI 루프백 캡처

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-006-01 |
| **요구사항 ID** | FR-AUD-006 |
| **우선순위** | P1 |
| **테스트 유형** | Integration |
| **사전 조건** | Windows 10+, 오디오 출력 장치 활성화, 시스템 오디오 재생 중 |
| **테스트 절차** | 1. `SystemAudioCapture::new_windows()` 호출<br>2. WASAPI 루프백 모드로 오디오 렌더 엔드포인트 열기<br>3. 공유 모드(shared mode)로 캡처 시작<br>4. 3초간 데이터 수집<br>5. 캡처 중지 및 데이터 검증 |
| **기대 결과** | - WASAPI 초기화 성공<br>- 루프백 캡처 데이터 수신<br>- 샘플레이트가 시스템 출력 장치와 일치<br>- 레이턴시 < 30ms |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_system_audio_windows.rs` |
| **자동화 여부** | 반자동 (Windows CI 환경 필요) |
| **플랫폼** | Windows |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-AUD-006-02: WASAPI 오디오 장치 변경 핫스왑

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-006-02 |
| **요구사항 ID** | FR-AUD-006 |
| **우선순위** | P1 |
| **테스트 유형** | Integration |
| **사전 조건** | Windows 10+, 2개 이상의 오디오 출력 장치 |
| **테스트 절차** | 1. 기본 출력 장치로 루프백 캡처 시작<br>2. 캡처 중 기본 오디오 출력 장치를 다른 장치로 변경<br>3. 장치 변경 이벤트 수신 확인<br>4. 새 장치로 자동 전환되어 캡처 지속되는지 확인 |
| **기대 결과** | - 장치 변경 이벤트 정상 감지<br>- 캡처 중단 없이 새 장치로 전환<br>- 전환 시 오디오 갭 < 500ms<br>- 에러 없이 정상 동작 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_system_audio_windows.rs` |
| **자동화 여부** | 수동 |
| **플랫폼** | Windows |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-AUD-007: Linux PulseAudio/PipeWire - P1 Phase 4

### TC-AUD-007-01: PulseAudio 모니터 소스 캡처

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-007-01 |
| **요구사항 ID** | FR-AUD-007 |
| **우선순위** | P1 |
| **테스트 유형** | Integration |
| **사전 조건** | Linux, PulseAudio 서버 실행 중, 모니터 소스 사용 가능 |
| **테스트 절차** | 1. `SystemAudioCapture::new_linux()` 호출<br>2. PulseAudio 모니터 소스 열거<br>3. 기본 싱크의 모니터 소스로 캡처 시작<br>4. 3초간 데이터 수집<br>5. 캡처 중지 및 검증 |
| **기대 결과** | - PulseAudio 연결 성공<br>- 모니터 소스 캡처 데이터 수신<br>- 오디오 데이터 유효 (NaN/Inf 없음)<br>- 레이턴시 < 50ms |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_system_audio_linux.rs` |
| **자동화 여부** | 반자동 (Linux CI 환경 필요) |
| **플랫폼** | Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-AUD-007-02: PipeWire 호환성 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-007-02 |
| **요구사항 ID** | FR-AUD-007 |
| **우선순위** | P1 |
| **테스트 유형** | Integration |
| **사전 조건** | Linux, PipeWire가 PulseAudio 호환 모드로 실행 중 |
| **테스트 절차** | 1. PipeWire 환경에서 `SystemAudioCapture::new_linux()` 호출<br>2. PulseAudio 호환 API로 모니터 소스 캡처 시작<br>3. 3초간 데이터 수집<br>4. PipeWire 네이티브 API로도 동일 테스트 수행 |
| **기대 결과** | - PipeWire 환경에서 PulseAudio API 정상 동작<br>- 캡처 데이터 유효<br>- PulseAudio/PipeWire 자동 감지 동작 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_system_audio_linux.rs` |
| **자동화 여부** | 반자동 |
| **플랫폼** | Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-AUD-008: iOS AVAudioEngine - P1 Phase 3

### TC-AUD-008-01: AVAudioEngine 마이크 캡처

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-008-01 |
| **요구사항 ID** | FR-AUD-008 |
| **우선순위** | P1 |
| **테스트 유형** | Integration |
| **사전 조건** | iOS 16+, 마이크 권한 허용 |
| **테스트 절차** | 1. `AVAudioEngine` 인스턴스 생성<br>2. 입력 노드의 installTap으로 오디오 버퍼 수신 설정<br>3. 엔진 시작<br>4. 3초간 오디오 버퍼 수집<br>5. 엔진 정지 및 데이터 검증 |
| **기대 결과** | - AVAudioEngine 정상 시작<br>- 오디오 버퍼 콜백 수신<br>- 16kHz 리샘플링 가능<br>- 백그라운드 오디오 세션 설정 정상 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_ios_audio.rs` |
| **자동화 여부** | 반자동 (iOS 디바이스/시뮬레이터 필요) |
| **플랫폼** | iOS |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-AUD-008-02: iOS 오디오 세션 인터럽트 처리

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-008-02 |
| **요구사항 ID** | FR-AUD-008 |
| **우선순위** | P1 |
| **테스트 유형** | Integration |
| **사전 조건** | iOS 디바이스, 녹음 진행 중 |
| **테스트 절차** | 1. 오디오 캡처 시작<br>2. 전화 수신 시뮬레이션 (오디오 세션 인터럽트)<br>3. 인터럽트 종료 후 자동 복구 확인<br>4. 복구 후 캡처 데이터 유효성 검증 |
| **기대 결과** | - 인터럽트 감지 이벤트 정상 수신<br>- 인터럽트 종료 후 자동 캡처 재개<br>- 재개 후 데이터 유효<br>- 사용자에게 인터럽트 상태 알림 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_ios_audio.rs` |
| **자동화 여부** | 수동 |
| **플랫폼** | iOS |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-AUD-009: Android AudioRecord - P1 Phase 3

### TC-AUD-009-01: Android AudioRecord 캡처

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-009-01 |
| **요구사항 ID** | FR-AUD-009 |
| **우선순위** | P1 |
| **테스트 유형** | Integration |
| **사전 조건** | Android 8.0+, RECORD_AUDIO 권한 허용 |
| **테스트 절차** | 1. `AudioRecord` 인스턴스 생성 (16kHz, MONO, PCM_FLOAT)<br>2. `startRecording()` 호출<br>3. 3초간 `read()` 로 오디오 데이터 수집<br>4. `stop()` 호출<br>5. 수집된 데이터 검증 |
| **기대 결과** | - AudioRecord 초기화 성공 (STATE_INITIALIZED)<br>- 캡처 데이터 수신<br>- 샘플레이트 16kHz<br>- JNI 브릿지를 통한 Rust ↔ Java 통신 정상 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_android_audio.rs` |
| **자동화 여부** | 반자동 (Android 에뮬레이터/디바이스 필요) |
| **플랫폼** | Android |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-AUD-009-02: Android 오디오 포커스 처리

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-009-02 |
| **요구사항 ID** | FR-AUD-009 |
| **우선순위** | P1 |
| **테스트 유형** | Integration |
| **사전 조건** | Android 디바이스, 녹음 진행 중 |
| **테스트 절차** | 1. 오디오 캡처 시작<br>2. 다른 앱이 오디오 포커스를 가져가는 상황 시뮬레이션<br>3. 오디오 포커스 변경 콜백 처리 확인<br>4. 포커스 복구 후 캡처 재개 확인 |
| **기대 결과** | - 오디오 포커스 손실 감지<br>- 포커스 복구 후 자동 캡처 재개<br>- 데이터 손실 최소화<br>- ANR 발생 없음 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/audio/test_android_audio.rs` |
| **자동화 여부** | 수동 |
| **플랫폼** | Android |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## FR-AUD-010: Web MediaDevices - P2 Phase 3

### TC-AUD-010-01: WebAudio MediaDevices 마이크 캡처

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-010-01 |
| **요구사항 ID** | FR-AUD-010 |
| **우선순위** | P2 |
| **테스트 유형** | E2E (Playwright) |
| **사전 조건** | Chrome/Firefox 최신 버전, 마이크 권한 허용 |
| **테스트 절차** | 1. Playwright로 VoxNote 웹 앱 열기<br>2. `navigator.mediaDevices.getUserMedia({ audio: true })` 호출<br>3. `AudioContext` + `ScriptProcessorNode` 또는 `AudioWorklet`으로 PCM 데이터 추출<br>4. 3초간 오디오 데이터 수집<br>5. WASM 모듈로 데이터 전달 확인 |
| **기대 결과** | - MediaDevices API 정상 호출<br>- 오디오 스트림 수신<br>- WASM 모듈로 데이터 전달 성공<br>- 샘플 데이터 유효 (NaN/Inf 없음) |
| **테스트 코드 위치** | `tests/e2e/audio/test_web_audio.spec.ts` |
| **자동화 여부** | 자동 |
| **플랫폼** | Web (Chrome, Firefox, Safari) |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-AUD-010-02: Web AudioWorklet 기반 실시간 처리

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-010-02 |
| **요구사항 ID** | FR-AUD-010 |
| **우선순위** | P2 |
| **테스트 유형** | E2E (Playwright) |
| **사전 조건** | AudioWorklet 지원 브라우저, WASM 모듈 로드 완료 |
| **테스트 절차** | 1. AudioWorklet 프로세서 등록<br>2. 마이크 스트림 연결<br>3. AudioWorklet에서 WASM 리샘플러로 데이터 전달<br>4. 리샘플링된 16kHz 출력 확인 |
| **기대 결과** | - AudioWorklet 정상 등록 및 실행<br>- 128 프레임 단위 콜백 정상 수신<br>- 리샘플링 출력 16kHz mono f32<br>- 오디오 글리치 없음 |
| **테스트 코드 위치** | `tests/e2e/audio/test_web_audio.spec.ts` |
| **자동화 여부** | 자동 |
| **플랫폼** | Web (Chrome, Firefox) |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-AUD-010-03: 브라우저 탭 전환 시 캡처 지속성

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-010-03 |
| **요구사항 ID** | FR-AUD-010 |
| **우선순위** | P2 |
| **테스트 유형** | E2E (Playwright) |
| **사전 조건** | 웹 앱에서 녹음 진행 중 |
| **테스트 절차** | 1. 녹음 시작<br>2. Playwright로 새 탭 열기 (기존 탭 비활성화)<br>3. 5초 대기<br>4. 원래 탭으로 복귀<br>5. 녹음 데이터 확인 |
| **기대 결과** | - 탭 비활성화 중에도 오디오 캡처 지속<br>- AudioWorklet이 throttle되지 않음<br>- 데이터 갭 < 100ms |
| **테스트 코드 위치** | `tests/e2e/audio/test_web_audio.spec.ts` |
| **자동화 여부** | 자동 |
| **플랫폼** | Web (Chrome, Firefox) |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-AUD-010-04: Web 오디오 캡처 성능 벤치마크

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-AUD-010-04 |
| **요구사항 ID** | FR-AUD-010 |
| **우선순위** | P2 |
| **테스트 유형** | Performance |
| **사전 조건** | WASM 모듈 최적화 빌드 |
| **테스트 절차** | 1. 오디오 캡처 + 리샘플링 + VAD 파이프라인 시작<br>2. 30초간 실행하며 CPU 사용률 모니터링<br>3. WASM 메모리 사용량 추적<br>4. 프레임 드롭 횟수 기록 |
| **기대 결과** | - CPU 사용률 < 10% (단일 코어 기준)<br>- WASM 메모리 < 50MB<br>- 프레임 드롭 < 0.1%<br>- GC 포즈로 인한 글리치 없음 |
| **테스트 코드 위치** | `tests/e2e/audio/test_web_audio.spec.ts` |
| **자동화 여부** | 자동 |
| **플랫폼** | Web (Chrome) |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |
