use tauri::State;

use voxnote_core::llm::prompt::PromptBuilder;
use voxnote_core::llm::templates::SummaryTemplate;
use voxnote_core::llm::GenerateConfig;
use voxnote_core::storage::SummaryRow;

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
    // 1. 전사 세그먼트 조회
    let segments = state
        .store
        .get_segments(&request.note_id)
        .map_err(|e| e.to_string())?;

    if segments.is_empty() {
        return Err("No transcript segments found for this note".to_string());
    }

    // 2. 전사 텍스트 조합 (타임스탬프 포함)
    let transcript = segments
        .iter()
        .map(|s| {
            let mins = s.start_ms / 60000;
            let secs = (s.start_ms % 60000) / 1000;
            format!("[{:02}:{:02}] {}", mins, secs, s.text)
        })
        .collect::<Vec<_>>()
        .join("\n");

    // 3. 템플릿 조회
    let template_id = request
        .template_id
        .unwrap_or_else(|| "meeting-notes".to_string());

    let builtins = SummaryTemplate::builtins();
    let template = builtins
        .iter()
        .find(|t| t.id == template_id)
        .map(|t| t.prompt.as_str())
        .unwrap_or("");

    // 4. 프롬프트 빌드
    let previous_summary = state
        .store
        .get_latest_summary(&request.note_id)
        .ok()
        .flatten()
        .map(|s| s.content);

    let mut builder = PromptBuilder::new().with_template(template);
    if let Some(ref prev) = previous_summary {
        builder = builder.with_previous_summary(prev);
    }
    let prompt = builder.build(&transcript);

    // 5. LLM Provider로 생성
    let provider = state
        .provider_registry
        .lock()
        .map_err(|e| e.to_string())?
        .active_llm()
        .ok_or_else(|| "No active LLM provider configured. Please set up a provider in Settings.".to_string())?;

    let config = GenerateConfig::default();
    let model_name = provider.name().to_string();

    let summary_text = provider
        .generate(&prompt, &config)
        .await
        .map_err(|e| format!("LLM generation failed: {}", e))?;

    // 6. DB에 저장
    let mut row = SummaryRow::new(&request.note_id, &summary_text);
    row.template_id = Some(template_id.clone());
    row.model_used = Some(model_name.clone());
    row.provider = Some(model_name.clone());

    state
        .store
        .insert_summary(&row)
        .map_err(|e| e.to_string())?;

    // 7. Note 상태 업데이트
    if let Ok(Some(mut note)) = state.store.get_note(&request.note_id) {
        note.status = voxnote_core::models::NoteStatus::Done;
        note.updated_at = chrono::Utc::now();
        let _ = state.store.update_note(&note);
    }

    Ok(SummaryResponse {
        summary: summary_text,
        template_id,
        model_used: model_name,
    })
}

#[derive(serde::Deserialize)]
pub struct AskRequest {
    pub question: String,
    pub note_id: Option<String>,
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
    // 1. 관련 컨텍스트 수집
    let context = if let Some(ref note_id) = request.note_id {
        // 특정 노트의 전사 텍스트 사용
        let segments = state
            .store
            .get_segments(note_id)
            .map_err(|e| e.to_string())?;

        segments
            .iter()
            .map(|s| s.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    } else {
        // FTS5로 관련 세그먼트 검색
        let results = state
            .store
            .search_transcripts(&request.question)
            .map_err(|e| e.to_string())?;

        results
            .iter()
            .map(|r| r.text.as_str())
            .collect::<Vec<_>>()
            .join("\n")
    };

    if context.is_empty() {
        return Ok(AskResponse {
            answer: "No relevant transcript content found.".to_string(),
            sources: Vec::new(),
        });
    }

    // 2. QA 프롬프트 빌드
    let prompt = format!(
        "Based on the following meeting transcript, answer the user's question concisely and accurately.\n\n\
         ## Transcript\n{}\n\n\
         ## Question\n{}\n\n\
         ## Answer",
        context, request.question
    );

    // 3. LLM Provider로 응답 생성
    let provider = state
        .provider_registry
        .lock()
        .map_err(|e| e.to_string())?
        .active_llm()
        .ok_or_else(|| "No active LLM provider configured.".to_string())?;

    let config = GenerateConfig {
        temperature: 0.2,
        max_tokens: 1024,
        ..Default::default()
    };

    let answer = provider
        .generate(&prompt, &config)
        .await
        .map_err(|e| format!("LLM generation failed: {}", e))?;

    // 4. 소스 정보 수집
    let sources = if let Some(ref note_id) = request.note_id {
        vec![format!("Note: {}", note_id)]
    } else {
        let results = state
            .store
            .search_transcripts(&request.question)
            .unwrap_or_default();
        results
            .iter()
            .take(3)
            .map(|r| format!("Note: {} — \"{}\"", r.note_id, truncate(&r.text, 50)))
            .collect()
    };

    Ok(AskResponse { answer, sources })
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}…", &s[..max_len])
    }
}
