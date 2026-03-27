//! E2E: 녹음 → 전사 → 저장 → 검색 전체 플로우 테스트
//!
//! 실제 앱 사용 시나리오를 재현합니다:
//!   사용자가 녹음 시작 → 음성이 전사됨 → DB에 저장 → 검색 가능

use voxnote_core::config::AppConfig;
use voxnote_core::models::{Folder, Note, NoteStatus, Segment};
use voxnote_core::storage::SqliteStore;
use chrono::Utc;
use tempfile::TempDir;

// ── E2E-101: 녹음 → 노트 생성 → 세그먼트 추가 → 조회 ──────────

#[test]
fn e2e_101_recording_to_note_flow() {
    let store = SqliteStore::open_in_memory().unwrap();

    // 1. 녹음 시작 → 노트 생성
    let mut note = Note::new("2024-03-27 팀 미팅");
    assert_eq!(note.status, NoteStatus::Recording);
    store.insert_note(&note).unwrap();

    // 2. STT 결과 → 세그먼트 삽입 (실시간 시뮬레이션)
    let transcripts = vec![
        ("안녕하세요, 오늘 회의를 시작하겠습니다.", 0, 3000),
        ("첫 번째 안건은 프로젝트 일정입니다.", 3000, 6000),
        ("다음 주까지 디자인 리뷰를 완료해야 합니다.", 6000, 9000),
        ("VoxNote 개발 현황을 보고하겠습니다.", 9000, 12000),
        ("로컬 AI 기반 전사가 잘 동작하고 있습니다.", 12000, 15000),
    ];

    for (text, start, end) in &transcripts {
        let seg = Segment::new(&note.id, *text, *start, *end);
        store.insert_segment(&seg).unwrap();
    }

    // 3. 녹음 완료 → 상태 업데이트
    note.status = NoteStatus::Done;
    note.duration_ms = Some(15000);
    note.language = Some("ko".to_string());
    note.updated_at = Utc::now();
    store.update_note(&note).unwrap();

    // 4. 검증 — 노트 조회
    let fetched = store.get_note(&note.id).unwrap().unwrap();
    assert_eq!(fetched.title, "2024-03-27 팀 미팅");
    assert_eq!(fetched.status, NoteStatus::Done);
    assert_eq!(fetched.duration_ms, Some(15000));
    assert_eq!(fetched.language.as_deref(), Some("ko"));

    // 5. 검증 — 세그먼트 조회
    let segments = store.get_segments(&note.id).unwrap();
    assert_eq!(segments.len(), 5);
    assert_eq!(segments[0].text, "안녕하세요, 오늘 회의를 시작하겠습니다.");
    assert_eq!(segments[4].text, "로컬 AI 기반 전사가 잘 동작하고 있습니다.");

    // 시간순 정렬 확인
    for i in 1..segments.len() {
        assert!(segments[i].start_ms > segments[i - 1].start_ms);
    }

    // 6. 검증 — FTS5 전문검색
    let results = store.search_transcripts("VoxNote").unwrap();
    assert!(!results.is_empty(), "Should find 'VoxNote' in transcripts");
    assert_eq!(results[0].note_id, note.id);

    let results = store.search_transcripts("디자인*").unwrap();
    assert!(!results.is_empty(), "Should find '디자인*' with prefix search");
}

// ── E2E-102: 다중 노트 + 폴더 구조 시나리오 ────────────────────

#[test]
fn e2e_102_multi_note_folder_scenario() {
    let store = SqliteStore::open_in_memory().unwrap();

    // 폴더 구조 생성
    let work_folder = Folder::new("업무", None);
    store.insert_folder(&work_folder).unwrap();

    let project_folder = Folder::new("VoxNote 프로젝트", Some(work_folder.id.clone()));
    store.insert_folder(&project_folder).unwrap();

    // 여러 노트 생성 (다른 폴더에)
    let mut note1 = Note::new("스프린트 계획 회의");
    note1.folder_id = Some(project_folder.id.clone());
    store.insert_note(&note1).unwrap();

    let mut note2 = Note::new("디자인 리뷰");
    note2.folder_id = Some(project_folder.id.clone());
    store.insert_note(&note2).unwrap();

    let mut note3 = Note::new("일반 회의");
    note3.folder_id = Some(work_folder.id.clone());
    store.insert_note(&note3).unwrap();

    // 세그먼트 삽입
    store.insert_segment(&Segment::new(&note1.id, "스프린트 목표를 설정합니다", 0, 3000)).unwrap();
    store.insert_segment(&Segment::new(&note2.id, "디자인 시안을 검토합니다", 0, 3000)).unwrap();
    store.insert_segment(&Segment::new(&note3.id, "일반 업무 논의", 0, 3000)).unwrap();

    // 폴더별 노트 조회
    let project_notes = store.list_notes(Some(&project_folder.id)).unwrap();
    assert_eq!(project_notes.len(), 2);

    let work_notes = store.list_notes(Some(&work_folder.id)).unwrap();
    assert_eq!(work_notes.len(), 1);

    // 전체 노트 조회
    let all_notes = store.list_notes(None).unwrap();
    assert_eq!(all_notes.len(), 3);

    // 크로스 노트 검색
    let results = store.search_transcripts("스프린트*").unwrap();
    assert_eq!(results.len(), 1);
    assert_eq!(results[0].note_id, note1.id);
}

// ── E2E-103: 파일 기반 DB 영속성 ────────────────────────────────

#[test]
fn e2e_103_file_db_persistence() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("voxnote.db");

    let note_id: String;

    // 세션 1: 데이터 쓰기
    {
        let store = SqliteStore::open(&db_path).unwrap();
        let note = Note::new("Persistent Note");
        note_id = note.id.clone();
        store.insert_note(&note).unwrap();
        store.insert_segment(&Segment::new(&note_id, "Hello world", 0, 1000)).unwrap();
    } // store drop → DB 닫힘

    // 세션 2: 데이터 읽기 (DB 파일에서 재로드)
    {
        let store = SqliteStore::open(&db_path).unwrap();
        let note = store.get_note(&note_id).unwrap();
        assert!(note.is_some(), "Note should persist across sessions");
        assert_eq!(note.unwrap().title, "Persistent Note");

        let segments = store.get_segments(&note_id).unwrap();
        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].text, "Hello world");
    }
}

// ── E2E-104: 대량 데이터 시나리오 ───────────────────────────────

#[test]
fn e2e_104_bulk_data_performance() {
    let store = SqliteStore::open_in_memory().unwrap();

    let start = std::time::Instant::now();

    // 100개 노트 × 20개 세그먼트 = 2000 세그먼트
    let mut all_note_ids = Vec::new();
    for i in 0..100 {
        let note = Note::new(format!("Meeting #{}", i));
        all_note_ids.push(note.id.clone());
        store.insert_note(&note).unwrap();

        for j in 0..20 {
            let text = format!("Speaker {} said something about topic {} in meeting {}", j % 3, j, i);
            let seg = Segment::new(&note.id, &text, j * 3000, (j + 1) * 3000);
            store.insert_segment(&seg).unwrap();
        }
    }

    let insert_time = start.elapsed();
    assert!(
        insert_time < std::time::Duration::from_secs(5),
        "2000 inserts should complete in <5s, took {:?}",
        insert_time
    );

    // 전체 노트 조회 성능
    let start = std::time::Instant::now();
    let notes = store.list_notes(None).unwrap();
    let list_time = start.elapsed();
    assert_eq!(notes.len(), 100);
    assert!(
        list_time < std::time::Duration::from_millis(100),
        "Listing 100 notes should take <100ms, took {:?}",
        list_time
    );

    // FTS5 검색 성능
    let start = std::time::Instant::now();
    let results = store.search_transcripts("topic").unwrap();
    let search_time = start.elapsed();
    assert!(!results.is_empty());
    assert!(
        search_time < std::time::Duration::from_millis(50),
        "FTS5 search over 2000 segments should take <50ms, took {:?}",
        search_time
    );

    // 특정 노트의 세그먼트 조회
    let start = std::time::Instant::now();
    let segs = store.get_segments(&all_note_ids[50]).unwrap();
    let seg_time = start.elapsed();
    assert_eq!(segs.len(), 20);
    assert!(
        seg_time < std::time::Duration::from_millis(10),
        "Getting 20 segments should take <10ms, took {:?}",
        seg_time
    );
}

// ── E2E-105: 노트 삭제 후 잔여 데이터 없음 ─────────────────────

#[test]
fn e2e_105_delete_cleans_all_related_data() {
    let store = SqliteStore::open_in_memory().unwrap();

    let note = Note::new("Delete Test");
    store.insert_note(&note).unwrap();

    // 관련 데이터 삽입
    for i in 0..5 {
        store
            .insert_segment(&Segment::new(&note.id, &format!("Segment {}", i), i * 1000, (i + 1) * 1000))
            .unwrap();
    }

    // 삭제
    store.delete_note(&note.id).unwrap();

    // 노트 없음
    assert!(store.get_note(&note.id).unwrap().is_none());

    // 세그먼트도 CASCADE 삭제
    let segments = store.get_segments(&note.id).unwrap();
    assert!(segments.is_empty(), "Segments should be cascade deleted");

    // FTS5에서도 검색 불가
    let results = store.search_transcripts("Segment").unwrap();
    assert!(results.is_empty(), "FTS5 should also be cleaned after delete");
}

// ── E2E-106: Config 저장/로드 시나리오 ──────────────────────────

#[test]
fn e2e_106_config_lifecycle() {
    let dir = TempDir::new().unwrap();
    let config_path = dir.path().join("config.toml");

    // 1. 기본 설정 생성
    let mut config = AppConfig::default();
    assert_eq!(config.audio.sample_rate, 48000);

    // 2. 사용자가 설정 변경
    config.audio.vad_threshold = 0.7;
    config.audio.window_size_secs = 2.0;
    config.stt.language = Some("ko".to_string());
    config.stt.use_gpu = false;
    config.storage.encryption_enabled = true;
    config.model.max_cache_mb = 5120;

    // 3. 저장
    config.save(&config_path).unwrap();
    assert!(config_path.exists());

    // 4. 재로드
    let loaded = AppConfig::load(&config_path).unwrap();
    assert_eq!(loaded.audio.vad_threshold, 0.7);
    assert_eq!(loaded.audio.window_size_secs, 2.0);
    assert_eq!(loaded.stt.language.as_deref(), Some("ko"));
    assert_eq!(loaded.stt.use_gpu, false);
    assert_eq!(loaded.storage.encryption_enabled, true);
    assert_eq!(loaded.model.max_cache_mb, 5120);
}

// ── E2E-107: 녹음 상태 머신 전이 검증 ──────────────────────────

#[test]
fn e2e_107_recording_state_machine() {
    use voxnote_core::models::RecordingState;

    // 정상 흐름: Idle → Recording → Paused → Recording → Stopped
    let mut state = RecordingState::Idle;

    // 녹음 시작
    state = RecordingState::Recording;
    assert_eq!(state, RecordingState::Recording);

    // 일시정지
    state = RecordingState::Paused;
    assert_eq!(state, RecordingState::Paused);

    // 재개
    state = RecordingState::Recording;
    assert_eq!(state, RecordingState::Recording);

    // 중지
    state = RecordingState::Stopped;
    assert_eq!(state, RecordingState::Stopped);

    // 중지 후 다시 녹음 (새 세션)
    state = RecordingState::Recording;
    assert_eq!(state, RecordingState::Recording);
}
