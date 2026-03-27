use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// STT 전사 결과 세그먼트
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Segment {
    pub id: String,
    pub note_id: String,
    pub text: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub speaker_id: Option<String>,
    pub confidence: Option<f32>,
}

impl Segment {
    pub fn new(note_id: impl Into<String>, text: impl Into<String>, start_ms: i64, end_ms: i64) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            note_id: note_id.into(),
            text: text.into(),
            start_ms,
            end_ms,
            speaker_id: None,
            confidence: None,
        }
    }

    pub fn duration_ms(&self) -> i64 {
        self.end_ms - self.start_ms
    }
}
