pub mod accumulator;
#[cfg(feature = "desktop")]
pub mod capture;
pub mod resample;
pub mod ringbuf;
pub mod vad;

/// 오디오 청크 — STT 엔진에 전달되는 단위
#[derive(Debug, Clone)]
pub struct AudioChunk {
    /// PCM 샘플 (f32, 16kHz, mono)
    pub samples: Vec<f32>,
    /// 청크 시작 타임스탬프 (ms, 녹음 시작 기준)
    pub timestamp_ms: i64,
    /// 샘플레이트 (항상 16000)
    pub sample_rate: u32,
}

impl AudioChunk {
    pub fn new(samples: Vec<f32>, timestamp_ms: i64) -> Self {
        Self {
            samples,
            timestamp_ms,
            sample_rate: 16000,
        }
    }

    /// 청크 길이 (초)
    pub fn duration_secs(&self) -> f32 {
        self.samples.len() as f32 / self.sample_rate as f32
    }
}

/// 오디오 디바이스 정보
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct AudioDevice {
    pub name: String,
    pub is_default: bool,
    pub sample_rate: u32,
    pub channels: u16,
}
