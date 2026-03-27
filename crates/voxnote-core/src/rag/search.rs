/// 하이브리드 검색 — 벡터 유사도 + FTS5 키워드 검색
pub struct HybridSearch {
    /// 벡터 검색 가중치
    pub vector_weight: f32,
    /// 키워드 검색 가중치
    pub keyword_weight: f32,
    /// 상위 K개 결과
    pub top_k: usize,
    /// 유사도 임계값
    pub similarity_threshold: f32,
}

#[derive(Debug, Clone)]
pub struct SearchHit {
    pub chunk_text: String,
    pub note_id: String,
    pub chunk_index: usize,
    pub score: f32,
    pub source: SearchSource,
}

#[derive(Debug, Clone)]
pub enum SearchSource {
    Vector,
    Keyword,
    Hybrid,
}

impl HybridSearch {
    pub fn new() -> Self {
        Self {
            vector_weight: 0.7,
            keyword_weight: 0.3,
            top_k: 5,
            similarity_threshold: 0.7,
        }
    }

    /// 벡터 검색 결과와 키워드 검색 결과를 하이브리드 리랭킹
    pub fn merge_results(
        &self,
        vector_hits: Vec<SearchHit>,
        keyword_hits: Vec<SearchHit>,
    ) -> Vec<SearchHit> {
        let mut scored: std::collections::HashMap<String, (SearchHit, f32)> =
            std::collections::HashMap::new();

        for hit in vector_hits {
            let key = format!("{}:{}", hit.note_id, hit.chunk_index);
            let score = hit.score * self.vector_weight;
            scored
                .entry(key)
                .and_modify(|(_, s)| *s += score)
                .or_insert((hit, score));
        }

        for hit in keyword_hits {
            let key = format!("{}:{}", hit.note_id, hit.chunk_index);
            let score = hit.score * self.keyword_weight;
            scored
                .entry(key)
                .and_modify(|(_, s)| *s += score)
                .or_insert((hit, score));
        }

        let mut results: Vec<SearchHit> = scored
            .into_values()
            .filter(|(_, score)| *score >= self.similarity_threshold * self.vector_weight)
            .map(|(mut hit, score)| {
                hit.score = score;
                hit.source = SearchSource::Hybrid;
                hit
            })
            .collect();

        results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
        results.truncate(self.top_k);
        results
    }
}

/// 코사인 유사도 계산
pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 { 0.0 } else { dot / (norm_a * norm_b) }
}
