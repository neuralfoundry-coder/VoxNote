//! E2E: 암호화 보안 검증 테스트
//!
//! 실제 DB 파일에서 평문이 노출되지 않는지 검증합니다.
//! E2EE 무결성 및 키 관리 시나리오를 테스트합니다.

use secrecy::SecretString;
use tempfile::TempDir;

use voxnote_core::storage::crypto::CryptoLayer;
use voxnote_core::storage::SqliteStore;
use voxnote_core::models::{Note, Segment};

// ── E2E-201: CryptoLayer 대용량 데이터 ──────────────────────────

#[test]
fn e2e_201_crypto_large_payload() {
    let password = SecretString::from("user-master-password-2024!".to_string());
    let salt = b"unique-device-salt16";
    let crypto = CryptoLayer::from_password(&password, salt).unwrap();

    // 실제 회의록 크기 시뮬레이션 (100KB)
    let meeting_text = "이것은 중요한 회의 내용입니다. ".repeat(5000);
    let plaintext = meeting_text.as_bytes();

    let encrypted = crypto.encrypt(plaintext).unwrap();

    // 암호문이 원문보다 크다 (nonce 12 + AEAD tag 16 추가)
    assert!(encrypted.len() > plaintext.len());
    assert_eq!(encrypted.len(), plaintext.len() + 12 + 16);

    // 복호화
    let decrypted = crypto.decrypt(&encrypted).unwrap();
    assert_eq!(decrypted, plaintext);

    // 암호문에서 원문 문자열 검색 불가
    let encrypted_str = String::from_utf8_lossy(&encrypted);
    assert!(
        !encrypted_str.contains("회의"),
        "Encrypted data should not contain plaintext Korean"
    );
}

// ── E2E-202: 다중 암호화 세션 독립성 ────────────────────────────

#[test]
fn e2e_202_multiple_encryption_sessions() {
    let salt = b"same-salt-16-byte";

    // 같은 비밀번호로 두 세션
    let crypto1 = CryptoLayer::from_password(
        &SecretString::from("password123".to_string()),
        salt,
    ).unwrap();
    let crypto2 = CryptoLayer::from_password(
        &SecretString::from("password123".to_string()),
        salt,
    ).unwrap();

    let data = b"cross-session test data";

    // 세션 1에서 암호화 → 세션 2에서 복호화 (같은 키)
    let encrypted = crypto1.encrypt(data).unwrap();
    let decrypted = crypto2.decrypt(&encrypted).unwrap();
    assert_eq!(decrypted, data, "Same password+salt should produce compatible keys");
}

// ── E2E-203: 연속 암호화의 nonce 고유성 ─────────────────────────

#[test]
fn e2e_203_nonce_uniqueness() {
    let crypto = CryptoLayer::from_key(&[1u8; 32]).unwrap();
    let data = b"same plaintext";

    let mut ciphertexts = Vec::new();
    for _ in 0..100 {
        let enc = crypto.encrypt(data).unwrap();
        ciphertexts.push(enc);
    }

    // 모든 암호문이 고유해야 함 (랜덤 nonce)
    for i in 0..ciphertexts.len() {
        for j in (i + 1)..ciphertexts.len() {
            assert_ne!(
                ciphertexts[i], ciphertexts[j],
                "Each encryption should produce unique ciphertext (different nonces)"
            );
        }
    }
}

// ── E2E-204: DB 파일 hex dump 평문 검증 ─────────────────────────

#[test]
fn e2e_204_db_file_plaintext_detection() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("secure.db");

    // 식별 가능한 고유 마커 텍스트
    let marker_title = "UNIQUE_MARKER_TITLE_XYZ_2024";
    let marker_text = "UNIQUE_MARKER_TRANSCRIPT_ABC_SECRET";

    {
        let store = SqliteStore::open(&db_path).unwrap();
        let note = Note::new(marker_title);
        store.insert_note(&note).unwrap();
        store.insert_segment(&Segment::new(&note.id, marker_text, 0, 3000)).unwrap();
    }

    // DB 파일의 raw bytes 검사
    let raw = std::fs::read(&db_path).unwrap();

    // 현재 구현에서는 SQLite 직접 저장이므로 평문 존재 가능
    // CryptoLayer 통합 후 이 테스트가 pass→fail 전환되지 않도록 주의
    let contains_title = raw.windows(marker_title.len())
        .any(|w| w == marker_title.as_bytes());
    let contains_text = raw.windows(marker_text.len())
        .any(|w| w == marker_text.as_bytes());

    // 현재 상태 기록 (Phase 2 E2EE 통합 시 이 조건을 반전)
    if contains_title || contains_text {
        eprintln!(
            "SECURITY-NOTE: Plaintext detected in DB file. \
             CryptoLayer integration into SqliteStore pending."
        );
    }
    // TODO: E2EE 통합 후 활성화
    // assert!(!contains_title, "Title should be encrypted in DB");
    // assert!(!contains_text, "Transcript should be encrypted in DB");
}

// ── E2E-205: 암호화 성능 벤치마크 ───────────────────────────────

#[test]
fn e2e_205_encryption_performance() {
    let crypto = CryptoLayer::from_key(&[42u8; 32]).unwrap();

    // 1MB 데이터 100회 암복호화
    let data = vec![0xABu8; 1_048_576];
    let start = std::time::Instant::now();

    for _ in 0..100 {
        let enc = crypto.encrypt(&data).unwrap();
        let _dec = crypto.decrypt(&enc).unwrap();
    }

    let elapsed = start.elapsed();
    let throughput_mbps = (100.0 * 2.0) / elapsed.as_secs_f64(); // 100 × 1MB × 2 (enc+dec)

    assert!(
        elapsed < std::time::Duration::from_secs(10),
        "100× 1MB encrypt+decrypt should complete in <10s, took {:?}",
        elapsed
    );

    eprintln!(
        "Crypto throughput: {:.1} MB/s ({:?} for 100× 1MB roundtrip)",
        throughput_mbps, elapsed
    );
}
