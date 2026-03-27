//! 실제 앱 실행 기반 전체 E2E 테스트
//!
//! 실제 SQLite 파일 + Whisper 모델을 사용하여
//! 녹음→전사→저장→검색→내보내기 전체 파이프라인을 검증합니다.

use tempfile::TempDir;
use voxnote_core::config::AppConfig;
use voxnote_core::export::{markdown, ExportData};
use voxnote_core::models::{Note, NoteStatus, Segment};
use voxnote_core::storage::SqliteStore;

// ── E2E-REAL-001: 파일 기반 DB 전체 사용자 시나리오 ─────────────

#[test]
fn real_e2e_001_complete_user_scenario() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("voxnote.db");

    // 1. 앱 시작 → DB 초기화
    let store = SqliteStore::open(&db_path).unwrap();
    assert!(db_path.exists(), "DB file should be created");

    // 2. 폴더 생성
    let work_folder = voxnote_core::models::Folder::new("업무 회의", None);
    store.insert_folder(&work_folder).unwrap();

    // 3. 녹음 시작 → 노트 생성
    let mut note = Note::new("2024 Q1 스프린트 계획");
    note.folder_id = Some(work_folder.id.clone());
    store.insert_note(&note).unwrap();

    // 4. 실시간 전사 시뮬레이션 (5개 세그먼트)
    let segments_data = vec![
        ("김팀장: 안녕하세요, 오늘 스프린트 계획을 논의하겠습니다.", 0, 4000, Some("김팀장")),
        ("이개발: VoxNote 백엔드 구현이 80% 완료되었습니다.", 4000, 8000, Some("이개발")),
        ("박디자: UI 디자인은 다음 주 목요일까지 전달 예정입니다.", 8000, 12000, Some("박디자")),
        ("김팀장: 테스트 커버리지 목표는 80% 이상입니다.", 12000, 16000, Some("김팀장")),
        ("이개발: Whisper 모델 통합 테스트를 진행 중입니다.", 16000, 20000, Some("이개발")),
    ];

    for (text, start, end, speaker) in &segments_data {
        let mut seg = Segment::new(&note.id, *text, *start, *end);
        seg.speaker_id = speaker.map(|s| s.to_string());
        seg.confidence = Some(0.92);
        store.insert_segment(&seg).unwrap();
    }

    // 5. 녹음 완료 → 상태 업데이트
    note.status = NoteStatus::Done;
    note.duration_ms = Some(20000);
    note.language = Some("ko".to_string());
    note.updated_at = chrono::Utc::now();
    store.update_note(&note).unwrap();

    // ──── 검증 ────────────────────────────────────────────────

    // A. 노트 조회
    let fetched = store.get_note(&note.id).unwrap().unwrap();
    assert_eq!(fetched.status, NoteStatus::Done);
    assert_eq!(fetched.duration_ms, Some(20000));

    // B. 세그먼트 조회 (시간순 정렬)
    let segments = store.get_segments(&note.id).unwrap();
    assert_eq!(segments.len(), 5);
    assert!(segments[0].text.contains("안녕하세요"));
    assert!(segments[4].text.contains("Whisper"));
    for i in 1..segments.len() {
        assert!(segments[i].start_ms >= segments[i - 1].end_ms);
    }

    // C. 화자 확인
    assert_eq!(segments[0].speaker_id.as_deref(), Some("김팀장"));
    assert_eq!(segments[1].speaker_id.as_deref(), Some("이개발"));
    assert_eq!(segments[2].speaker_id.as_deref(), Some("박디자"));

    // D. 전문 검색
    let results = store.search_transcripts("VoxNote").unwrap();
    assert!(!results.is_empty());
    assert_eq!(results[0].note_id, note.id);

    let results = store.search_transcripts("스프린트*").unwrap();
    assert!(!results.is_empty());

    // E. 폴더 내 노트 조회
    let folder_notes = store.list_notes(Some(&work_folder.id)).unwrap();
    assert_eq!(folder_notes.len(), 1);

    // F. 내보내기
    let export_data = ExportData {
        note: fetched,
        segments,
        summary: Some("## 스프린트 계획\n- 백엔드 80% 완료\n- UI 디자인 다음 주 목요일\n- 테스트 커버리지 80% 목표".to_string()),
    };
    let md = markdown::export_markdown(&export_data);
    assert!(md.contains("# 2024 Q1 스프린트 계획"));
    assert!(md.contains("**김팀장**"));
    assert!(md.contains("## Summary"));
    assert!(md.contains("백엔드 80%"));
    assert!(md.contains("## Transcript"));

    // G. DB 파일 크기 확인
    let db_size = std::fs::metadata(&db_path).unwrap().len();
    assert!(
        db_size < 1_048_576,
        "DB should be <1MB for 5 segments, got {} bytes",
        db_size
    );

    eprintln!("E2E full scenario passed — DB size: {} bytes", db_size);
}

// ── E2E-REAL-002: 실제 Whisper 모델 → 전사 → 저장 ──────────────

#[cfg(feature = "stt")]
#[tokio::test]
async fn real_e2e_002_whisper_to_storage() {
    let model_path = dirs::home_dir().unwrap().join(".voxnote/models/ggml-tiny.bin");
    if !model_path.exists() {
        eprintln!("SKIP: whisper model not found");
        return;
    }

    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("test.db");
    let store = SqliteStore::open(&db_path).unwrap();

    // 노트 생성
    let note = Note::new("Whisper E2E Test");
    store.insert_note(&note).unwrap();

    // STT 추론
    use voxnote_core::audio::AudioChunk;
    use voxnote_core::stt::whisper::LocalSttProvider;
    use voxnote_core::stt::SttProvider;

    let provider = LocalSttProvider::new(model_path).unwrap();

    // 3초 합성 오디오
    let audio: Vec<f32> = (0..48000)
        .map(|i| (i as f32 / 16000.0 * 440.0 * 2.0 * std::f32::consts::PI).sin() * 0.3)
        .collect();
    let chunk = AudioChunk::new(audio, 0);

    let segments = provider.transcribe(&chunk, &note.id).await.unwrap();

    // 세그먼트 DB 저장
    for seg in &segments {
        store.insert_segment(seg).unwrap();
    }

    // 검증
    let stored = store.get_segments(&note.id).unwrap();
    assert_eq!(stored.len(), segments.len());

    eprintln!(
        "Whisper→Storage E2E: {} segments stored, texts: {:?}",
        stored.len(),
        stored.iter().map(|s| &s.text).collect::<Vec<_>>()
    );
}

// ── E2E-REAL-003: 대규모 녹음 시뮬레이션 (1시간) ────────────────

#[test]
fn real_e2e_003_one_hour_recording_simulation() {
    let dir = TempDir::new().unwrap();
    let db_path = dir.path().join("long.db");
    let store = SqliteStore::open(&db_path).unwrap();

    let note = Note::new("1 Hour Meeting");
    store.insert_note(&note).unwrap();

    let start = std::time::Instant::now();

    // 1시간 = 3600초 → 3초 윈도우 = 1200 세그먼트
    for i in 0..1200 {
        let text = format!(
            "Segment {} at {}:{:02}: Discussion about topic {}",
            i,
            i * 3 / 60,
            (i * 3) % 60,
            i % 10
        );
        let seg = Segment::new(&note.id, &text, i as i64 * 3000, (i as i64 + 1) * 3000);
        store.insert_segment(&seg).unwrap();
    }

    let insert_time = start.elapsed();

    // 검증
    let segments = store.get_segments(&note.id).unwrap();
    assert_eq!(segments.len(), 1200);

    // 검색 성능
    let start = std::time::Instant::now();
    let results = store.search_transcripts("topic").unwrap();
    let search_time = start.elapsed();
    assert!(!results.is_empty());

    let db_size = std::fs::metadata(&db_path).unwrap().len();

    eprintln!(
        "1hr simulation: 1200 segments, insert={:?}, search={:?}, DB={}KB",
        insert_time,
        search_time,
        db_size / 1024
    );

    assert!(insert_time < std::time::Duration::from_secs(5));
    assert!(search_time < std::time::Duration::from_millis(100));
}

// ── E2E-REAL-004: Config + 모델 디렉토리 실제 경로 ──────────────

#[test]
fn real_e2e_004_actual_paths() {
    let config = AppConfig::default();

    let data_dir = config.data_dir();
    let models_dir = config.models_dir();

    assert!(
        data_dir.to_string_lossy().contains(".voxnote"),
        "Data dir should be under ~/.voxnote"
    );
    assert!(
        models_dir.to_string_lossy().contains(".voxnote"),
        "Models dir should be under ~/.voxnote"
    );

    // 실제 모델 디렉토리 확인
    if models_dir.exists() {
        let files: Vec<_> = std::fs::read_dir(&models_dir)
            .unwrap()
            .filter_map(|e| e.ok())
            .map(|e| e.file_name().to_string_lossy().to_string())
            .collect();
        eprintln!("Models dir contains: {:?}", files);
    }
}

// ── E2E-REAL-005: 앱 바이너리 존재 및 크기 검증 ─────────────────

#[test]
fn real_e2e_005_binary_size() {
    let binary = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../target/release/voxnote-tauri");

    if !binary.exists() {
        eprintln!("SKIP: Release binary not built");
        return;
    }

    let size = std::fs::metadata(&binary).unwrap().len();
    let size_mb = size / 1_048_576;

    eprintln!("Binary size: {}MB", size_mb);

    // SRS NFR-PERF-004: 바이너리 크기 < 30MB (모델 미포함)
    assert!(
        size_mb < 30,
        "Binary should be <30MB (NFR-PERF-004), got {}MB",
        size_mb
    );
}
