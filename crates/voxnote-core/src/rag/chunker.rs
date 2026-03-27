/// 텍스트 청커 — RAG용 텍스트를 일정 크기로 분할
pub struct TextChunker {
    /// 청크당 최대 토큰 수 (대략적, 단어 기준)
    chunk_size: usize,
    /// 오버랩 비율 (0.0 ~ 0.5)
    overlap_ratio: f32,
}

#[derive(Debug, Clone)]
pub struct TextChunk {
    pub text: String,
    pub index: usize,
    pub start_char: usize,
    pub end_char: usize,
}

impl TextChunker {
    pub fn new(chunk_size: usize, overlap_ratio: f32) -> Self {
        Self {
            chunk_size,
            overlap_ratio: overlap_ratio.clamp(0.0, 0.5),
        }
    }

    /// 기본 설정 (512 토큰, 25% 오버랩)
    pub fn default_rag() -> Self {
        Self::new(512, 0.25)
    }

    /// 텍스트를 청크로 분할
    pub fn chunk(&self, text: &str) -> Vec<TextChunk> {
        let words: Vec<&str> = text.split_whitespace().collect();
        if words.is_empty() {
            return Vec::new();
        }

        let overlap = (self.chunk_size as f32 * self.overlap_ratio) as usize;
        let step = self.chunk_size.saturating_sub(overlap).max(1);

        let mut chunks = Vec::new();
        let mut start = 0;
        let mut index = 0;

        while start < words.len() {
            let end = (start + self.chunk_size).min(words.len());
            let chunk_text = words[start..end].join(" ");

            // 원본 텍스트에서의 위치 계산 (근사)
            let start_char = if start == 0 {
                0
            } else {
                text.find(words[start]).unwrap_or(0)
            };
            let end_char = if end >= words.len() {
                text.len()
            } else {
                text.find(words[end.min(words.len() - 1)]).unwrap_or(text.len())
            };

            chunks.push(TextChunk {
                text: chunk_text,
                index,
                start_char,
                end_char,
            });

            index += 1;
            start += step;

            if end >= words.len() {
                break;
            }
        }

        chunks
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_chunking() {
        let text = (0..100).map(|i| format!("word{}", i)).collect::<Vec<_>>().join(" ");
        let chunker = TextChunker::new(30, 0.25);
        let chunks = chunker.chunk(&text);

        assert!(chunks.len() > 1);
        assert!(chunks[0].text.contains("word0"));
        // 오버랩 확인
        if chunks.len() > 1 {
            let last_words_0: Vec<&str> = chunks[0].text.split_whitespace().rev().take(5).collect();
            let first_words_1: Vec<&str> = chunks[1].text.split_whitespace().take(10).collect();
            // 오버랩된 단어가 있어야 함
            assert!(last_words_0.iter().any(|w| first_words_1.contains(w)));
        }
    }
}
