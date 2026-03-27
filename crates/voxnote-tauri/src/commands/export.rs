use tauri::State;

use crate::state::AppState;

#[tauri::command]
pub async fn export_note(
    state: State<'_, AppState>,
    note_id: String,
    format: String,
) -> Result<Vec<u8>, String> {
    let note = state
        .store
        .get_note(&note_id)
        .map_err(|e| e.to_string())?
        .ok_or_else(|| "Note not found".to_string())?;

    let segments = state
        .store
        .get_segments(&note_id)
        .map_err(|e| e.to_string())?;

    let export_data = voxnote_core::export::ExportData {
        note,
        segments,
        summary: None, // TODO: DB에서 요약 조회
    };

    match format.as_str() {
        "markdown" | "md" => {
            let md = voxnote_core::export::markdown::export_markdown(&export_data);
            Ok(md.into_bytes())
        }
        "pdf" => voxnote_core::export::pdf::export_pdf(&export_data)
            .map_err(|e| e.to_string()),
        "docx" => voxnote_core::export::docx::export_docx(&export_data)
            .map_err(|e| e.to_string()),
        _ => Err(format!("Unsupported format: {}", format)),
    }
}
