# TC-NFR-PERF: 성능 비기능 요구사항 테스트 케이스

| 항목 | 내용 |
|------|------|
| **문서 ID** | TC-NFR-PERF |
| **SRS 참조** | VoxNote SRS v1.0 - NFR-PERF (NFR-PERF-001 ~ NFR-PERF-007) |
| **작성일** | 2026-03-27 |
| **상태** | 초안 |
| **테스트 코드 위치** | `crates/voxnote-core/benches/`, `tests/nfr/perf/` |

## 테스트 요약

| 테스트 유형 | 개수 |
|-------------|------|
| Benchmark (criterion) | 10 |
| Integration | 3 |
| Manual / Script | 4 |
| CI 자동화 | 4 |
| **합계** | **21** |

---

## NFR-PERF-001: 전사 지연 < 300ms (GPU) - P0

### TC-NFR-PERF-001-01: GPU 환경 단일 청크 전사 레이턴시 벤치마크

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-001-01 |
| **요구사항 ID** | NFR-PERF-001 |
| **테스트 유형** | Benchmark (criterion) |
| **사전 조건** | - GPU 가속 활성화 (CUDA/Metal)<br>- whisper.cpp base 모델 로드 완료<br>- 2초 분량의 테스트 오디오 청크 준비 |
| **테스트 절차** | 1. criterion 벤치마크 그룹 `stt_latency` 설정<br>2. GPU 컨텍스트로 WhisperContext 초기화<br>3. 2초 오디오 청크를 100회 반복 전사<br>4. 각 반복의 레이턴시(ms) 기록<br>5. 평균, p50, p95, p99 레이턴시 산출 |
| **기대 결과** | - 평균 레이턴시 < 300ms<br>- p95 레이턴시 < 400ms<br>- p99 레이턴시 < 500ms<br>- 표준편차 < 50ms (안정성) |
| **테스트 코드 위치** | `crates/voxnote-core/benches/stt_latency.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS (Metal), Linux (CUDA) |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PERF-001-02: CPU 전용 환경 전사 레이턴시 기준선 측정

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-001-02 |
| **요구사항 ID** | NFR-PERF-001 |
| **테스트 유형** | Benchmark (criterion) |
| **사전 조건** | - GPU 가속 비활성화 (`--no-default-features`)<br>- whisper.cpp base 모델 로드 완료<br>- 2초 분량의 테스트 오디오 청크 준비 |
| **테스트 절차** | 1. criterion 벤치마크 그룹 `stt_latency_cpu` 설정<br>2. CPU 전용 컨텍스트로 WhisperContext 초기화<br>3. 2초 오디오 청크를 50회 반복 전사<br>4. 각 반복의 레이턴시(ms) 기록<br>5. GPU 결과와 비교 리포트 생성 |
| **기대 결과** | - 평균 레이턴시 < 1000ms (CPU 기준선)<br>- GPU 대비 성능 비율 기록 (참고용)<br>- 메모리 사용량 GPU 대비 동일 수준 |
| **테스트 코드 위치** | `crates/voxnote-core/benches/stt_latency.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PERF-001-03: 연속 청크 처리 시 레이턴시 안정성 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-001-03 |
| **요구사항 ID** | NFR-PERF-001 |
| **테스트 유형** | Benchmark (criterion) |
| **사전 조건** | - GPU 가속 활성화<br>- 60초 분량의 테스트 오디오 준비<br>- 2초 간격 슬라이딩 윈도우 설정 |
| **테스트 절차** | 1. 60초 오디오를 2초 청크 30개로 분할<br>2. 순차적으로 모든 청크 전사<br>3. 각 청크의 개별 레이턴시 기록<br>4. 시간 경과에 따른 레이턴시 추이 분석<br>5. 메모리 누수 여부 확인 |
| **기대 결과** | - 모든 청크 레이턴시 < 300ms<br>- 첫 번째 청크와 마지막 청크 레이턴시 차이 < 20%<br>- 메모리 사용량 증가 없음 (±5% 이내) |
| **테스트 코드 위치** | `crates/voxnote-core/benches/stt_latency.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS (Metal), Linux (CUDA) |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-PERF-002: 앱 시작 시간 < 1초 - P0

### TC-NFR-PERF-002-01: 콜드 스타트 시간 측정

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-002-01 |
| **요구사항 ID** | NFR-PERF-002 |
| **테스트 유형** | Integration / 수동 측정 |
| **사전 조건** | - 릴리스 빌드 바이너리 준비<br>- 시스템 캐시 클리어 (`purge` / `sync && echo 3 > /proc/sys/vm/drop_caches`)<br>- 모델 파일 미로드 상태 |
| **테스트 절차** | 1. 시스템 캐시 클리어<br>2. `time voxnote --headless --startup-only` 실행<br>3. 프로세스 시작부터 UI 렌더 준비 완료까지 시간 측정<br>4. 10회 반복 측정 후 평균/중앙값 산출<br>5. 스타트업 프로파일 로그 수집 |
| **기대 결과** | - 평균 콜드 스타트 시간 < 1초<br>- p95 콜드 스타트 시간 < 1.5초<br>- UI 첫 프레임 렌더까지 < 800ms |
| **테스트 코드 위치** | `tests/nfr/perf/startup_time.sh` |
| **자동화 여부** | 반자동 (스크립트 실행, 결과 수동 검증) |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PERF-002-02: 웜 스타트 시간 측정

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-002-02 |
| **요구사항 ID** | NFR-PERF-002 |
| **테스트 유형** | Integration / 수동 측정 |
| **사전 조건** | - 릴리스 빌드 바이너리 준비<br>- 이전 실행으로 시스템 캐시 워밍 상태<br>- 모델 파일 디스크 캐시 적재 완료 |
| **테스트 절차** | 1. 앱을 한 번 실행 후 종료 (캐시 워밍)<br>2. 즉시 `time voxnote --headless --startup-only` 재실행<br>3. 프로세스 시작부터 UI 렌더 준비 완료까지 시간 측정<br>4. 10회 반복 측정 후 평균 산출 |
| **기대 결과** | - 평균 웜 스타트 시간 < 500ms<br>- 콜드 스타트 대비 50% 이상 개선<br>- 모델 로드 제외 시 < 300ms |
| **테스트 코드 위치** | `tests/nfr/perf/startup_time.sh` |
| **자동화 여부** | 반자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PERF-002-03: 모바일 환경 앱 시작 시간

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-002-03 |
| **요구사항 ID** | NFR-PERF-002 |
| **테스트 유형** | Manual |
| **사전 조건** | - iOS/Android 릴리스 빌드 설치<br>- 기기 재부팅 후 테스트 |
| **테스트 절차** | 1. 앱 아이콘 탭<br>2. 스톱워치로 첫 화면 표시까지 시간 측정<br>3. 5회 반복 측정 |
| **기대 결과** | - 평균 시작 시간 < 1.5초 (모바일 허용치)<br>- 스플래시 화면 표시 < 200ms |
| **테스트 코드 위치** | 수동 테스트 (기록: `tests/nfr/perf/mobile_startup_results.md`) |
| **자동화 여부** | 수동 |
| **플랫폼** | iOS, Android |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-PERF-003: 기본 메모리 사용량 < 500MB - P1

### TC-NFR-PERF-003-01: 유휴 상태 메모리 프로파일링 (dhat)

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-003-01 |
| **요구사항 ID** | NFR-PERF-003 |
| **테스트 유형** | Integration (프로파일링) |
| **사전 조건** | - dhat 프로파일러 빌드 (`--features dhat-heap`)<br>- whisper.cpp base 모델 로드 |
| **테스트 절차** | 1. dhat 계측 빌드로 앱 실행<br>2. 모델 로드 후 유휴 상태에서 30초 대기<br>3. dhat 프로파일 출력 (`dhat-heap.json`)<br>4. 총 힙 할당량, 최대 힙 사용량, 할당 횟수 분석<br>5. 메모리 할당 상위 10개 호출 스택 추출 |
| **기대 결과** | - 최대 힙 사용량 < 500MB<br>- 모델 로드 후 유휴 시 추가 할당 < 10MB<br>- 메모리 누수 의심 할당 없음 |
| **테스트 코드 위치** | `tests/nfr/perf/memory_profile_dhat.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PERF-003-02: 장시간 운용 메모리 안정성 (jemalloc)

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-003-02 |
| **요구사항 ID** | NFR-PERF-003 |
| **테스트 유형** | Integration (프로파일링) |
| **사전 조건** | - jemalloc 빌드 (`--features jemalloc`)<br>- 1시간 분량의 녹음 시뮬레이션 데이터 준비 |
| **테스트 절차** | 1. jemalloc 통계 활성화 (`MALLOC_CONF=stats_print:true`)<br>2. 1시간 분량의 연속 전사 시뮬레이션 실행<br>3. 5분 간격으로 RSS 메모리 기록<br>4. 전사 완료 후 jemalloc 통계 출력<br>5. 메모리 사용량 추이 그래프 생성 |
| **기대 결과** | - 전체 운용 중 RSS < 500MB 유지<br>- 시간 경과에 따른 메모리 증가율 < 1MB/분<br>- 전사 완료 후 메모리 해제되어 유휴 수준 복귀 |
| **테스트 코드 위치** | `tests/nfr/perf/memory_stability_jemalloc.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PERF-003-03: 다중 탭/세션 메모리 상한 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-003-03 |
| **요구사항 ID** | NFR-PERF-003 |
| **테스트 유형** | Integration |
| **사전 조건** | - 앱 실행 상태<br>- 기존 녹음 노트 10개 이상 존재 |
| **테스트 절차** | 1. 앱 실행 후 기본 메모리 사용량 기록<br>2. 노트 10개를 동시에 열기<br>3. 각 노트 열 때마다 메모리 사용량 기록<br>4. 모든 노트 닫기 후 메모리 사용량 기록<br>5. GC 사이클 후 최종 메모리 비교 |
| **기대 결과** | - 노트 10개 열기 후 총 메모리 < 500MB<br>- 노트당 추가 메모리 < 20MB<br>- 닫기 후 메모리 해제 확인 (기본 수준 ±10%) |
| **테스트 코드 위치** | `tests/nfr/perf/memory_multi_session.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-PERF-004: 바이너리 크기 < 30MB - P1

### TC-NFR-PERF-004-01: 릴리스 바이너리 크기 확인

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-004-01 |
| **요구사항 ID** | NFR-PERF-004 |
| **테스트 유형** | CI 자동화 |
| **사전 조건** | - CI 환경에서 릴리스 빌드 완료<br>- `cargo build --release` 실행 |
| **테스트 절차** | 1. `cargo build --release` 실행<br>2. `target/release/voxnote` 바이너리 크기 측정<br>3. `strip` 적용 후 크기 재측정<br>4. 각 플랫폼별 빌드 크기 비교 |
| **기대 결과** | - strip 후 바이너리 크기 < 30MB<br>- strip 전후 크기 차이 기록 (참고용)<br>- 이전 빌드 대비 크기 증가 < 5% |
| **테스트 코드 위치** | `.github/workflows/ci.yml` (binary_size 스텝) |
| **자동화 여부** | 자동 (CI) |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PERF-004-02: 크레이트별 바이너리 기여도 분석

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-004-02 |
| **요구사항 ID** | NFR-PERF-004 |
| **테스트 유형** | CI 자동화 |
| **사전 조건** | - `cargo-bloat` 설치<br>- 릴리스 빌드 준비 |
| **테스트 절차** | 1. `cargo bloat --release --crates` 실행<br>2. 크레이트별 바이너리 기여 크기 정렬<br>3. 상위 10개 크레이트 목록 기록<br>4. 이전 빌드 대비 증감 비교 |
| **기대 결과** | - 각 크레이트의 바이너리 기여도 문서화<br>- whisper.cpp 바인딩 < 15MB<br>- UI 프레임워크 < 8MB<br>- 기타 합계 < 7MB |
| **테스트 코드 위치** | `.github/workflows/ci.yml` (bloat_analysis 스텝) |
| **자동화 여부** | 자동 (CI) |
| **플랫폼** | macOS, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-PERF-005: 1시간 STT 처리 < 8분(CPU) / < 2분(GPU) - P0

### TC-NFR-PERF-005-01: GPU 환경 1시간 오디오 전사 처리량 벤치마크

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-005-01 |
| **요구사항 ID** | NFR-PERF-005 |
| **테스트 유형** | Benchmark (criterion) |
| **사전 조건** | - GPU 가속 활성화<br>- whisper.cpp base 모델 로드<br>- 1시간 분량의 테스트 오디오 파일 준비 |
| **테스트 절차** | 1. criterion 벤치마크 그룹 `stt_throughput_gpu` 설정<br>2. 1시간 오디오 파일을 전사 파이프라인에 입력<br>3. 전체 처리 시간 측정<br>4. RTF (Real-Time Factor) 계산<br>5. GPU 사용률 모니터링 |
| **기대 결과** | - 총 처리 시간 < 2분 (120초)<br>- RTF < 0.033 (GPU)<br>- GPU 사용률 > 70% 유지<br>- VRAM 사용량 안정적 |
| **테스트 코드 위치** | `crates/voxnote-core/benches/stt_throughput.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS (Metal), Linux (CUDA) |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PERF-005-02: CPU 전용 환경 1시간 오디오 전사 처리량 벤치마크

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-005-02 |
| **요구사항 ID** | NFR-PERF-005 |
| **테스트 유형** | Benchmark (criterion) |
| **사전 조건** | - GPU 가속 비활성화<br>- whisper.cpp base 모델 로드<br>- 1시간 분량의 테스트 오디오 파일 준비<br>- CPU 스레드 수 확인 |
| **테스트 절차** | 1. criterion 벤치마크 그룹 `stt_throughput_cpu` 설정<br>2. CPU 스레드 수를 4, 8, 전체로 변경하며 테스트<br>3. 각 설정에서 1시간 오디오 전사 시간 측정<br>4. 스레드 수에 따른 확장성(scalability) 분석 |
| **기대 결과** | - 총 처리 시간 < 8분 (480초, 전체 스레드 사용)<br>- RTF < 0.133 (CPU)<br>- CPU 사용률 > 80% 유지<br>- 스레드 수 증가에 따른 준선형 성능 향상 |
| **테스트 코드 위치** | `crates/voxnote-core/benches/stt_throughput.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PERF-005-03: 다양한 오디오 포맷/품질별 처리량 비교

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-005-03 |
| **요구사항 ID** | NFR-PERF-005 |
| **테스트 유형** | Benchmark (criterion) |
| **사전 조건** | - 동일 내용의 오디오를 다양한 포맷으로 준비<br>  (WAV 16kHz, WAV 44.1kHz, MP3 128kbps, OGG) |
| **테스트 절차** | 1. 각 포맷별 10분 오디오 전사 시간 측정<br>2. 포맷별 전처리(리샘플링) 시간 별도 기록<br>3. 전사 정확도(WER) 포맷별 비교<br>4. 처리량 결과 표로 정리 |
| **기대 결과** | - 모든 포맷에서 처리 시간 기준 충족<br>- 리샘플링 오버헤드 < 전체 처리 시간의 5%<br>- 포맷 간 WER 차이 < 2% |
| **테스트 코드 위치** | `crates/voxnote-core/benches/stt_throughput.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-PERF-006: 요약 생성 < 10초 - P1

### TC-NFR-PERF-006-01: 로컬 LLM 요약 생성 레이턴시 벤치마크

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-006-01 |
| **요구사항 ID** | NFR-PERF-006 |
| **테스트 유형** | Benchmark (criterion) |
| **사전 조건** | - 로컬 LLM 모델 (llama.cpp 등) 로드 완료<br>- 2,000자 분량의 전사 텍스트 준비 |
| **테스트 절차** | 1. criterion 벤치마크 그룹 `llm_summary` 설정<br>2. 2,000자 텍스트를 요약 파이프라인에 입력<br>3. 첫 토큰 출력 시간(TTFT) 측정<br>4. 전체 요약 생성 시간 측정<br>5. 생성된 요약의 토큰 수 기록 |
| **기대 결과** | - 전체 요약 생성 시간 < 10초<br>- TTFT (Time To First Token) < 1초<br>- 생성 토큰 속도 > 20 tokens/sec<br>- 요약 길이 200~500자 |
| **테스트 코드 위치** | `crates/voxnote-core/benches/llm_summary.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PERF-006-02: 외부 API (OpenAI 등) 요약 생성 레이턴시

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-006-02 |
| **요구사항 ID** | NFR-PERF-006 |
| **테스트 유형** | Integration |
| **사전 조건** | - 외부 API 키 설정 완료<br>- 네트워크 연결 활성<br>- 2,000자 분량의 전사 텍스트 준비 |
| **테스트 절차** | 1. 외부 API Provider로 요약 요청 전송<br>2. 네트워크 왕복 시간(RTT) 별도 측정<br>3. 전체 요약 생성 시간 측정<br>4. 스트리밍 응답 시 TTFT 기록<br>5. 10회 반복하여 분산 측정 |
| **기대 결과** | - 전체 요약 생성 시간 < 10초 (네트워크 포함)<br>- TTFT < 2초<br>- 네트워크 지연 제외 시 처리 시간 < 5초<br>- 타임아웃 설정 15초에서 실패 없음 |
| **테스트 코드 위치** | `crates/voxnote-core/benches/llm_summary.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PERF-006-03: 긴 텍스트(10,000자) 요약 성능

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-006-03 |
| **요구사항 ID** | NFR-PERF-006 |
| **테스트 유형** | Benchmark (criterion) |
| **사전 조건** | - 로컬 LLM 모델 로드<br>- 10,000자 분량의 전사 텍스트 준비 |
| **테스트 절차** | 1. 10,000자 텍스트를 청킹하여 요약 파이프라인 입력<br>2. 청크별 요약 → 최종 요약 2단계 처리<br>3. 각 단계별 처리 시간 기록<br>4. 최종 요약 품질 확인 |
| **기대 결과** | - 총 요약 시간 < 30초 (긴 텍스트 허용치)<br>- 청크당 처리 시간 < 10초<br>- 메모리 사용량 < 2GB (긴 컨텍스트) |
| **테스트 코드 위치** | `crates/voxnote-core/benches/llm_summary.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-PERF-007: 외부 API 응답 시간 (Provider 의존) - P2

### TC-NFR-PERF-007-01: Mock 서버를 이용한 API 지연 시뮬레이션

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-007-01 |
| **요구사항 ID** | NFR-PERF-007 |
| **테스트 유형** | Integration |
| **사전 조건** | - Mock 서버 구동 (wiremock-rs)<br>- 다양한 지연 시나리오 설정 (100ms, 500ms, 2s, 5s, 10s) |
| **테스트 절차** | 1. wiremock-rs Mock 서버에 지연 시나리오 등록<br>2. 각 지연 수준에서 API 호출 실행<br>3. 앱 측 타임아웃 처리 동작 확인<br>4. 재시도 로직 동작 확인<br>5. UI 로딩 표시기 동작 확인 |
| **기대 결과** | - 100ms~2s: 정상 응답 처리<br>- 5s: 로딩 표시기 표시 확인<br>- 10s: 타임아웃 경고 표시<br>- 15s: 타임아웃 에러 + 로컬 폴백 제안 |
| **테스트 코드 위치** | `tests/nfr/perf/api_latency_mock.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PERF-007-02: 실제 API Provider별 응답 시간 기록

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-007-02 |
| **요구사항 ID** | NFR-PERF-007 |
| **테스트 유형** | Manual |
| **사전 조건** | - 각 Provider API 키 설정 (OpenAI, Anthropic, Groq 등)<br>- 동일한 테스트 프롬프트 준비 |
| **테스트 절차** | 1. 각 Provider에 동일한 요약 요청 전송<br>2. TTFT, 전체 응답 시간, 토큰 수 기록<br>3. 시간대별 (오전/오후/야간) 측정 비교<br>4. 결과를 Provider별 비교 표로 정리 |
| **기대 결과** | - 각 Provider별 평균 응답 시간 문서화<br>- Provider 간 성능 편차 기록<br>- 기본 추천 Provider 선정 근거 확보 |
| **테스트 코드 위치** | `tests/nfr/perf/api_provider_benchmark.rs` |
| **자동화 여부** | 반자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PERF-007-03: API 응답 지연 시 UI 반응성 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PERF-007-03 |
| **요구사항 ID** | NFR-PERF-007 |
| **테스트 유형** | Integration |
| **사전 조건** | - Mock 서버 5초 지연 설정<br>- UI 테스트 프레임워크 준비 |
| **테스트 절차** | 1. 5초 지연 Mock 서버로 API 호출 발생<br>2. API 응답 대기 중 UI 조작 시도 (스크롤, 버튼 클릭)<br>3. UI 프레임 레이트 모니터링<br>4. API 응답 수신 후 결과 표시 시간 확인 |
| **기대 결과** | - API 대기 중 UI 프레임 레이트 > 30fps<br>- UI 조작에 대한 응답 지연 < 100ms<br>- 프로그레스 표시기 정상 동작<br>- 취소 버튼 동작 확인 |
| **테스트 코드 위치** | `tests/nfr/perf/api_ui_responsiveness.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |
