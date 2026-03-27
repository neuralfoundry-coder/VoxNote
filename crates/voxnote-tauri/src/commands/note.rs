use tauri::State;
use voxnote_core::models::{Folder, Note, Segment};
use voxnote_core::storage::sqlite::SearchResult;

use crate::state::AppState;

#[tauri::command]
pub async fn list_notes(
    state: State<'_, AppState>,
    folder_id: Option<String>,
) -> Result<Vec<Note>, String> {
    state
        .store
        .list_notes(folder_id.as_deref())
        .map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn get_note(state: State<'_, AppState>, id: String) -> Result<Option<Note>, String> {
    state.store.get_note(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn create_note(state: State<'_, AppState>, title: String) -> Result<Note, String> {
    let note = Note::new(title);
    state
        .store
        .insert_note(&note)
        .map_err(|e| e.to_string())?;
    Ok(note)
}

#[tauri::command]
pub async fn update_note(state: State<'_, AppState>, note: Note) -> Result<(), String> {
    state.store.update_note(&note).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn delete_note(state: State<'_, AppState>, id: String) -> Result<(), String> {
    state.store.delete_note(&id).map_err(|e| e.to_string())
}

#[tauri::command]
pub async fn search_notes(
    state: State<'_, AppState>,
    query: String,
) -> Result<Vec<SearchResult>, String> {
    state
        .store
        .search_transcripts(&query)
        .map_err(|e| e.to_string())
}
