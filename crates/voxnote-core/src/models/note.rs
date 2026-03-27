use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum NoteStatus {
    Recording,
    Transcribing,
    Summarizing,
    Done,
    Error,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Note {
    pub id: String,
    pub title: String,
    pub status: NoteStatus,
    pub folder_id: Option<String>,
    pub duration_ms: Option<i64>,
    pub language: Option<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

impl Note {
    pub fn new(title: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4().to_string(),
            title: title.into(),
            status: NoteStatus::Recording,
            folder_id: None,
            duration_ms: None,
            language: None,
            created_at: now,
            updated_at: now,
        }
    }
}
