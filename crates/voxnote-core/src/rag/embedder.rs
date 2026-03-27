use async_trait::async_trait;

use crate::error::VoxNoteError;

/// 임베딩 Provider 트레이트
#[async_trait]
pub trait EmbeddingProvider: Send + Sync {
    /// 텍스트를 벡터로 변환
    async fn embed(&self, text: &str) -> Result<Vec<f32>, VoxNoteError>;

    /// 배치 임베딩
    async fn embed_batch(&self, texts: &[String]) -> Result<Vec<Vec<f32>>, VoxNoteError> {
        let mut results = Vec::with_capacity(texts.len());
        for text in texts {
            results.push(self.embed(text).await?);
        }
        Ok(results)
    }

    /// 임베딩 차원 수
    fn dimension(&self) -> usize;

    fn name(&self) -> &str;
}

/// 간단한 TF-IDF 기반 임베딩 (ONNX 모델 대체용 스텁)
pub struct SimpleEmbedder {
    dimension: usize,
}

impl SimpleEmbedder {
    pub fn new(dimension: usize) -> Self {
        Self { dimension }
    }
}

#[async_trait]
impl EmbeddingProvider for SimpleEmbedder {
    async fn embed(&self, text: &str) -> Result<Vec<f32>, VoxNoteError> {
        // 단어 해시 기반 간단 임베딩 (프로덕션에서는 all-MiniLM-L6-v2 ONNX 사용)
        let mut vector = vec![0.0f32; self.dimension];
        for (i, word) in text.split_whitespace().enumerate() {
            let hash = simple_hash(word);
            let idx = hash % self.dimension;
            vector[idx] += 1.0;
        }
        // L2 정규화
        let norm: f32 = vector.iter().map(|x| x * x).sum::<f32>().sqrt();
        if norm > 0.0 {
            for v in &mut vector {
                *v /= norm;
            }
        }
        Ok(vector)
    }

    fn dimension(&self) -> usize {
        self.dimension
    }

    fn name(&self) -> &str {
        "simple-tfidf"
    }
}

fn simple_hash(s: &str) -> usize {
    s.bytes().fold(0usize, |acc, b| acc.wrapping_mul(31).wrapping_add(b as usize))
}
