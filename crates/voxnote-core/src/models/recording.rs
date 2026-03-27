use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RecordingState {
    Idle,
    Recording,
    Paused,
    Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Recording {
    pub session_id: String,
    pub note_id: String,
    pub state: RecordingState,
    pub elapsed_ms: u64,
    pub sample_rate: u32,
}
