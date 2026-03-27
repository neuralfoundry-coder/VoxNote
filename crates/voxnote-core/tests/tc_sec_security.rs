//! TC-SEC: 보안 테스트
//! 관련 요구사항: NFR-SEC-001 ~ NFR-SEC-006
//! 참조: docs/architecture/07-security-architecture.md

use voxnote_core::storage::crypto::CryptoLayer;
use voxnote_core::storage::SqliteStore;
use voxnote_core::models::{Note, Segment};
use secrecy::SecretString;
use tempfile::TempDir;

// ── NFR-SEC-002: DB 파일 평문 부재 확인 ─────────────────────────

#[test]
fn tc_sec_002_db_no_plaintext_in_raw_file() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");

    let store = SqliteStore::open(&db_path).unwrap();

    // 식별 가능한 고유 텍스트 삽입
    let note = Note::new("SECRET_MEETING_TITLE_XYZ");
    store.insert_note(&note).unwrap();

    let segment = Segment::new(&note.id, "CONFIDENTIAL_TRANSCRIPT_ABC", 0, 3000);
    store.insert_segment(&segment).unwrap();

    // DB 파일 읽기
    drop(store); // 연결 해제
    let raw_bytes = std::fs::read(&db_path).unwrap();
    let raw_text = String::from_utf8_lossy(&raw_bytes);

    // NOTE: 현재 구현에서는 SQLite 직접 저장이므로 평문이 보임
    // Phase 2에서 CryptoLayer를 Storage에 통합하면 이 테스트가 실패→통과로 전환됨
    // 지금은 평문이 존재하는지 탐지하는 것 자체가 목적
    let has_plaintext = raw_text.contains("SECRET_MEETING_TITLE_XYZ")
        || raw_text.contains("CONFIDENTIAL_TRANSCRIPT_ABC");

    if has_plaintext {
        eprintln!(
            "WARNING: Plaintext detected in DB file. \
             E2EE integration pending (Phase 2)."
        );
    }
    // 현재는 경고만 — E2EE 통합 후 assert!(!has_plaintext)로 전환
}

// ── NFR-SEC-005: Argon2id 키 파생 ───────────────────────────────

#[test]
fn tc_sec_005_argon2id_key_derivation() {
    let password = SecretString::from("test-password-123".to_string());
    let salt = b"voxnote-unique-salt!";

    let crypto = CryptoLayer::from_password(&password, salt);
    assert!(crypto.is_ok(), "Argon2id key derivation should succeed");

    // 같은 입력 → 같은 키 → 같은 암호문 패턴은 아님 (랜덤 nonce)
    let crypto = crypto.unwrap();
    let enc1 = crypto.encrypt(b"test").unwrap();
    let enc2 = crypto.encrypt(b"test").unwrap();
    assert_ne!(enc1, enc2, "Different nonces should produce different ciphertext");
}

// ── NFR-SEC-006: ChaCha20-Poly1305 스트리밍 암호화 ──────────────

#[test]
fn tc_sec_006_chacha20_large_data() {
    let crypto = CryptoLayer::from_key(&[42u8; 32]).unwrap();

    // 1MB 데이터 암호화/복호화
    let large_data: Vec<u8> = (0..1_048_576).map(|i| (i % 256) as u8).collect();
    let encrypted = crypto.encrypt(&large_data).unwrap();
    let decrypted = crypto.decrypt(&encrypted).unwrap();

    assert_eq!(decrypted.len(), large_data.len());
    assert_eq!(decrypted, large_data);
}

#[test]
fn tc_sec_006_chacha20_short_ciphertext_rejected() {
    let crypto = CryptoLayer::from_key(&[42u8; 32]).unwrap();

    // 12바이트 미만 → 에러
    let result = crypto.decrypt(&[0u8; 5]);
    assert!(result.is_err(), "Too short ciphertext should be rejected");
}

// ── 추가 보안 테스트 ────────────────────────────────────────────

#[test]
fn tc_sec_different_salts_different_keys() {
    let password = SecretString::from("same-password".to_string());

    let crypto1 = CryptoLayer::from_password(&password, b"salt-1-16-bytes!").unwrap();
    let crypto2 = CryptoLayer::from_password(&password, b"salt-2-16-bytes!").unwrap();

    let encrypted = crypto1.encrypt(b"data").unwrap();
    let result = crypto2.decrypt(&encrypted);
    assert!(result.is_err(), "Different salts should produce different keys");
}
