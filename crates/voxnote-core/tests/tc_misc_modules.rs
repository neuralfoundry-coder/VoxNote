//! 기타 모듈 통합 테스트 — RAG, PostProcessor, Config, Export, Sync

use voxnote_core::config::AppConfig;
use voxnote_core::llm::prompt::PromptBuilder;
use voxnote_core::llm::templates::SummaryTemplate;
use voxnote_core::post_processor::aho_corasick::ProperNounMatcher;
use voxnote_core::post_processor::speaker_tagger;
use voxnote_core::rag::chunker::TextChunker;
use voxnote_core::diarize::SpeakerSegment;
use voxnote_core::export::markdown::export_markdown;
use voxnote_core::export::ExportData;
use voxnote_core::models::{Note, NoteStatus, Segment};
use voxnote_core::sync::key_exchange;

// ── Config ──────────────────────────────────────────────────────

#[test]
fn tc_config_default() {
    let config = AppConfig::default();
    assert_eq!(config.audio.sample_rate, 48000);
    assert_eq!(config.audio.vad_threshold, 0.5);
    assert_eq!(config.audio.window_size_secs, 3.0);
    assert!(config.storage.encryption_enabled);
    assert!(config.stt.use_gpu);
}

#[test]
fn tc_config_save_load_roundtrip() {
    let dir = tempfile::TempDir::new().unwrap();
    let path = dir.path().join("config.toml");

    let mut config = AppConfig::default();
    config.audio.vad_threshold = 0.7;
    config.stt.language = Some("ko".to_string());
    config.save(&path).unwrap();

    let loaded = AppConfig::load(&path).unwrap();
    assert_eq!(loaded.audio.vad_threshold, 0.7);
    assert_eq!(loaded.stt.language.as_deref(), Some("ko"));
}

#[test]
fn tc_config_load_nonexistent_returns_default() {
    let config = AppConfig::load(std::path::Path::new("/nonexistent/config.toml")).unwrap();
    assert_eq!(config.audio.sample_rate, 48000);
}

// ── PromptBuilder ───────────────────────────────────────────────

#[test]
fn tc_prompt_builder_full() {
    let prompt = PromptBuilder::new()
        .with_previous_summary("이전 회의에서 일정을 확정했습니다.")
        .with_template(voxnote_core::llm::prompt::templates::MEETING_NOTES)
        .build("발화자A: 오늘 안건은 예산입니다.\n발화자B: 동의합니다.");

    assert!(prompt.contains("meeting notes assistant"));
    assert!(prompt.contains("Previous Summary"));
    assert!(prompt.contains("이전 회의"));
    assert!(prompt.contains("예산"));
    assert!(prompt.contains("## Attendees"));
}

#[test]
fn tc_prompt_builder_minimal() {
    let prompt = PromptBuilder::new().build("Hello world");
    assert!(prompt.contains("Hello world"));
    assert!(prompt.contains("meeting notes assistant"));
    assert!(!prompt.contains("Previous Summary"));
}

// ── Templates ───────────────────────────────────────────────────

#[test]
fn tc_templates_builtins() {
    let templates = SummaryTemplate::builtins();
    assert_eq!(templates.len(), 4);
    assert!(templates.iter().all(|t| t.is_builtin));

    let ids: Vec<&str> = templates.iter().map(|t| t.id.as_str()).collect();
    assert!(ids.contains(&"meeting-notes"));
    assert!(ids.contains(&"brainstorming"));
    assert!(ids.contains(&"lecture-notes"));
    assert!(ids.contains(&"one-on-one"));
}

// ── ProperNounMatcher ───────────────────────────────────────────

#[test]
fn tc_proper_noun_multi_terms() {
    let mut matcher = ProperNounMatcher::new();
    matcher.add_term("복스노트", "VoxNote");
    matcher.add_term("리액트", "React");
    matcher.add_term("타입스크립트", "TypeScript");

    let result = matcher.process("복스노트에서 리액트와 타입스크립트를 사용합니다");
    assert!(result.contains("VoxNote"));
    assert!(result.contains("React"));
}

#[test]
fn tc_proper_noun_empty_text() {
    let matcher = ProperNounMatcher::new();
    assert_eq!(matcher.process(""), "");
}

// ── SpeakerTagger ───────────────────────────────────────────────

#[test]
fn tc_speaker_tagger_multiple() {
    let mut segments = vec![
        Segment::new("n1", "Hello", 0, 1500),
        Segment::new("n1", "How are you", 1500, 3000),
        Segment::new("n1", "Im fine", 3000, 4500),
    ];

    let speakers = vec![
        SpeakerSegment { speaker_id: "Alice".into(), start_ms: 0, end_ms: 2000, confidence: 0.9 },
        SpeakerSegment { speaker_id: "Bob".into(), start_ms: 2000, end_ms: 5000, confidence: 0.85 },
    ];

    speaker_tagger::tag_speakers(&mut segments, &speakers);
    assert_eq!(segments[0].speaker_id, Some("Alice".to_string()));
    assert_eq!(segments[2].speaker_id, Some("Bob".to_string()));
}

// ── RAG Chunker ─────────────────────────────────────────────────

#[test]
fn tc_rag_chunker_basic() {
    let text = (0..200).map(|i| format!("word{}", i)).collect::<Vec<_>>().join(" ");
    let chunker = TextChunker::new(50, 0.25);
    let chunks = chunker.chunk(&text);

    assert!(chunks.len() >= 3, "200 words / 50 chunk = at least 3 chunks");
    assert!(chunks[0].text.contains("word0"));

    for chunk in &chunks {
        let word_count = chunk.text.split_whitespace().count();
        assert!(word_count <= 50, "Chunk should have <= 50 words, got {}", word_count);
    }
}

#[test]
fn tc_rag_chunker_empty() {
    let chunker = TextChunker::default_rag();
    let chunks = chunker.chunk("");
    assert!(chunks.is_empty());
}

#[test]
fn tc_rag_chunker_small_text() {
    let chunker = TextChunker::new(100, 0.25);
    let chunks = chunker.chunk("short text");
    assert_eq!(chunks.len(), 1);
    assert_eq!(chunks[0].text, "short text");
}

// ── Diarizer ────────────────────────────────────────────────────

#[test]
fn tc_diarize_cosine_similarity() {
    let a = vec![1.0, 0.0, 0.0, 0.0];
    let b = vec![1.0, 0.0, 0.0, 0.0];
    let c = vec![0.0, 1.0, 0.0, 0.0];

    assert!((voxnote_core::diarize::onnx::cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);
    assert!(voxnote_core::diarize::onnx::cosine_similarity(&a, &c).abs() < 1e-6);
}

// ── Export Markdown ─────────────────────────────────────────────

#[test]
fn tc_export_markdown_full() {
    let note = Note {
        id: "test-id".to_string(),
        title: "Weekly Standup".to_string(),
        status: NoteStatus::Done,
        folder_id: None,
        duration_ms: Some(1800000),
        language: Some("en".to_string()),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let segments = vec![
        Segment { id: "s1".into(), note_id: "test-id".into(), text: "Good morning".into(),
                  start_ms: 0, end_ms: 2000, speaker_id: Some("Alice".into()), confidence: Some(0.95) },
        Segment { id: "s2".into(), note_id: "test-id".into(), text: "Let's start".into(),
                  start_ms: 2000, end_ms: 4000, speaker_id: Some("Bob".into()), confidence: Some(0.9) },
    ];

    let data = ExportData { note, segments, summary: Some("Meeting summary here".to_string()) };
    let md = export_markdown(&data);

    assert!(md.contains("# Weekly Standup"));
    assert!(md.contains("**Duration:**"));
    assert!(md.contains("30m"));
    assert!(md.contains("## Summary"));
    assert!(md.contains("Meeting summary here"));
    assert!(md.contains("## Transcript"));
    assert!(md.contains("**Alice**"));
    assert!(md.contains("**Bob**"));
    assert!(md.contains("Good morning"));
}

#[test]
fn tc_export_markdown_no_summary() {
    let note = Note::new("No Summary");
    let data = ExportData { note, segments: vec![], summary: None };
    let md = export_markdown(&data);

    assert!(md.contains("# No Summary"));
    assert!(!md.contains("## Summary"));
}

// ── Sync Key Exchange ───────────────────────────────────────────

#[test]
fn tc_sync_pairing_code_format() {
    let code = key_exchange::generate_pairing_code("device-1", &[1, 2, 3, 4]);
    assert_eq!(code.len(), 6);
    assert!(code.chars().all(|c| c.is_ascii_digit()));
}

#[test]
fn tc_sync_pairing_code_deterministic() {
    let code1 = key_exchange::generate_pairing_code("dev-1", &[10, 20]);
    let code2 = key_exchange::generate_pairing_code("dev-1", &[10, 20]);
    assert_eq!(code1, code2, "Same input should produce same code");
}

#[test]
fn tc_sync_pairing_code_different_devices() {
    let code1 = key_exchange::generate_pairing_code("device-A", &[1, 2, 3]);
    let code2 = key_exchange::generate_pairing_code("device-B", &[1, 2, 3]);
    assert_ne!(code1, code2, "Different devices should get different codes");
}
