//! E2E: CRDT 동기화 시뮬레이션 테스트
//!
//! 두 디바이스가 동시에 편집 → 동기화 → 수렴하는 시나리오

#[cfg(feature = "sync")]
mod sync_tests {
    use voxnote_core::sync::crdt::CrdtDocument;
    use voxnote_core::sync::encryption::SyncEncryption;
    use voxnote_core::sync::key_exchange;
    use voxnote_core::storage::crypto::CryptoLayer;

    #[test]
    fn e2e_sync_two_device_convergence() {
        let doc_a = CrdtDocument::new();
        let doc_b = CrdtDocument::new();

        // 디바이스 A: 초기 노트 작성
        doc_a.set_text("title", "Team Meeting");
        doc_a.set_text("notes", "Action item 1: Review PRs");

        // A→B 초기 동기화
        let state_a = doc_a.encode_state();
        doc_b.apply_update(&state_a).unwrap();
        assert_eq!(doc_b.get_text("title"), "Team Meeting");

        // 동시 편집
        doc_a.set_text("notes", "Action item 1: Review PRs\nAction item 2: Deploy");
        doc_b.set_text("notes", "Action item 1: Review PRs\nAction item 3: Test");

        // 양방향 동기화
        let sv_b = doc_b.state_vector();
        let diff_a = doc_a.encode_diff(&sv_b).unwrap();
        doc_b.apply_update(&diff_a).unwrap();

        let sv_a = doc_a.state_vector();
        let diff_b = doc_b.encode_diff(&sv_a).unwrap();
        doc_a.apply_update(&diff_b).unwrap();

        // CRDT 수렴: 양쪽이 동일한 최종 상태
        assert_eq!(doc_a.get_text("notes"), doc_b.get_text("notes"));
        assert_eq!(doc_a.get_text("title"), doc_b.get_text("title"));
    }

    #[test]
    fn e2e_sync_encrypted_delta_exchange() {
        let crypto = CryptoLayer::from_key(&[42u8; 32]).unwrap();
        let sync_enc = SyncEncryption::new(crypto);

        let doc_a = CrdtDocument::new();
        doc_a.set_text("notes", "Confidential meeting content");

        let state = doc_a.encode_state();

        // 델타 암호화
        let encrypted = sync_enc.encrypt_delta(&state).unwrap();
        assert_ne!(encrypted, state, "Encrypted should differ from plaintext");

        // 델타 복호화
        let decrypted = sync_enc.decrypt_delta(&encrypted).unwrap();

        // 복호화된 델타로 문서 복원
        let doc_b = CrdtDocument::new();
        doc_b.apply_update(&decrypted).unwrap();
        assert_eq!(doc_b.get_text("notes"), "Confidential meeting content");
    }

    #[test]
    fn e2e_sync_device_pairing() {
        let keypair_a = key_exchange::generate_device_keypair("macbook-pro");
        let keypair_b = key_exchange::generate_device_keypair("iphone-15");

        // 각 디바이스의 페어링 코드
        let code_a = key_exchange::generate_pairing_code(&keypair_a.device_id, &keypair_a.public_key);
        let code_b = key_exchange::generate_pairing_code(&keypair_b.device_id, &keypair_b.public_key);

        assert_eq!(code_a.len(), 6);
        assert_eq!(code_b.len(), 6);
        assert_ne!(code_a, code_b, "Different devices should have different codes");

        // 같은 입력은 같은 코드 (결정론적)
        let code_a2 = key_exchange::generate_pairing_code(&keypair_a.device_id, &keypair_a.public_key);
        assert_eq!(code_a, code_a2);
    }
}

// sync feature 없이도 테스트가 통과하도록
#[cfg(not(feature = "sync"))]
#[test]
fn e2e_sync_feature_gated() {
    // sync feature가 비활성화 상태에서도 다른 테스트에 영향 없음
    assert!(true);
}
