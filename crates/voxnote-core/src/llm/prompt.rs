/// 프롬프트 빌더 — 시스템 프롬프트 + 이전 요약 + 현재 전사 + 템플릿
pub struct PromptBuilder {
    system_prompt: String,
    previous_summary: Option<String>,
    template_directives: Option<String>,
}

impl PromptBuilder {
    pub fn new() -> Self {
        Self {
            system_prompt: DEFAULT_SYSTEM_PROMPT.to_string(),
            previous_summary: None,
            template_directives: None,
        }
    }

    pub fn with_system_prompt(mut self, prompt: &str) -> Self {
        self.system_prompt = prompt.to_string();
        self
    }

    pub fn with_previous_summary(mut self, summary: &str) -> Self {
        self.previous_summary = Some(summary.to_string());
        self
    }

    pub fn with_template(mut self, template: &str) -> Self {
        self.template_directives = Some(template.to_string());
        self
    }

    /// 최종 프롬프트 조합
    pub fn build(&self, transcript: &str) -> String {
        let mut parts = vec![self.system_prompt.clone()];

        if let Some(ref template) = self.template_directives {
            parts.push(format!("\n## Output Format\n{}", template));
        }

        if let Some(ref prev) = self.previous_summary {
            parts.push(format!("\n## Previous Summary\n{}", prev));
        }

        parts.push(format!("\n## Current Transcript\n{}", transcript));
        parts.push("\n## Summary\nPlease generate a structured summary:".to_string());

        parts.join("\n")
    }
}

const DEFAULT_SYSTEM_PROMPT: &str = r#"You are a meeting notes assistant. Your task is to create structured, concise summaries from meeting transcripts. Focus on:
- Key decisions made
- Action items with assignees
- Important discussion points
- Next steps

Be factual and concise. Use bullet points."#;

/// 내장 템플릿
pub mod templates {
    pub const MEETING_NOTES: &str = r#"Format as meeting notes with these sections:
## Attendees
## Agenda
## Discussion
## Decisions
## Action Items
## Next Steps"#;

    pub const BRAINSTORMING: &str = r#"Format as brainstorming session notes:
## Topic
## Ideas Generated
## Top Ideas (ranked)
## Next Actions"#;

    pub const LECTURE_NOTES: &str = r#"Format as lecture notes:
## Topic
## Key Concepts
## Important Details
## Questions & Answers
## Summary"#;

    pub const ONE_ON_ONE: &str = r#"Format as 1:1 meeting notes:
## Updates
## Blockers
## Goals for Next Period
## Action Items"#;
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prompt_builder() {
        let prompt = PromptBuilder::new()
            .with_previous_summary("Previous meeting discussed X.")
            .with_template(templates::MEETING_NOTES)
            .build("Speaker A: Let's finalize the design.\nSpeaker B: Agreed.");

        assert!(prompt.contains("meeting notes assistant"));
        assert!(prompt.contains("Previous Summary"));
        assert!(prompt.contains("finalize the design"));
        assert!(prompt.contains("## Attendees"));
    }
}
