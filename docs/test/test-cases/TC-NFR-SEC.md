# TC-NFR-SEC: 보안 비기능 요구사항 테스트 케이스

| 항목 | 내용 |
|------|------|
| **문서 ID** | TC-NFR-SEC |
| **SRS 참조** | VoxNote SRS v1.0 - NFR-SEC (NFR-SEC-001 ~ NFR-SEC-008) |
| **작성일** | 2026-03-27 |
| **상태** | 초안 |
| **테스트 코드 위치** | `tests/nfr/sec/`, `crates/voxnote-core/tests/security/` |

## 테스트 요약

| 테스트 유형 | 개수 |
|-------------|------|
| Integration | 6 |
| 코드 리뷰 / 정적 분석 | 5 |
| 침투 테스트 | 3 |
| 퍼징 (Fuzz) | 3 |
| UI / E2E | 2 |
| Manual | 5 |
| **합계** | **24** |

---

## NFR-SEC-001: 로컬 모드 데이터 비전송 - P0

### TC-NFR-SEC-001-01: tcpdump 네트워크 패킷 캡처 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-001-01 |
| **요구사항 ID** | NFR-SEC-001 |
| **테스트 유형** | Integration (네트워크 모니터링) |
| **사전 조건** | - 로컬 전용 모드 설정<br>- tcpdump/Wireshark 설치<br>- 테스트 오디오 파일 준비 |
| **테스트 절차** | 1. tcpdump 캡처 시작 (`tcpdump -i any -w capture.pcap`)<br>2. 앱을 로컬 전용 모드로 실행<br>3. 녹음 → 전사 → 요약 전체 워크플로 수행<br>4. tcpdump 캡처 종료<br>5. pcap 파일 분석: 외부 IP로의 아웃바운드 패킷 검사 |
| **기대 결과** | - 외부 서버로의 HTTP/HTTPS 요청 0건<br>- DNS 쿼리 중 VoxNote 관련 도메인 0건<br>- 로컬호스트(127.0.0.1) 외 통신 없음<br>- 오디오/텍스트 데이터 포함 패킷 0건 |
| **테스트 코드 위치** | `tests/nfr/sec/local_mode_network.sh` |
| **자동화 여부** | 반자동 (스크립트 실행, 결과 수동 검증) |
| **플랫폼** | macOS, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-SEC-001-02: 방화벽 차단 상태에서 로컬 모드 정상 동작

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-001-02 |
| **요구사항 ID** | NFR-SEC-001 |
| **테스트 유형** | Integration |
| **사전 조건** | - 로컬 전용 모드 설정<br>- 방화벽으로 앱의 모든 아웃바운드 트래픽 차단 |
| **테스트 절차** | 1. OS 방화벽에서 VoxNote 바이너리 아웃바운드 차단<br>2. 앱 실행 및 로컬 모드 확인<br>3. 녹음 → 전사 → 요약 워크플로 전체 수행<br>4. 에러 로그 확인<br>5. 모든 기능 정상 동작 검증 |
| **기대 결과** | - 모든 로컬 기능 100% 정상 동작<br>- 네트워크 관련 에러 로그 0건<br>- 전사 정확도 네트워크 환경과 동일<br>- 요약 품질 로컬 모델 기준 충족 |
| **테스트 코드 위치** | `tests/nfr/sec/local_mode_firewall.sh` |
| **자동화 여부** | 반자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-SEC-001-03: 로컬 모드에서 텔레메트리/분석 데이터 전송 차단

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-001-03 |
| **요구사항 ID** | NFR-SEC-001 |
| **테스트 유형** | 코드 리뷰 / 정적 분석 |
| **사전 조건** | - 소스 코드 접근 가능<br>- 정적 분석 도구 설치 (cargo-audit, semgrep) |
| **테스트 절차** | 1. 소스 코드에서 HTTP 클라이언트 사용처 전수 조사<br>2. 텔레메트리/분석 관련 코드 경로 검사<br>3. `#[cfg(feature = "local-only")]` 시 네트워크 코드 비활성화 확인<br>4. semgrep 규칙으로 의도치 않은 네트워크 호출 탐지 |
| **기대 결과** | - 로컬 모드 빌드 시 네트워크 관련 코드 컴파일 제외<br>- 텔레메트리 전송 코드 부재 확인<br>- 조건부 컴파일 누락 지점 0건 |
| **테스트 코드 위치** | `tests/nfr/sec/code_review_network.md` |
| **자동화 여부** | 반자동 (semgrep 자동, 리뷰 수동) |
| **플랫폼** | 전체 |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-SEC-002: age E2EE 암호화 저장 - P0

### TC-NFR-SEC-002-01: DB 파일 직접 조회 시 평문 노출 불가

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-002-01 |
| **요구사항 ID** | NFR-SEC-002 |
| **테스트 유형** | Integration |
| **사전 조건** | - 앱에서 전사 결과 저장 완료<br>- 알려진 테스트 문구("VoxNote 보안 테스트 문자열") 포함 |
| **테스트 절차** | 1. 알려진 문구를 포함하는 녹음 전사 및 저장<br>2. 앱 종료<br>3. DB 파일을 직접 열기 (sqlite3 / hexdump)<br>4. 알려진 문구 문자열 검색<br>5. 바이너리 패턴 검색 (UTF-8, UTF-16) |
| **기대 결과** | - DB 파일 내 평문 문자열 발견 0건<br>- age 암호화 헤더 (`age-encryption.org`) 확인<br>- hexdump에서 알려진 문구 패턴 미발견 |
| **테스트 코드 위치** | `tests/nfr/sec/e2ee_db_check.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-SEC-002-02: age 암호화/복호화 라운드트립 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-002-02 |
| **요구사항 ID** | NFR-SEC-002 |
| **테스트 유형** | Integration |
| **사전 조건** | - age 키 쌍 생성 완료<br>- 다양한 크기의 테스트 데이터 준비 (1KB, 1MB, 100MB) |
| **테스트 절차** | 1. 테스트 데이터 age 암호화<br>2. 암호화된 데이터를 디스크에 저장<br>3. 디스크에서 읽어 age 복호화<br>4. 원본 데이터와 바이트 단위 비교<br>5. 잘못된 키로 복호화 시도 |
| **기대 결과** | - 암호화 → 복호화 후 원본과 100% 일치<br>- 모든 크기에서 정상 동작<br>- 잘못된 키로 복호화 시 명확한 에러 반환<br>- 부분 손상된 데이터 복호화 시 에러 반환 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/security/test_age_roundtrip.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-SEC-003: 서버 평문 접근 불가 - P0

### TC-NFR-SEC-003-01: 서버 코드 리뷰 - 평문 접근 경로 부재 확인

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-003-01 |
| **요구사항 ID** | NFR-SEC-003 |
| **테스트 유형** | 코드 리뷰 / 정적 분석 |
| **사전 조건** | - 서버 사이드 코드 접근 가능<br>- 동기화 프로토콜 문서 참조 |
| **테스트 절차** | 1. 서버 코드에서 복호화 키/함수 사용처 검색<br>2. 데이터 수신 → 저장 경로에서 복호화 여부 확인<br>3. 로그에 평문 데이터 기록 여부 확인<br>4. 메모리 덤프에서 평문 잔류 가능성 검토 |
| **기대 결과** | - 서버 코드에 age 복호화 키/로직 부재<br>- 서버 로그에 사용자 데이터 평문 기록 없음<br>- 전송 데이터가 E2EE 상태로 저장됨을 확인 |
| **테스트 코드 위치** | `tests/nfr/sec/server_code_review.md` |
| **자동화 여부** | 수동 (코드 리뷰) |
| **플랫폼** | 서버 |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-SEC-003-02: 서버 침투 테스트 - 암호화 데이터 접근 시도

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-003-02 |
| **요구사항 ID** | NFR-SEC-003 |
| **테스트 유형** | 침투 테스트 |
| **사전 조건** | - 스테이징 서버 환경 준비<br>- 테스트 데이터 동기화 완료<br>- 침투 테스트 도구 준비 (Burp Suite 등) |
| **테스트 절차** | 1. 서버 API를 통해 저장된 데이터 요청<br>2. 응답 데이터가 암호화 상태인지 확인<br>3. SQL 인젝션 등으로 직접 DB 접근 시도<br>4. 서버 관리자 권한으로 데이터 접근 시도<br>5. 메모리 덤프를 통한 평문 추출 시도 |
| **기대 결과** | - API 응답 데이터가 age 암호화 상태<br>- SQL 인젝션 등 공격 차단<br>- 관리자 권한으로도 평문 접근 불가<br>- 메모리에 평문 데이터 잔류 없음 |
| **테스트 코드 위치** | `tests/nfr/sec/pentest_server_report.md` |
| **자동화 여부** | 수동 (침투 테스트) |
| **플랫폼** | 서버 |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-SEC-003-03: 동기화 프로토콜 E2EE 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-003-03 |
| **요구사항 ID** | NFR-SEC-003 |
| **테스트 유형** | Integration |
| **사전 조건** | - 클라이언트 2대 + 서버 환경<br>- 프록시 서버 (mitmproxy) 설치 |
| **테스트 절차** | 1. 클라이언트 A에서 노트 생성 및 동기화<br>2. mitmproxy로 동기화 트래픽 캡처<br>3. 캡처된 페이로드에서 평문 검색<br>4. 클라이언트 B에서 수신 및 복호화 확인<br>5. 서버 DB에서 해당 데이터 직접 조회 |
| **기대 결과** | - 전송 중 페이로드가 age 암호화 상태<br>- 클라이언트 B에서 정상 복호화<br>- 서버 DB에 암호화된 blob만 존재<br>- MITM 공격으로 평문 추출 불가 |
| **테스트 코드 위치** | `tests/nfr/sec/sync_e2ee_verify.rs` |
| **자동화 여부** | 반자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-SEC-004: API 키 키체인 암호화 저장 - P0

### TC-NFR-SEC-004-01: 파일시스템 스캔 - API 키 평문 노출 검사

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-004-01 |
| **요구사항 ID** | NFR-SEC-004 |
| **테스트 유형** | Integration |
| **사전 조건** | - 테스트용 API 키 설정 완료 (`sk-test-1234567890abcdef`)<br>- 앱 데이터 디렉토리 경로 확인 |
| **테스트 절차** | 1. 앱에서 테스트 API 키 저장<br>2. 앱 종료<br>3. 앱 데이터 디렉토리 전체 파일 스캔<br>4. 모든 파일에서 API 키 문자열 검색<br>5. 설정 파일, DB 파일, 로그 파일 개별 확인 |
| **기대 결과** | - 파일시스템 어디에서도 API 키 평문 미발견<br>- 설정 파일에 키체인 참조만 존재<br>- 로그 파일에 API 키 마스킹 처리 확인<br>- 임시 파일에도 평문 없음 |
| **테스트 코드 위치** | `tests/nfr/sec/api_key_filesystem_scan.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-SEC-004-02: OS 키체인 통합 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-004-02 |
| **요구사항 ID** | NFR-SEC-004 |
| **테스트 유형** | Integration |
| **사전 조건** | - OS별 키체인 접근 권한 설정<br>  (macOS: Keychain Access, Windows: Credential Manager, Linux: libsecret) |
| **테스트 절차** | 1. 앱에서 API 키 저장<br>2. OS 키체인 도구로 저장된 항목 확인<br>3. 앱에서 API 키 읽기 및 원본 비교<br>4. OS 키체인에서 직접 삭제 후 앱 동작 확인<br>5. 앱에서 API 키 삭제 후 키체인 확인 |
| **기대 결과** | - OS 키체인에 VoxNote 항목 존재<br>- 앱 읽기 시 원본과 일치<br>- 키체인 삭제 시 앱에서 재설정 안내<br>- 앱 삭제 시 키체인 항목도 정리 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/security/test_keychain.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-SEC-004-03: 다른 앱에서 VoxNote 키체인 항목 접근 차단

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-004-03 |
| **요구사항 ID** | NFR-SEC-004 |
| **테스트 유형** | 침투 테스트 |
| **사전 조건** | - VoxNote API 키가 키체인에 저장된 상태<br>- 별도의 테스트 바이너리 준비 |
| **테스트 절차** | 1. 별도 바이너리에서 VoxNote 키체인 항목 읽기 시도<br>2. 다른 앱 서명으로 접근 시도<br>3. macOS: 키체인 접근 제어 목록(ACL) 확인<br>4. 루트 권한으로 접근 시도 |
| **기대 결과** | - 다른 앱에서 키체인 항목 접근 차단<br>- macOS ACL에 VoxNote만 허용 확인<br>- 접근 시도 시 OS 인증 프롬프트 발생 |
| **테스트 코드 위치** | `tests/nfr/sec/keychain_isolation_test.rs` |
| **자동화 여부** | 반자동 |
| **플랫폼** | macOS, Windows |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-SEC-005: Argon2 키 파생 - P0

### TC-NFR-SEC-005-01: Argon2id 파라미터 구현 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-005-01 |
| **요구사항 ID** | NFR-SEC-005 |
| **테스트 유형** | 코드 리뷰 / 정적 분석 |
| **사전 조건** | - 소스 코드 접근 가능<br>- OWASP Argon2 권장 파라미터 참조 |
| **테스트 절차** | 1. Argon2 구현 코드 위치 확인<br>2. Argon2id 변형 사용 여부 확인<br>3. 파라미터 검증: 메모리 비용 ≥ 64MB, 반복 횟수 ≥ 3, 병렬도 ≥ 4<br>4. 솔트 생성: CSPRNG 사용 여부, 최소 16바이트<br>5. 출력 키 길이 ≥ 32바이트 확인 |
| **기대 결과** | - Argon2id 변형 사용 확인<br>- 메모리 비용 ≥ 64MB<br>- 반복 횟수 ≥ 3<br>- 솔트: CSPRNG 생성, ≥ 16바이트<br>- 출력 키 길이 ≥ 32바이트 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/security/test_argon2_params.rs` |
| **자동화 여부** | 자동 (파라미터 검증) + 수동 (코드 리뷰) |
| **플랫폼** | 전체 |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-SEC-005-02: Argon2 키 파생 결정성 및 호환성

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-005-02 |
| **요구사항 ID** | NFR-SEC-005 |
| **테스트 유형** | Integration |
| **사전 조건** | - Argon2 참조 구현 테스트 벡터 준비<br>- RFC 9106 테스트 벡터 참조 |
| **테스트 절차** | 1. RFC 9106 테스트 벡터로 키 파생 실행<br>2. 참조 출력과 바이트 단위 비교<br>3. 동일 입력에 대한 반복 실행 결정성 확인<br>4. 다른 플랫폼에서 동일 결과 생성 확인 |
| **기대 결과** | - RFC 9106 테스트 벡터와 100% 일치<br>- 100회 반복 시 동일 출력<br>- macOS/Windows/Linux 동일 결과 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/security/test_argon2_vectors.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-SEC-005-03: Argon2 브루트포스 저항성 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-005-03 |
| **요구사항 ID** | NFR-SEC-005 |
| **테스트 유형** | Benchmark |
| **사전 조건** | - 현재 Argon2 파라미터 설정 확인<br>- 벤치마크 환경 준비 |
| **테스트 절차** | 1. 현재 파라미터로 단일 키 파생 시간 측정<br>2. 1,000회 키 파생 시도 시간 측정<br>3. GPU 가속 공격 시뮬레이션 (hashcat 등)<br>4. 초당 시도 가능 횟수 산출 |
| **기대 결과** | - 단일 키 파생 시간 ≥ 500ms<br>- CPU에서 초당 시도 ≤ 2회<br>- 메모리 제약으로 GPU 병렬 공격 비효율 확인 |
| **테스트 코드 위치** | `tests/nfr/sec/argon2_bruteforce_bench.rs` |
| **자동화 여부** | 반자동 |
| **플랫폼** | macOS, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-SEC-006: ChaCha20-Poly1305 암호화 - P0

### TC-NFR-SEC-006-01: ChaCha20-Poly1305 구현 정합성 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-006-01 |
| **요구사항 ID** | NFR-SEC-006 |
| **테스트 유형** | 코드 리뷰 / 정적 분석 |
| **사전 조건** | - 소스 코드 접근 가능<br>- RFC 8439 테스트 벡터 준비 |
| **테스트 절차** | 1. ChaCha20-Poly1305 구현 코드 확인<br>2. RFC 8439 테스트 벡터로 암호화/복호화 검증<br>3. 넌스(nonce) 생성: CSPRNG 사용 여부, 고유성 확인<br>4. 인증 태그 검증 로직 확인<br>5. 넌스 재사용 방지 메커니즘 검토 |
| **기대 결과** | - RFC 8439 테스트 벡터와 100% 일치<br>- 넌스: CSPRNG 생성, 12바이트<br>- 인증 태그 검증 실패 시 복호화 거부<br>- 넌스 재사용 방지 확인 (카운터/랜덤) |
| **테스트 코드 위치** | `crates/voxnote-core/tests/security/test_chacha20_vectors.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | 전체 |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-SEC-006-02: 암호문 변조 탐지 (Poly1305 무결성)

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-006-02 |
| **요구사항 ID** | NFR-SEC-006 |
| **테스트 유형** | Integration |
| **사전 조건** | - 암호화된 테스트 데이터 준비 |
| **테스트 절차** | 1. 데이터 암호화 후 저장<br>2. 암호문 1바이트 변조<br>3. 변조된 암호문 복호화 시도<br>4. 인증 태그 변조 후 복호화 시도<br>5. 넌스 변조 후 복호화 시도 |
| **기대 결과** | - 암호문 변조: 복호화 실패 + `AuthenticationError` 반환<br>- 태그 변조: 복호화 실패 + 에러 반환<br>- 넌스 변조: 복호화 실패 + 에러 반환<br>- 모든 변조 시도에서 평문 유출 없음 |
| **테스트 코드 위치** | `crates/voxnote-core/tests/security/test_chacha20_tamper.rs` |
| **자동화 여부** | 자동 |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-SEC-007: 외부 API 데이터 전송 고지 - P0

### TC-NFR-SEC-007-01: 외부 API 사용 시 사용자 고지 UI 표시

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-007-01 |
| **요구사항 ID** | NFR-SEC-007 |
| **테스트 유형** | UI / E2E |
| **사전 조건** | - 외부 API Provider (OpenAI 등) 설정<br>- Playwright 테스트 프레임워크 준비 |
| **테스트 절차** | 1. 외부 API Provider를 기본으로 설정<br>2. 전사 또는 요약 기능 최초 사용 시도<br>3. 데이터 전송 고지 다이얼로그 표시 확인<br>4. "전송될 데이터" 목록 표시 확인<br>5. "동의" / "거부" 선택 후 동작 확인 |
| **기대 결과** | - 최초 사용 시 고지 다이얼로그 반드시 표시<br>- 전송 데이터 범위 명확히 표시 (오디오/텍스트/메타데이터)<br>- "거부" 시 로컬 모드로 자동 전환<br>- "동의" 기록이 설정에 저장 |
| **테스트 코드 위치** | `tests/nfr/sec/ui_data_consent.spec.ts` |
| **자동화 여부** | 자동 (Playwright) |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-SEC-007-02: Provider 변경 시 재고지 확인

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-007-02 |
| **요구사항 ID** | NFR-SEC-007 |
| **테스트 유형** | UI / E2E |
| **사전 조건** | - 기존 Provider 동의 완료 상태<br>- 새로운 Provider 추가 가능 |
| **테스트 절차** | 1. 기존 Provider (OpenAI)에서 새 Provider (Anthropic)로 변경<br>2. 새 Provider에 대한 고지 다이얼로그 표시 확인<br>3. 데이터 처리 정책 차이점 표시 확인<br>4. 로컬 → 외부 전환 시 고지 확인 |
| **기대 결과** | - Provider 변경 시 고지 다이얼로그 재표시<br>- 새 Provider의 데이터 정책 표시<br>- 로컬 → 외부 전환 시 반드시 고지<br>- 동의 이력이 Provider별로 관리 |
| **테스트 코드 위치** | `tests/nfr/sec/ui_provider_change_consent.spec.ts` |
| **자동화 여부** | 자동 (Playwright) |
| **플랫폼** | macOS, Windows, Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-SEC-008: 보안 감사 및 퍼징 - P1

### TC-NFR-SEC-008-01: cargo-fuzz 퍼징 테스트

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-008-01 |
| **요구사항 ID** | NFR-SEC-008 |
| **테스트 유형** | 퍼징 (Fuzz) |
| **사전 조건** | - cargo-fuzz 설치<br>- 퍼징 타겟 함수 정의 완료 |
| **테스트 절차** | 1. 퍼징 타겟 정의: 오디오 디코더, 암호화 모듈, CRDT 파서<br>2. `cargo fuzz run fuzz_audio_decoder -- -max_total_time=3600`<br>3. `cargo fuzz run fuzz_crypto_module -- -max_total_time=3600`<br>4. `cargo fuzz run fuzz_crdt_parser -- -max_total_time=3600`<br>5. 발견된 크래시 분석 및 분류 |
| **기대 결과** | - 각 타겟 최소 1시간 퍼징 완료<br>- 메모리 안전 위반(ASan) 크래시 0건<br>- 패닉 발생 시 안전한 에러 처리 확인<br>- 코드 커버리지 > 60% (퍼징 대상 함수) |
| **테스트 코드 위치** | `fuzz/fuzz_targets/` |
| **자동화 여부** | 자동 (CI 야간 빌드) |
| **플랫폼** | Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-SEC-008-02: AFL++ 구조적 퍼징

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-008-02 |
| **요구사항 ID** | NFR-SEC-008 |
| **테스트 유형** | 퍼징 (Fuzz) |
| **사전 조건** | - AFL++ 설치 및 계측 빌드<br>- 시드 코퍼스 준비 (유효한 오디오, 암호문, CRDT 데이터) |
| **테스트 절차** | 1. AFL++ 계측 빌드 생성<br>2. 시드 코퍼스 기반 퍼징 실행 (24시간)<br>3. 커버리지 가이드 변이 관찰<br>4. 발견된 고유 크래시 수집<br>5. 크래시 재현 및 분류 |
| **기대 결과** | - 24시간 퍼징 안정 실행<br>- 보안 관련 크래시 0건<br>- 경로 커버리지 점진적 증가<br>- 모든 발견 사항 이슈 트래커 등록 |
| **테스트 코드 위치** | `fuzz/afl_targets/` |
| **자동화 여부** | 자동 (CI 주간 빌드) |
| **플랫폼** | Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-SEC-008-03: cargo-audit 의존성 취약점 스캔

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-SEC-008-03 |
| **요구사항 ID** | NFR-SEC-008 |
| **테스트 유형** | 정적 분석 |
| **사전 조건** | - cargo-audit 설치<br>- RustSec 어드바이저리 DB 최신 상태 |
| **테스트 절차** | 1. `cargo audit` 실행<br>2. 알려진 취약점(CVE) 목록 확인<br>3. 각 취약점의 심각도 분류 (Critical/High/Medium/Low)<br>4. Critical/High 취약점 수정 계획 수립<br>5. `cargo deny check advisories` 추가 검증 |
| **기대 결과** | - Critical 취약점 0건<br>- High 취약점 0건 (또는 수정 계획 수립)<br>- 모든 의존성 최신 보안 패치 적용<br>- CI에서 매 빌드 시 자동 검사 |
| **테스트 코드 위치** | `.github/workflows/ci.yml` (security_audit 스텝) |
| **자동화 여부** | 자동 (CI) |
| **플랫폼** | 전체 |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |
