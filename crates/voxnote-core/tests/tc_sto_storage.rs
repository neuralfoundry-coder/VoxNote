//! TC-STO: 저장소 통합 테스트
//! 관련 요구사항: FR-STO-001 ~ FR-STO-007
//! 참조: docs/test/test-cases/TC-STO.md

use voxnote_core::models::{Folder, Note, NoteStatus, Segment};
use voxnote_core::storage::SqliteStore;
use chrono::Utc;

fn test_store() -> SqliteStore {
    SqliteStore::open_in_memory().expect("Failed to create in-memory store")
}

// ── TC-STO-001-01: DB 초기화 및 스키마 ──────────────────────────

#[test]
fn tc_sto_001_01_schema_has_all_tables() {
    let store = test_store();
    // 이 시점에서 migration v1이 실행됨 (SqliteStore::open_in_memory)
    // 9 테이블 + FTS5 + schema_version 존재 확인은 migration 테스트에서 커버
    // 여기서는 CRUD 가능 여부 확인
    let note = Note::new("Schema Test");
    store.insert_note(&note).unwrap();
    let fetched = store.get_note(&note.id).unwrap();
    assert!(fetched.is_some());
}

// ── TC-STO-001-02: Notes CRUD ───────────────────────────────────

#[test]
fn tc_sto_001_02_note_create_read() {
    let store = test_store();
    let note = Note::new("Create Test");
    store.insert_note(&note).unwrap();

    let fetched = store.get_note(&note.id).unwrap().unwrap();
    assert_eq!(fetched.title, "Create Test");
    assert_eq!(fetched.status, NoteStatus::Recording);
    assert!(fetched.id.len() > 10); // UUID
}

#[test]
fn tc_sto_001_02_note_update() {
    let store = test_store();
    let mut note = Note::new("Before Update");
    store.insert_note(&note).unwrap();

    note.title = "After Update".to_string();
    note.status = NoteStatus::Done;
    note.duration_ms = Some(60000);
    note.language = Some("ko".to_string());
    note.updated_at = Utc::now();
    store.update_note(&note).unwrap();

    let fetched = store.get_note(&note.id).unwrap().unwrap();
    assert_eq!(fetched.title, "After Update");
    assert_eq!(fetched.status, NoteStatus::Done);
    assert_eq!(fetched.duration_ms, Some(60000));
    assert_eq!(fetched.language.as_deref(), Some("ko"));
}

#[test]
fn tc_sto_001_02_note_delete() {
    let store = test_store();
    let note = Note::new("Delete Me");
    store.insert_note(&note).unwrap();

    store.delete_note(&note.id).unwrap();
    assert!(store.get_note(&note.id).unwrap().is_none());
}

#[test]
fn tc_sto_001_02_note_delete_nonexistent() {
    let store = test_store();
    let result = store.delete_note("nonexistent-id");
    assert!(result.is_err(), "Deleting nonexistent note should fail");
}

#[test]
fn tc_sto_001_02_note_list_ordering() {
    let store = test_store();

    for i in 0..5 {
        let note = Note::new(format!("Note {}", i));
        store.insert_note(&note).unwrap();
        // 약간의 시간 차이를 위해
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    let notes = store.list_notes(None).unwrap();
    assert_eq!(notes.len(), 5);

    // created_at DESC 정렬 확인
    for i in 1..notes.len() {
        assert!(
            notes[i - 1].created_at >= notes[i].created_at,
            "Notes should be sorted by created_at DESC"
        );
    }
}

// ── TC-STO-001-03: 동시 접근 ────────────────────────────────────

#[test]
fn tc_sto_001_03_concurrent_inserts() {
    // SqliteStore는 Mutex<Connection>이므로 동시 접근 시 직렬화됨
    let store = std::sync::Arc::new(test_store());
    let mut handles = Vec::new();

    for thread_id in 0..5 {
        let store = store.clone();
        handles.push(std::thread::spawn(move || {
            for i in 0..20 {
                let note = Note::new(format!("Thread{}-Note{}", thread_id, i));
                store.insert_note(&note).unwrap();
            }
        }));
    }

    for h in handles {
        h.join().unwrap();
    }

    let all_notes = store.list_notes(None).unwrap();
    assert_eq!(all_notes.len(), 100, "All 100 notes should be inserted");
}

// ── TC-STO-002-01: FTS5 전문검색 ────────────────────────────────

#[test]
fn tc_sto_002_01_fts5_korean_search() {
    let store = test_store();

    let note = Note::new("Korean Meeting");
    store.insert_note(&note).unwrap();

    let segments = vec![
        Segment::new(&note.id, "오늘 회의에서 프로젝트 일정을 논의했습니다", 0, 3000),
        Segment::new(&note.id, "다음 주까지 디자인 리뷰를 완료해야 합니다", 3000, 6000),
        Segment::new(&note.id, "VoxNote 개발 진행 상황을 보고했습니다", 6000, 9000),
    ];

    for seg in &segments {
        store.insert_segment(seg).unwrap();
    }

    // FTS5 unicode61 토크나이저는 CJK를 문자 단위로 토큰화
    // 접두사 검색 (*) 사용하거나 정확한 토큰 매칭 필요
    let results = store.search_transcripts("회의*").unwrap();
    assert!(!results.is_empty(), "Should find '회의*' with prefix search");
    assert!(results[0].highlight.contains("<mark>"));

    // 영어 단어는 정확 매칭
    let results = store.search_transcripts("VoxNote").unwrap();
    assert!(!results.is_empty(), "Should find 'VoxNote'");
}

#[test]
fn tc_sto_002_02_fts5_no_results() {
    let store = test_store();
    let note = Note::new("Empty Search Test");
    store.insert_note(&note).unwrap();
    store.insert_segment(&Segment::new(&note.id, "Hello world", 0, 1000)).unwrap();

    let results = store.search_transcripts("zzzznonexistent").unwrap();
    assert!(results.is_empty());
}

// ── TC-STO-004-01: E2EE 암호화/복호화 ──────────────────────────

#[test]
fn tc_sto_004_01_encryption_roundtrip() {
    use voxnote_core::storage::crypto::CryptoLayer;
    use secrecy::SecretString;

    let password = SecretString::from("my-secure-password".to_string());
    let salt = b"voxnote-salt-16bytes";

    let crypto = CryptoLayer::from_password(&password, salt).unwrap();

    // 다양한 크기의 데이터 암호화/복호화
    for size in [0, 1, 16, 256, 1024, 65536] {
        let plaintext: Vec<u8> = (0..size).map(|i| (i % 256) as u8).collect();
        let encrypted = crypto.encrypt(&plaintext).unwrap();

        // 암호문은 원문과 달라야 함 (nonce 12 + ciphertext + tag 16)
        if size > 0 {
            assert_ne!(&encrypted[12..], plaintext.as_slice());
        }
        assert!(encrypted.len() > plaintext.len(), "Encrypted should be larger");

        let decrypted = crypto.decrypt(&encrypted).unwrap();
        assert_eq!(decrypted, plaintext, "Roundtrip failed for size {}", size);
    }
}

#[test]
fn tc_sto_004_02_encryption_wrong_key_fails() {
    use voxnote_core::storage::crypto::CryptoLayer;
    use secrecy::SecretString;

    let salt = b"voxnote-salt-16bytes";
    let crypto1 = CryptoLayer::from_password(
        &SecretString::from("password-1".to_string()),
        salt,
    ).unwrap();
    let crypto2 = CryptoLayer::from_password(
        &SecretString::from("password-2".to_string()),
        salt,
    ).unwrap();

    let data = b"sensitive meeting notes";
    let encrypted = crypto1.encrypt(data).unwrap();
    let result = crypto2.decrypt(&encrypted);
    assert!(result.is_err(), "Decryption with wrong key should fail");
}

#[test]
fn tc_sto_004_03_encryption_tampered_data() {
    use voxnote_core::storage::crypto::CryptoLayer;
    use secrecy::SecretString;

    let crypto = CryptoLayer::from_password(
        &SecretString::from("test".to_string()),
        b"salt-16-bytes!!!",
    ).unwrap();

    let encrypted = crypto.encrypt(b"important data").unwrap();

    // 데이터 변조
    let mut tampered = encrypted.clone();
    if let Some(byte) = tampered.last_mut() {
        *byte ^= 0xFF;
    }

    let result = crypto.decrypt(&tampered);
    assert!(result.is_err(), "Tampered data should fail decryption");
}

// ── TC-STO-007: 폴더 구조 ──────────────────────────────────────

#[test]
fn tc_sto_007_01_folder_hierarchy() {
    let store = test_store();

    let root = Folder::new("업무", None);
    store.insert_folder(&root).unwrap();

    let sub = Folder::new("프로젝트A", Some(root.id.clone()));
    store.insert_folder(&sub).unwrap();

    // 루트 폴더 조회
    let root_folders = store.list_folders(None).unwrap();
    assert_eq!(root_folders.len(), 1);
    assert_eq!(root_folders[0].name, "업무");

    // 하위 폴더 조회
    let sub_folders = store.list_folders(Some(&root.id)).unwrap();
    assert_eq!(sub_folders.len(), 1);
    assert_eq!(sub_folders[0].name, "프로젝트A");
}

#[test]
fn tc_sto_007_02_note_in_folder() {
    let store = test_store();

    let folder = Folder::new("Meetings", None);
    store.insert_folder(&folder).unwrap();

    let mut note = Note::new("Meeting 1");
    note.folder_id = Some(folder.id.clone());
    store.insert_note(&note).unwrap();

    let notes_in_folder = store.list_notes(Some(&folder.id)).unwrap();
    assert_eq!(notes_in_folder.len(), 1);
    assert_eq!(notes_in_folder[0].title, "Meeting 1");

    // 다른 폴더에는 없어야 함
    let notes_elsewhere = store.list_notes(Some("other-folder")).unwrap();
    assert!(notes_elsewhere.is_empty());
}

// ── TC-STO: Segment 관련 ────────────────────────────────────────

#[test]
fn tc_sto_segments_ordering() {
    let store = test_store();
    let note = Note::new("Segment Order Test");
    store.insert_note(&note).unwrap();

    // 역순으로 삽입
    store.insert_segment(&Segment::new(&note.id, "Third", 6000, 9000)).unwrap();
    store.insert_segment(&Segment::new(&note.id, "First", 0, 3000)).unwrap();
    store.insert_segment(&Segment::new(&note.id, "Second", 3000, 6000)).unwrap();

    let segments = store.get_segments(&note.id).unwrap();
    assert_eq!(segments.len(), 3);
    // timestamp_ms ASC 정렬
    assert_eq!(segments[0].text, "First");
    assert_eq!(segments[1].text, "Second");
    assert_eq!(segments[2].text, "Third");
}

#[test]
fn tc_sto_cascade_delete() {
    let store = test_store();
    let note = Note::new("Cascade Test");
    store.insert_note(&note).unwrap();

    store.insert_segment(&Segment::new(&note.id, "seg1", 0, 1000)).unwrap();
    store.insert_segment(&Segment::new(&note.id, "seg2", 1000, 2000)).unwrap();

    store.delete_note(&note.id).unwrap();

    // CASCADE DELETE: 세그먼트도 삭제됨
    let segments = store.get_segments(&note.id).unwrap();
    assert!(segments.is_empty(), "Segments should be cascade deleted");
}
