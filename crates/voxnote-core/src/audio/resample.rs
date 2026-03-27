use rubato::{FftFixedIn, Resampler as RubatoResampler};
use tracing::debug;

use crate::error::AudioError;

/// rubato 기반 오디오 리샘플러 (→ 16kHz mono)
pub struct Resampler {
    resampler: FftFixedIn<f32>,
    chunk_size: usize,
    input_buffer: Vec<Vec<f32>>,
}

const TARGET_RATE: u32 = 16000;

impl Resampler {
    /// 새 리샘플러 생성
    ///
    /// - `input_rate`: 입력 샘플레이트 (예: 48000)
    /// - `input_channels`: 입력 채널 수 (예: 2 = stereo)
    pub fn new(input_rate: u32, input_channels: u16) -> Result<Self, AudioError> {
        let chunk_size = 1024;
        let resampler = FftFixedIn::new(
            input_rate as usize,
            TARGET_RATE as usize,
            chunk_size,
            2, // sub_chunks
            1, // output channels (mono)
        )
        .map_err(|e| AudioError::Resample(e.to_string()))?;

        debug!(
            "Resampler created: {}Hz {}ch → {}Hz mono",
            input_rate, input_channels, TARGET_RATE
        );

        Ok(Self {
            resampler,
            chunk_size,
            input_buffer: vec![Vec::new()],
        })
    }

    /// 다채널 f32 샘플 → 16kHz mono f32 변환
    ///
    /// 입력이 충분히 누적되면 변환된 샘플을 반환하고,
    /// 아직 부족하면 빈 Vec을 반환합니다.
    pub fn process(&mut self, input: &[f32], channels: u16) -> Result<Vec<f32>, AudioError> {
        // 스테레오 → 모노 다운믹스
        let mono: Vec<f32> = if channels > 1 {
            input
                .chunks_exact(channels as usize)
                .map(|frame| frame.iter().sum::<f32>() / channels as f32)
                .collect()
        } else {
            input.to_vec()
        };

        self.input_buffer[0].extend_from_slice(&mono);

        let mut output = Vec::new();

        while self.input_buffer[0].len() >= self.chunk_size {
            let chunk: Vec<Vec<f32>> = vec![
                self.input_buffer[0].drain(..self.chunk_size).collect()
            ];

            match self.resampler.process(&chunk, None) {
                Ok(resampled) => {
                    if let Some(channel) = resampled.into_iter().next() {
                        output.extend(channel);
                    }
                }
                Err(e) => return Err(AudioError::Resample(e.to_string())),
            }
        }

        Ok(output)
    }

    /// 잔여 버퍼 플러시 (녹음 종료 시)
    pub fn flush(&mut self) -> Result<Vec<f32>, AudioError> {
        if self.input_buffer[0].is_empty() {
            return Ok(Vec::new());
        }

        // 남은 샘플을 0으로 패딩하여 처리
        let remaining = self.input_buffer[0].len();
        if remaining < self.chunk_size {
            self.input_buffer[0].resize(self.chunk_size, 0.0);
        }

        let chunk: Vec<Vec<f32>> = vec![
            self.input_buffer[0].drain(..).collect()
        ];

        match self.resampler.process(&chunk, None) {
            Ok(resampled) => {
                Ok(resampled.into_iter().next().unwrap_or_default())
            }
            Err(e) => Err(AudioError::Resample(e.to_string())),
        }
    }

    pub fn output_rate(&self) -> u32 {
        TARGET_RATE
    }
}
