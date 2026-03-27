use serde::{Deserialize, Serialize};
use uuid::Uuid;

/// 요약 템플릿
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SummaryTemplate {
    pub id: String,
    pub name: String,
    pub description: String,
    pub prompt: String,
    pub is_builtin: bool,
}

impl SummaryTemplate {
    pub fn new_custom(name: &str, description: &str, prompt: &str) -> Self {
        Self {
            id: Uuid::new_v4().to_string(),
            name: name.to_string(),
            description: description.to_string(),
            prompt: prompt.to_string(),
            is_builtin: false,
        }
    }

    /// 내장 템플릿 목록
    pub fn builtins() -> Vec<Self> {
        vec![
            Self {
                id: "meeting-notes".to_string(),
                name: "Meeting Notes".to_string(),
                description: "Standard meeting notes with attendees, decisions, and action items".to_string(),
                prompt: super::prompt::templates::MEETING_NOTES.to_string(),
                is_builtin: true,
            },
            Self {
                id: "brainstorming".to_string(),
                name: "Brainstorming".to_string(),
                description: "Brainstorming session with ideas and rankings".to_string(),
                prompt: super::prompt::templates::BRAINSTORMING.to_string(),
                is_builtin: true,
            },
            Self {
                id: "lecture-notes".to_string(),
                name: "Lecture Notes".to_string(),
                description: "Lecture notes with key concepts and Q&A".to_string(),
                prompt: super::prompt::templates::LECTURE_NOTES.to_string(),
                is_builtin: true,
            },
            Self {
                id: "one-on-one".to_string(),
                name: "1:1 Meeting".to_string(),
                description: "One-on-one meeting with updates and blockers".to_string(),
                prompt: super::prompt::templates::ONE_ON_ONE.to_string(),
                is_builtin: true,
            },
        ]
    }
}
