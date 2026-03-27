use super::AudioChunk;

/// 슬라이딩 윈도우 어큐뮬레이터
///
/// 16kHz mono 샘플을 축적하다가 윈도우 크기에 도달하면
/// AudioChunk를 생성하여 STT 엔진에 전달합니다.
/// 오버랩을 사용하여 문맥 연속성을 확보합니다.
pub struct Accumulator {
    buffer: Vec<f32>,
    /// 윈도우 크기 (샘플 수)
    window_samples: usize,
    /// 오버랩 크기 (샘플 수)
    overlap_samples: usize,
    /// 녹음 시작 이후 총 처리된 샘플 수
    total_samples: u64,
    sample_rate: u32,
}

impl Accumulator {
    /// 새 어큐뮬레이터 생성
    ///
    /// - `window_secs`: 윈도우 크기 (초, 예: 3.0)
    /// - `overlap_secs`: 오버랩 크기 (초, 예: 0.5)
    /// - `sample_rate`: 샘플레이트 (16000)
    pub fn new(window_secs: f32, overlap_secs: f32, sample_rate: u32) -> Self {
        let window_samples = (window_secs * sample_rate as f32) as usize;
        let overlap_samples = (overlap_secs * sample_rate as f32) as usize;

        Self {
            buffer: Vec::with_capacity(window_samples),
            window_samples,
            overlap_samples,
            total_samples: 0,
            sample_rate,
        }
    }

    /// 샘플 추가 후, 준비된 AudioChunk가 있으면 반환
    pub fn push(&mut self, samples: &[f32]) -> Vec<AudioChunk> {
        self.buffer.extend_from_slice(samples);
        let mut chunks = Vec::new();

        while self.buffer.len() >= self.window_samples {
            let chunk_samples: Vec<f32> = self.buffer[..self.window_samples].to_vec();
            let timestamp_ms =
                (self.total_samples as f64 / self.sample_rate as f64 * 1000.0) as i64;

            chunks.push(AudioChunk::new(chunk_samples, timestamp_ms));

            // 오버랩을 유지하면서 윈도우 이동
            let advance = self.window_samples - self.overlap_samples;
            self.buffer.drain(..advance);
            self.total_samples += advance as u64;
        }

        chunks
    }

    /// 남은 버퍼 강제 플러시 (녹음 종료 시)
    pub fn flush(&mut self) -> Option<AudioChunk> {
        if self.buffer.is_empty() {
            return None;
        }

        let timestamp_ms =
            (self.total_samples as f64 / self.sample_rate as f64 * 1000.0) as i64;
        let chunk = AudioChunk::new(self.buffer.drain(..).collect(), timestamp_ms);
        Some(chunk)
    }

    /// 버퍼 초기화
    pub fn reset(&mut self) {
        self.buffer.clear();
        self.total_samples = 0;
    }

    /// 현재 버퍼 길이 (초)
    pub fn buffered_secs(&self) -> f32 {
        self.buffer.len() as f32 / self.sample_rate as f32
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_accumulator_produces_chunks() {
        let mut acc = Accumulator::new(1.0, 0.25, 16000);

        // 1초 분량 = 16000 샘플
        let samples = vec![0.1f32; 16000];
        let chunks = acc.push(&samples);

        assert_eq!(chunks.len(), 1);
        assert_eq!(chunks[0].samples.len(), 16000);
        assert_eq!(chunks[0].timestamp_ms, 0);
    }

    #[test]
    fn test_accumulator_overlap() {
        let mut acc = Accumulator::new(1.0, 0.5, 16000);

        // 2초 분량
        let samples = vec![0.1f32; 32000];
        let chunks = acc.push(&samples);

        // 1초 윈도우, 0.5초 오버랩 → 0.5초씩 전진 → 3개 청크 (0, 0.5, 1.0)
        // 실제: 윈도우=16000, 오버랩=8000, 전진=8000
        // 32000 / 8000 = 4번 전진 가능하지만 마지막은 부족할 수 있음
        assert!(chunks.len() >= 2);
    }

    #[test]
    fn test_flush_remaining() {
        let mut acc = Accumulator::new(1.0, 0.25, 16000);

        let samples = vec![0.1f32; 8000]; // 0.5초 (윈도우 미달)
        let chunks = acc.push(&samples);
        assert!(chunks.is_empty());

        let flushed = acc.flush();
        assert!(flushed.is_some());
        assert_eq!(flushed.unwrap().samples.len(), 8000);
    }
}
