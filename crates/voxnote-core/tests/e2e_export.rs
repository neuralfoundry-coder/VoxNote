//! E2E: 내보내기 시나리오 테스트
//!
//! 노트 → Markdown/PDF/DOCX 내보내기 전체 플로우 검증

use voxnote_core::export::{markdown, pdf, docx, ExportData};
use voxnote_core::models::{Note, NoteStatus, Segment};
use chrono::Utc;

fn create_test_export_data() -> ExportData {
    let note = Note {
        id: "export-test".to_string(),
        title: "프로젝트 킥오프 미팅".to_string(),
        status: NoteStatus::Done,
        folder_id: None,
        duration_ms: Some(2700000), // 45분
        language: Some("ko".to_string()),
        created_at: Utc::now(),
        updated_at: Utc::now(),
    };

    let segments = vec![
        Segment {
            id: "s1".into(), note_id: "export-test".into(),
            text: "안녕하세요, 프로젝트 킥오프 미팅을 시작하겠습니다.".into(),
            start_ms: 0, end_ms: 4000,
            speaker_id: Some("김팀장".into()), confidence: Some(0.95),
        },
        Segment {
            id: "s2".into(), note_id: "export-test".into(),
            text: "이번 프로젝트의 목표는 VoxNote MVP 출시입니다.".into(),
            start_ms: 4000, end_ms: 8000,
            speaker_id: Some("김팀장".into()), confidence: Some(0.92),
        },
        Segment {
            id: "s3".into(), note_id: "export-test".into(),
            text: "백엔드는 Rust로 구현하고 프론트엔드는 React를 사용합니다.".into(),
            start_ms: 8000, end_ms: 12000,
            speaker_id: Some("이개발".into()), confidence: Some(0.88),
        },
        Segment {
            id: "s4".into(), note_id: "export-test".into(),
            text: "일정은 3개월로 잡겠습니다.".into(),
            start_ms: 12000, end_ms: 15000,
            speaker_id: Some("김팀장".into()), confidence: Some(0.91),
        },
    ];

    let summary = Some(
        "## 프로젝트 킥오프\n\
         - VoxNote MVP 3개월 출시 목표\n\
         - 백엔드: Rust, 프론트엔드: React\n\
         - 다음 회의: 스프린트 계획".to_string()
    );

    ExportData { note, segments, summary }
}

// ── E2E-301: Markdown 내보내기 ──────────────────────────────────

#[test]
fn e2e_301_markdown_export_structure() {
    let data = create_test_export_data();
    let md = markdown::export_markdown(&data);

    // 제목
    assert!(md.starts_with("# 프로젝트 킥오프 미팅\n"));

    // 메타데이터
    assert!(md.contains("**Language:** ko"));
    assert!(md.contains("**Duration:**"));
    assert!(md.contains("45m"));

    // 요약
    assert!(md.contains("## Summary"));
    assert!(md.contains("VoxNote MVP"));

    // 전사
    assert!(md.contains("## Transcript"));

    // 화자 구분
    assert!(md.contains("**김팀장**"));
    assert!(md.contains("**이개발**"));

    // 전사 텍스트
    assert!(md.contains("프로젝트 킥오프 미팅을 시작하겠습니다"));
    assert!(md.contains("Rust로 구현하고"));
}

#[test]
fn e2e_301_markdown_export_no_speaker() {
    let data = ExportData {
        note: Note::new("No Speaker Test"),
        segments: vec![
            Segment::new("n1", "First segment", 0, 3000),
            Segment::new("n1", "Second segment", 3000, 6000),
        ],
        summary: None,
    };
    let md = markdown::export_markdown(&data);

    assert!(!md.contains("## Summary"));
    assert!(md.contains("First segment"));
    assert!(md.contains("Second segment"));
}

// ── E2E-302: PDF 내보내기 ───────────────────────────────────────

#[test]
fn e2e_302_pdf_export_produces_bytes() {
    let data = create_test_export_data();
    let bytes = pdf::export_pdf(&data).unwrap();

    assert!(!bytes.is_empty(), "PDF export should produce non-empty bytes");
    // 현재는 Markdown 텍스트를 바이트로 반환 (프로덕션에서 genpdf 연동)
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("프로젝트 킥오프 미팅"));
}

// ── E2E-303: DOCX 내보내기 ──────────────────────────────────────

#[test]
fn e2e_303_docx_export_produces_bytes() {
    let data = create_test_export_data();
    let bytes = docx::export_docx(&data).unwrap();

    assert!(!bytes.is_empty());
    let text = String::from_utf8_lossy(&bytes);
    assert!(text.contains("프로젝트 킥오프 미팅"));
}

// ── E2E-304: 빈 노트 내보내기 ──────────────────────────────────

#[test]
fn e2e_304_export_empty_note() {
    let data = ExportData {
        note: Note::new("Empty"),
        segments: vec![],
        summary: None,
    };
    let md = markdown::export_markdown(&data);
    assert!(md.contains("# Empty"));
    assert!(md.contains("## Transcript"));
}
