pub mod docx;
pub mod markdown;
pub mod pdf;

use crate::models::{Note, Segment};

/// 내보내기 형식
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum ExportFormat {
    Markdown,
    Pdf,
    Docx,
}

/// 내보내기 대상 데이터
pub struct ExportData {
    pub note: Note,
    pub segments: Vec<Segment>,
    pub summary: Option<String>,
}
