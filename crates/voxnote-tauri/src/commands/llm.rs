use tauri::State;

use crate::state::AppState;

#[derive(serde::Deserialize)]
pub struct SummaryRequest {
    pub note_id: String,
    pub template_id: Option<String>,
}

#[derive(serde::Serialize)]
pub struct SummaryResponse {
    pub summary: String,
    pub template_id: String,
    pub model_used: String,
}

#[tauri::command]
pub async fn generate_summary(
    state: State<'_, AppState>,
    request: SummaryRequest,
) -> Result<SummaryResponse, String> {
    // TODO: 실제 LLM Provider로 요약 생성
    Ok(SummaryResponse {
        summary: format!("Summary for note {} (template: {:?})", request.note_id, request.template_id),
        template_id: request.template_id.unwrap_or_else(|| "meeting-notes".to_string()),
        model_used: "pending".to_string(),
    })
}

#[derive(serde::Deserialize)]
pub struct AskRequest {
    pub question: String,
}

#[derive(serde::Serialize)]
pub struct AskResponse {
    pub answer: String,
    pub sources: Vec<String>,
}

#[tauri::command]
pub async fn ask_voxnote(
    state: State<'_, AppState>,
    request: AskRequest,
) -> Result<AskResponse, String> {
    // TODO: RAG 파이프라인 연결
    Ok(AskResponse {
        answer: format!("RAG answer for: {}", request.question),
        sources: Vec::new(),
    })
}
