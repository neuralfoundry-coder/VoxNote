use crate::error::AudioError;

/// Voice Activity Detection 트레이트
pub trait VoiceActivityDetector: Send + Sync {
    /// 음성 프레임(16kHz mono f32)에 대해 음성 확률 반환 (0.0 ~ 1.0)
    fn detect(&mut self, samples: &[f32]) -> Result<f32, AudioError>;

    /// 현재 임계값 기준으로 음성 구간인지 여부
    fn is_speech(&mut self, samples: &[f32]) -> Result<bool, AudioError> {
        let probability = self.detect(samples)?;
        Ok(probability >= self.threshold())
    }

    /// 현재 임계값
    fn threshold(&self) -> f32;

    /// 임계값 설정
    fn set_threshold(&mut self, threshold: f32);
}

/// 에너지 기반 간단한 VAD 구현 (Silero VAD ONNX 모델 대체용)
///
/// Phase 2에서 Silero VAD (ONNX via ort)로 교체 가능
pub struct EnergyVad {
    threshold: f32,
    /// 에너지 임계값 (RMS 기준)
    energy_threshold: f32,
}

impl EnergyVad {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold,
            energy_threshold: 0.002,
        }
    }

    /// 에너지 기준값 설정 (기본 0.002, 낮을수록 민감)
    pub fn set_energy_floor(&mut self, floor: f32) {
        self.energy_threshold = floor;
    }
}

impl VoiceActivityDetector for EnergyVad {
    fn detect(&mut self, samples: &[f32]) -> Result<f32, AudioError> {
        if samples.is_empty() {
            return Ok(0.0);
        }

        // RMS 에너지 계산
        let rms = (samples.iter().map(|s| s * s).sum::<f32>() / samples.len() as f32).sqrt();

        // 에너지를 확률로 변환 (sigmoid-like)
        let probability = if rms < self.energy_threshold * 0.5 {
            0.0
        } else if rms > self.energy_threshold * 2.0 {
            1.0
        } else {
            (rms / (self.energy_threshold * 2.0)).clamp(0.0, 1.0)
        };

        Ok(probability)
    }

    fn threshold(&self) -> f32 {
        self.threshold
    }

    fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.0, 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_silence_detection() {
        let mut vad = EnergyVad::new(0.5);
        let silence = vec![0.0f32; 480];
        assert!(!vad.is_speech(&silence).unwrap());
    }

    #[test]
    fn test_speech_detection() {
        let mut vad = EnergyVad::new(0.5);
        // 큰 진폭의 신호
        let speech: Vec<f32> = (0..480).map(|i| (i as f32 * 0.1).sin() * 0.5).collect();
        assert!(vad.is_speech(&speech).unwrap());
    }
}
