use super::ExportData;

/// Markdown 형식 내보내기
pub fn export_markdown(data: &ExportData) -> String {
    let mut md = String::new();

    md.push_str(&format!("# {}\n\n", data.note.title));
    md.push_str(&format!(
        "**Date:** {}  \n",
        data.note.created_at.format("%Y-%m-%d %H:%M")
    ));

    if let Some(lang) = &data.note.language {
        md.push_str(&format!("**Language:** {}  \n", lang));
    }

    if let Some(duration) = data.note.duration_ms {
        let mins = duration / 60000;
        let secs = (duration % 60000) / 1000;
        md.push_str(&format!("**Duration:** {}m {}s  \n", mins, secs));
    }

    md.push_str("\n---\n\n");

    // Summary
    if let Some(ref summary) = data.summary {
        md.push_str("## Summary\n\n");
        md.push_str(summary);
        md.push_str("\n\n---\n\n");
    }

    // Transcript
    md.push_str("## Transcript\n\n");
    let mut current_speaker: Option<String> = None;

    for seg in &data.segments {
        let timestamp = format_timestamp(seg.start_ms);

        if seg.speaker_id != current_speaker {
            if let Some(ref speaker) = seg.speaker_id {
                md.push_str(&format!("\n**{}** ({})\n\n", speaker, timestamp));
            } else {
                md.push_str(&format!("\n*{}*\n\n", timestamp));
            }
            current_speaker = seg.speaker_id.clone();
        }

        md.push_str(&format!("{}\n\n", seg.text));
    }

    md
}

fn format_timestamp(ms: i64) -> String {
    let total_secs = ms / 1000;
    let mins = total_secs / 60;
    let secs = total_secs % 60;
    format!("{:02}:{:02}", mins, secs)
}
