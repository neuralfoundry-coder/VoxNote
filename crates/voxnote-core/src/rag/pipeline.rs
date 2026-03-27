use std::sync::Arc;
use tracing::debug;

use super::chunker::TextChunker;
use super::embedder::EmbeddingProvider;
use super::search::{cosine_similarity, HybridSearch, SearchHit, SearchSource};
use crate::error::VoxNoteError;
use crate::llm::{GenerateConfig, LlmProvider};

/// RAG 파이프라인 — "Ask VoxNote"
pub struct RagPipeline {
    embedder: Arc<dyn EmbeddingProvider>,
    llm: Arc<dyn LlmProvider>,
    search: HybridSearch,
}

impl RagPipeline {
    pub fn new(embedder: Arc<dyn EmbeddingProvider>, llm: Arc<dyn LlmProvider>) -> Self {
        Self {
            embedder,
            llm,
            search: HybridSearch::new(),
        }
    }

    /// 질의 → 임베딩 → 검색 → 컨텍스트 조합 → LLM 응답 생성
    pub async fn ask(
        &self,
        question: &str,
        stored_chunks: &[(String, Vec<f32>, String)], // (text, vector, note_id)
    ) -> Result<RagAnswer, VoxNoteError> {
        // 1. 질의 임베딩
        let query_embedding = self.embedder.embed(question).await?;

        // 2. 벡터 검색
        let mut vector_hits: Vec<SearchHit> = stored_chunks
            .iter()
            .enumerate()
            .map(|(i, (text, vector, note_id))| {
                let score = cosine_similarity(&query_embedding, vector);
                SearchHit {
                    chunk_text: text.clone(),
                    note_id: note_id.clone(),
                    chunk_index: i,
                    score,
                    source: SearchSource::Vector,
                }
            })
            .filter(|hit| hit.score > 0.0)
            .collect();

        vector_hits.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        vector_hits.truncate(self.search.top_k);

        // 3. 컨텍스트 조합
        let context: String = vector_hits
            .iter()
            .map(|hit| format!("[Source: {}]\n{}", hit.note_id, hit.chunk_text))
            .collect::<Vec<_>>()
            .join("\n\n---\n\n");

        if context.is_empty() {
            return Ok(RagAnswer {
                answer: "No relevant information found in your notes.".to_string(),
                sources: Vec::new(),
            });
        }

        // 4. LLM 응답 생성
        let prompt = format!(
            "Based on the following meeting notes, answer the question. \
             Cite specific sources when possible.\n\n\
             ## Context\n{}\n\n\
             ## Question\n{}\n\n\
             ## Answer",
            context, question
        );

        let config = GenerateConfig {
            temperature: 0.2,
            top_p: 0.9,
            max_tokens: 1024,
            grammar: None,
        };

        let answer = self
            .llm
            .generate(&prompt, &config)
            .await
            .map_err(|e| VoxNoteError::Llm(e))?;

        let sources: Vec<String> = vector_hits
            .iter()
            .map(|h| h.note_id.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        debug!("RAG answer generated with {} source notes", sources.len());

        Ok(RagAnswer { answer, sources })
    }
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct RagAnswer {
    pub answer: String,
    pub sources: Vec<String>,
}
