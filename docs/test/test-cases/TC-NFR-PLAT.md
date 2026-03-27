# TC-NFR-PLAT: 크로스플랫폼 비기능 요구사항 테스트 케이스

| 항목 | 내용 |
|------|------|
| **문서 ID** | TC-NFR-PLAT |
| **SRS 참조** | VoxNote SRS v1.0 - NFR-PLAT (NFR-PLAT-001 ~ NFR-PLAT-005) |
| **작성일** | 2026-03-27 |
| **상태** | 초안 |
| **테스트 코드 위치** | `tests/nfr/plat/`, `.github/workflows/ci-matrix.yml` |

## 테스트 요약

| 테스트 유형 | 개수 |
|-------------|------|
| CI 매트릭스 빌드 | 5 |
| Integration | 5 |
| E2E | 5 |
| **합계** | **15** |

---

## NFR-PLAT-001: macOS 최소 사양 동작 - P0

### TC-NFR-PLAT-001-01: macOS 빌드 및 기본 기능 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PLAT-001-01 |
| **요구사항 ID** | NFR-PLAT-001 |
| **테스트 유형** | CI 매트릭스 빌드 |
| **사전 조건** | - GitHub Actions macOS runner (macos-14)<br>- Xcode 최신 안정 버전<br>- Rust 툴체인: `aarch64-apple-darwin`, `x86_64-apple-darwin` |
| **테스트 절차** | 1. `cargo build --release --target aarch64-apple-darwin` 실행<br>2. `cargo build --release --target x86_64-apple-darwin` 실행<br>3. `cargo test --workspace` 전체 테스트 실행<br>4. 바이너리 크기 및 서명 확인<br>5. Metal GPU 가속 기능 활성화 확인 |
| **기대 결과** | - Apple Silicon (M1+) 네이티브 빌드 성공<br>- Intel x86_64 빌드 성공<br>- 전체 테스트 통과율 100%<br>- Metal 프레임워크 정상 링크<br>- 최소 macOS 13 (Ventura) 호환 |
| **테스트 코드 위치** | `.github/workflows/ci-matrix.yml` (macos 매트릭스) |
| **자동화 여부** | 자동 (CI) |
| **플랫폼** | macOS |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PLAT-001-02: macOS 플랫폼 통합 기능 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PLAT-001-02 |
| **요구사항 ID** | NFR-PLAT-001 |
| **테스트 유형** | Integration |
| **사전 조건** | - macOS 릴리스 빌드 설치 완료<br>- 마이크 접근 권한 허용 |
| **테스트 절차** | 1. Keychain Access 통합 테스트 (API 키 저장/조회)<br>2. CoreAudio 오디오 캡처 테스트<br>3. Metal GPU 전사 가속 테스트<br>4. 알림 센터 통합 테스트<br>5. 유니버설 바이너리 (arm64 + x86_64) 동작 확인 |
| **기대 결과** | - Keychain 저장/조회 정상<br>- CoreAudio 오디오 캡처 레이턴시 < 50ms<br>- Metal 가속 시 전사 성능 향상 확인<br>- macOS 알림 정상 표시<br>- Rosetta 2 호환성 확인 |
| **테스트 코드 위치** | `tests/nfr/plat/macos_integration.rs` |
| **자동화 여부** | 반자동 |
| **플랫폼** | macOS |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PLAT-001-03: macOS E2E 전체 워크플로 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PLAT-001-03 |
| **요구사항 ID** | NFR-PLAT-001 |
| **테스트 유형** | E2E |
| **사전 조건** | - macOS 앱 설치 및 초기 설정 완료<br>- 테스트 오디오 파일 준비 |
| **테스트 절차** | 1. 앱 실행 → 새 녹음 시작<br>2. 5분 녹음 → 전사 확인<br>3. 요약 생성 → 결과 확인<br>4. 노트 저장 → 재시작 후 데이터 유지 확인<br>5. 내보내기 (Markdown, PDF) 기능 확인 |
| **기대 결과** | - 전체 워크플로 오류 없이 완료<br>- 전사 정확도 기준 충족 (WER < 15%)<br>- 요약 생성 < 10초<br>- 재시작 후 데이터 100% 유지<br>- 내보내기 파일 정상 생성 |
| **테스트 코드 위치** | `tests/nfr/plat/macos_e2e.spec.ts` |
| **자동화 여부** | 자동 (Playwright) |
| **플랫폼** | macOS |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-PLAT-002: Windows 최소 사양 동작 - P0

### TC-NFR-PLAT-002-01: Windows 빌드 및 기본 기능 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PLAT-002-01 |
| **요구사항 ID** | NFR-PLAT-002 |
| **테스트 유형** | CI 매트릭스 빌드 |
| **사전 조건** | - GitHub Actions Windows runner (windows-latest)<br>- MSVC 툴체인<br>- Rust 타겟: `x86_64-pc-windows-msvc` |
| **테스트 절차** | 1. `cargo build --release --target x86_64-pc-windows-msvc` 실행<br>2. `cargo test --workspace` 전체 테스트 실행<br>3. Windows Defender SmartScreen 호환성 확인<br>4. WASAPI 오디오 백엔드 테스트<br>5. DirectML GPU 가속 확인 (가능 시) |
| **기대 결과** | - 빌드 성공, 전체 테스트 통과<br>- 최소 Windows 10 21H2 호환<br>- WASAPI 오디오 캡처 정상<br>- Credential Manager API 키 저장 정상<br>- 코드 서명 확인 |
| **테스트 코드 위치** | `.github/workflows/ci-matrix.yml` (windows 매트릭스) |
| **자동화 여부** | 자동 (CI) |
| **플랫폼** | Windows |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PLAT-002-02: Windows 플랫폼 통합 기능 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PLAT-002-02 |
| **요구사항 ID** | NFR-PLAT-002 |
| **테스트 유형** | Integration |
| **사전 조건** | - Windows 릴리스 빌드 설치<br>- 마이크 접근 권한 허용 |
| **테스트 절차** | 1. Windows Credential Manager 통합 테스트<br>2. WASAPI 오디오 캡처 테스트<br>3. Windows 알림 센터 통합 테스트<br>4. 한국어 IME 입력 호환성 테스트<br>5. 고DPI 디스플레이 렌더링 확인 |
| **기대 결과** | - Credential Manager 저장/조회 정상<br>- WASAPI 캡처 레이턴시 < 50ms<br>- Windows 토스트 알림 정상 표시<br>- 한국어 IME 입력 정상<br>- 150%/200% DPI 스케일링 정상 |
| **테스트 코드 위치** | `tests/nfr/plat/windows_integration.rs` |
| **자동화 여부** | 반자동 |
| **플랫폼** | Windows |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PLAT-002-03: Windows E2E 전체 워크플로 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PLAT-002-03 |
| **요구사항 ID** | NFR-PLAT-002 |
| **테스트 유형** | E2E |
| **사전 조건** | - Windows 앱 설치 및 초기 설정 완료<br>- 테스트 오디오 파일 준비 |
| **테스트 절차** | 1. 앱 실행 → 새 녹음 시작<br>2. 5분 녹음 → 전사 확인<br>3. 요약 생성 → 결과 확인<br>4. 노트 저장 → 재시작 후 데이터 유지 확인<br>5. 내보내기 기능 확인 |
| **기대 결과** | - 전체 워크플로 오류 없이 완료<br>- macOS와 동일한 기능 수준 확인<br>- 성능 기준 충족 |
| **테스트 코드 위치** | `tests/nfr/plat/windows_e2e.spec.ts` |
| **자동화 여부** | 자동 (Playwright) |
| **플랫폼** | Windows |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-PLAT-003: Linux 최소 사양 동작 - P1

### TC-NFR-PLAT-003-01: Linux 빌드 및 기본 기능 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PLAT-003-01 |
| **요구사항 ID** | NFR-PLAT-003 |
| **테스트 유형** | CI 매트릭스 빌드 |
| **사전 조건** | - GitHub Actions Linux runner (ubuntu-22.04)<br>- Rust 타겟: `x86_64-unknown-linux-gnu`<br>- 필수 라이브러리: ALSA, PulseAudio, libsecret |
| **테스트 절차** | 1. `cargo build --release --target x86_64-unknown-linux-gnu` 실행<br>2. `cargo test --workspace` 전체 테스트 실행<br>3. AppImage/Flatpak 패키징 테스트<br>4. PulseAudio/PipeWire 오디오 백엔드 테스트<br>5. Vulkan/OpenCL GPU 가속 확인 (가능 시) |
| **기대 결과** | - 빌드 성공, 전체 테스트 통과<br>- Ubuntu 22.04+ / Fedora 38+ 호환<br>- PulseAudio/PipeWire 오디오 정상<br>- libsecret 키 저장 정상<br>- AppImage 패키지 정상 실행 |
| **테스트 코드 위치** | `.github/workflows/ci-matrix.yml` (linux 매트릭스) |
| **자동화 여부** | 자동 (CI) |
| **플랫폼** | Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PLAT-003-02: Linux 플랫폼 통합 기능 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PLAT-003-02 |
| **요구사항 ID** | NFR-PLAT-003 |
| **테스트 유형** | Integration |
| **사전 조건** | - Linux 릴리스 빌드 설치<br>- ALSA/PulseAudio 설정 완료 |
| **테스트 절차** | 1. libsecret (GNOME Keyring / KWallet) 통합 테스트<br>2. PulseAudio 오디오 캡처 테스트<br>3. PipeWire 오디오 캡처 테스트<br>4. D-Bus 알림 통합 테스트<br>5. Wayland/X11 렌더링 호환성 확인 |
| **기대 결과** | - libsecret 저장/조회 정상<br>- PulseAudio/PipeWire 캡처 정상<br>- D-Bus 알림 표시<br>- Wayland 및 X11 모두 정상 렌더링<br>- HiDPI 스케일링 정상 |
| **테스트 코드 위치** | `tests/nfr/plat/linux_integration.rs` |
| **자동화 여부** | 반자동 |
| **플랫폼** | Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PLAT-003-03: Linux E2E 전체 워크플로 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PLAT-003-03 |
| **요구사항 ID** | NFR-PLAT-003 |
| **테스트 유형** | E2E |
| **사전 조건** | - Linux 앱 설치 및 초기 설정 완료<br>- 테스트 오디오 파일 준비 |
| **테스트 절차** | 1. 앱 실행 → 새 녹음 시작<br>2. 5분 녹음 → 전사 확인<br>3. 요약 생성 → 결과 확인<br>4. 노트 저장 → 재시작 후 데이터 유지 확인<br>5. 다양한 배포판 (Ubuntu, Fedora) 테스트 |
| **기대 결과** | - 전체 워크플로 오류 없이 완료<br>- 주요 배포판에서 동일 기능 수준<br>- 성능 기준 충족 |
| **테스트 코드 위치** | `tests/nfr/plat/linux_e2e.spec.ts` |
| **자동화 여부** | 자동 (Playwright) |
| **플랫폼** | Linux |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-PLAT-004: iOS 최소 사양 동작 - P1

### TC-NFR-PLAT-004-01: iOS 빌드 및 기본 기능 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PLAT-004-01 |
| **요구사항 ID** | NFR-PLAT-004 |
| **테스트 유형** | CI 매트릭스 빌드 |
| **사전 조건** | - Xcode 최신 안정 버전<br>- Rust 타겟: `aarch64-apple-ios`<br>- iOS 시뮬레이터 준비 |
| **테스트 절차** | 1. `cargo build --release --target aarch64-apple-ios` 실행<br>2. iOS 시뮬레이터에서 앱 실행<br>3. 코어 기능 (전사, 요약) 동작 확인<br>4. CoreML 통합 확인<br>5. App Store 제출 요건 검증 |
| **기대 결과** | - iOS 16+ 호환<br>- aarch64 네이티브 빌드 성공<br>- CoreML 가속 정상 동작<br>- 시뮬레이터에서 기본 기능 통과<br>- 메모리 사용량 < 300MB (모바일 제한) |
| **테스트 코드 위치** | `.github/workflows/ci-matrix.yml` (ios 매트릭스) |
| **자동화 여부** | 반자동 (CI 빌드 + 시뮬레이터 수동) |
| **플랫폼** | iOS |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PLAT-004-02: iOS 플랫폼 통합 기능 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PLAT-004-02 |
| **요구사항 ID** | NFR-PLAT-004 |
| **테스트 유형** | Integration |
| **사전 조건** | - iOS 실기기 준비<br>- 마이크 접근 권한 허용 |
| **테스트 절차** | 1. iOS Keychain 통합 테스트<br>2. AVAudioEngine 오디오 캡처 테스트<br>3. CoreML 전사 가속 테스트<br>4. 백그라운드 녹음 동작 테스트<br>5. 저전력 모드에서 동작 확인 |
| **기대 결과** | - Keychain 저장/조회 정상<br>- AVAudioEngine 캡처 정상<br>- CoreML 가속 적용 확인<br>- 백그라운드 녹음 유지<br>- 저전력 모드에서도 기본 기능 동작 |
| **테스트 코드 위치** | `tests/nfr/plat/ios_integration.swift` |
| **자동화 여부** | 반자동 |
| **플랫폼** | iOS |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PLAT-004-03: iOS E2E 전체 워크플로 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PLAT-004-03 |
| **요구사항 ID** | NFR-PLAT-004 |
| **테스트 유형** | E2E |
| **사전 조건** | - iOS 앱 설치 완료<br>- 테스트 오디오 준비 |
| **테스트 절차** | 1. 앱 실행 → 새 녹음 시작<br>2. 2분 녹음 → 전사 확인<br>3. 요약 생성 → 결과 확인<br>4. 앱 종료 → 재실행 후 데이터 유지 확인<br>5. iCloud 동기화 확인 (해당 시) |
| **기대 결과** | - 전체 워크플로 오류 없이 완료<br>- 터치 UI 반응성 < 100ms<br>- 배터리 소모 모니터링 기준 이내 |
| **테스트 코드 위치** | `tests/nfr/plat/ios_e2e.swift` |
| **자동화 여부** | 반자동 (XCUITest) |
| **플랫폼** | iOS |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

---

## NFR-PLAT-005: Android 최소 사양 동작 - P1

### TC-NFR-PLAT-005-01: Android 빌드 및 기본 기능 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PLAT-005-01 |
| **요구사항 ID** | NFR-PLAT-005 |
| **테스트 유형** | CI 매트릭스 빌드 |
| **사전 조건** | - Android NDK 설치<br>- Rust 타겟: `aarch64-linux-android`, `x86_64-linux-android`<br>- Android 에뮬레이터 준비 |
| **테스트 절차** | 1. `cargo build --release --target aarch64-linux-android` 실행<br>2. `cargo build --release --target x86_64-linux-android` 실행<br>3. Android 에뮬레이터에서 앱 실행<br>4. NNAPI 가속 확인 (가능 시)<br>5. Google Play 제출 요건 검증 |
| **기대 결과** | - Android 12 (API 31)+ 호환<br>- arm64-v8a / x86_64 빌드 성공<br>- 에뮬레이터에서 기본 기능 통과<br>- 메모리 사용량 < 300MB (모바일 제한)<br>- APK 크기 < 50MB |
| **테스트 코드 위치** | `.github/workflows/ci-matrix.yml` (android 매트릭스) |
| **자동화 여부** | 반자동 (CI 빌드 + 에뮬레이터 수동) |
| **플랫폼** | Android |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PLAT-005-02: Android 플랫폼 통합 기능 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PLAT-005-02 |
| **요구사항 ID** | NFR-PLAT-005 |
| **테스트 유형** | Integration |
| **사전 조건** | - Android 실기기 준비<br>- 마이크 접근 권한 허용 |
| **테스트 절차** | 1. Android Keystore 통합 테스트<br>2. AAudio/OpenSL ES 오디오 캡처 테스트<br>3. NNAPI 전사 가속 테스트<br>4. 포그라운드 서비스 녹음 테스트<br>5. Doze 모드에서 동작 확인 |
| **기대 결과** | - Keystore 저장/조회 정상<br>- AAudio 캡처 정상<br>- 포그라운드 서비스로 녹음 유지<br>- Doze 모드에서 예약 작업 정상<br>- 다양한 화면 크기 대응 |
| **테스트 코드 위치** | `tests/nfr/plat/android_integration.kt` |
| **자동화 여부** | 반자동 |
| **플랫폼** | Android |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |

### TC-NFR-PLAT-005-03: Android E2E 전체 워크플로 검증

| 항목 | 내용 |
|------|------|
| **테스트 ID** | TC-NFR-PLAT-005-03 |
| **요구사항 ID** | NFR-PLAT-005 |
| **테스트 유형** | E2E |
| **사전 조건** | - Android 앱 설치 완료<br>- 테스트 오디오 준비 |
| **테스트 절차** | 1. 앱 실행 → 새 녹음 시작<br>2. 2분 녹음 → 전사 확인<br>3. 요약 생성 → 결과 확인<br>4. 앱 종료 → 재실행 후 데이터 유지 확인<br>5. 다양한 기기에서 호환성 확인 |
| **기대 결과** | - 전체 워크플로 오류 없이 완료<br>- 터치 UI 반응성 < 100ms<br>- 배터리 소모 모니터링 기준 이내<br>- Pixel, Samsung Galaxy 등 주요 기기 호환 |
| **테스트 코드 위치** | `tests/nfr/plat/android_e2e.kt` |
| **자동화 여부** | 반자동 (Espresso/UI Automator) |
| **플랫폼** | Android |
| **결과** | [ ] Pass / [ ] Fail / [ ] Skip |
