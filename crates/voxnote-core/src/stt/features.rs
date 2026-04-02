//! 오디오 특징 추출 — mel spectrogram, fbank features, CMVN
//!
//! SenseVoice와 Qwen3-ASR ONNX 모델의 전처리에 사용됩니다.
//! feature gate: `stt-onnx`

use rustfft::num_complex::Complex;
use rustfft::FftPlanner;
use std::f32::consts::PI;

/// Mel spectrogram 계산 (Qwen3-ASR용)
///
/// Whisper-스타일 mel spectrogram:
/// - n_fft=400, hop_length=160, n_mels=128
/// - Hann window, Slaney mel scale, 0-8kHz
///
/// 반환: [n_frames, n_mels] 형태의 log-mel spectrogram
pub fn mel_spectrogram(
    samples: &[f32],
    sample_rate: u32,
    n_fft: usize,
    hop_length: usize,
    n_mels: usize,
) -> Vec<Vec<f32>> {
    let window = hann_window(n_fft);
    let stft = compute_stft(samples, n_fft, hop_length, &window);
    let power_spec = power_spectrum(&stft);
    let mel_filters = mel_filterbank(n_mels, n_fft, sample_rate);
    apply_mel_and_log(&power_spec, &mel_filters)
}

/// Fbank 특징 추출 (SenseVoice용)
///
/// - num_mel_bins=80, frame_length=25ms, frame_shift=10ms
/// - Dither=0, snip_edges=false
///
/// 반환: [n_frames, num_mel_bins] 형태의 log-fbank features
pub fn fbank_features(
    samples: &[f32],
    sample_rate: u32,
    num_mel_bins: usize,
    frame_length_ms: f32,
    frame_shift_ms: f32,
) -> Vec<Vec<f32>> {
    let frame_length = (sample_rate as f32 * frame_length_ms / 1000.0) as usize;
    let frame_shift = (sample_rate as f32 * frame_shift_ms / 1000.0) as usize;

    // n_fft는 frame_length 이상의 2의 거듭제곱
    let n_fft = frame_length.next_power_of_two();

    let window = hann_window(frame_length);
    let stft = compute_stft_with_pad(samples, n_fft, frame_length, frame_shift, &window);
    let power_spec = power_spectrum(&stft);
    let mel_filters = mel_filterbank(num_mel_bins, n_fft, sample_rate);
    apply_mel_and_log(&power_spec, &mel_filters)
}

/// CMVN (Cepstral Mean-Variance Normalization) 적용
///
/// SenseVoice의 am.mvn 통계를 사용하여 정규화합니다.
pub fn apply_cmvn(features: &mut [Vec<f32>], mean: &[f32], istd: &[f32]) {
    for frame in features.iter_mut() {
        for (i, val) in frame.iter_mut().enumerate() {
            if i < mean.len() {
                *val = (*val - mean[i]) * istd[i];
            }
        }
    }
}

/// Kaldi 형식 CMVN 파일 파싱 (am.mvn)
///
/// 형식: 공백 구분 텍스트, 행 = [sum, sumsq, count] 또는 mean/var 벡터
/// sherpa-onnx 배포판 기준: 첫 번째 행 = mean, 두 번째 행 = inverse std
pub fn parse_cmvn_file(content: &str) -> Option<(Vec<f32>, Vec<f32>)> {
    let parse_line = |line: &str| -> Vec<f32> {
        line.split_whitespace()
            .filter_map(|s| {
                let s = s.trim_matches(|c: char| c == '[' || c == ']');
                if s.is_empty() {
                    return None;
                }
                s.parse::<f32>().ok()
            })
            .collect()
    };

    // 숫자를 포함하는 행만 수집
    let vectors: Vec<Vec<f32>> = content
        .lines()
        .map(|l| parse_line(l))
        .filter(|v| !v.is_empty())
        .collect();

    if vectors.len() < 2 {
        return None;
    }

    Some((vectors[0].clone(), vectors[1].clone()))
}

// ── 내부 구현 ────────────────────────────────────────────────

/// Hann 윈도우 생성
fn hann_window(length: usize) -> Vec<f32> {
    (0..length)
        .map(|i| 0.5 * (1.0 - (2.0 * PI * i as f32 / length as f32).cos()))
        .collect()
}

/// STFT 계산 (n_fft == window_length)
fn compute_stft(
    samples: &[f32],
    n_fft: usize,
    hop_length: usize,
    window: &[f32],
) -> Vec<Vec<Complex<f32>>> {
    compute_stft_with_pad(samples, n_fft, n_fft, hop_length, window)
}

/// STFT 계산 (window_length과 n_fft 독립)
fn compute_stft_with_pad(
    samples: &[f32],
    n_fft: usize,
    window_length: usize,
    hop_length: usize,
    window: &[f32],
) -> Vec<Vec<Complex<f32>>> {
    let mut planner = FftPlanner::<f32>::new();
    let fft = planner.plan_fft_forward(n_fft);

    let n_frames = if samples.len() >= window_length {
        (samples.len() - window_length) / hop_length + 1
    } else {
        1
    };

    let mut result = Vec::with_capacity(n_frames);

    for frame_idx in 0..n_frames {
        let start = frame_idx * hop_length;
        let mut buffer = vec![Complex::new(0.0, 0.0); n_fft];

        for i in 0..window_length.min(samples.len().saturating_sub(start)) {
            let sample = if start + i < samples.len() {
                samples[start + i]
            } else {
                0.0
            };
            let w = if i < window.len() { window[i] } else { 1.0 };
            buffer[i] = Complex::new(sample * w, 0.0);
        }

        fft.process(&mut buffer);
        // 양수 주파수만 (n_fft/2 + 1)
        result.push(buffer[..n_fft / 2 + 1].to_vec());
    }

    result
}

/// 파워 스펙트럼: |STFT|^2
fn power_spectrum(stft: &[Vec<Complex<f32>>]) -> Vec<Vec<f32>> {
    stft.iter()
        .map(|frame| frame.iter().map(|c| c.norm_sqr()).collect())
        .collect()
}

/// Mel filterbank 생성 (Slaney 방식)
fn mel_filterbank(n_mels: usize, n_fft: usize, sample_rate: u32) -> Vec<Vec<f32>> {
    let n_freqs = n_fft / 2 + 1;
    let f_max = sample_rate as f32 / 2.0;
    let f_min = 0.0;

    let mel_min = hz_to_mel(f_min);
    let mel_max = hz_to_mel(f_max);

    // n_mels + 2 mel-spaced points
    let mel_points: Vec<f32> = (0..=n_mels + 1)
        .map(|i| mel_min + (mel_max - mel_min) * i as f32 / (n_mels + 1) as f32)
        .collect();

    let hz_points: Vec<f32> = mel_points.iter().map(|&m| mel_to_hz(m)).collect();

    let bin_points: Vec<f32> = hz_points
        .iter()
        .map(|&f| f * n_fft as f32 / sample_rate as f32)
        .collect();

    let mut filters = vec![vec![0.0f32; n_freqs]; n_mels];

    for i in 0..n_mels {
        let left = bin_points[i];
        let center = bin_points[i + 1];
        let right = bin_points[i + 2];

        for j in 0..n_freqs {
            let freq = j as f32;
            if freq >= left && freq <= center && center > left {
                filters[i][j] = (freq - left) / (center - left);
            } else if freq > center && freq <= right && right > center {
                filters[i][j] = (right - freq) / (right - center);
            }
        }
    }

    filters
}

/// Mel filterbank 적용 + log 변환
fn apply_mel_and_log(power_spec: &[Vec<f32>], mel_filters: &[Vec<f32>]) -> Vec<Vec<f32>> {
    let n_mels = mel_filters.len();
    let floor = 1e-10_f32;

    power_spec
        .iter()
        .map(|frame| {
            (0..n_mels)
                .map(|m| {
                    let energy: f32 = frame
                        .iter()
                        .zip(mel_filters[m].iter())
                        .map(|(p, f)| p * f)
                        .sum();
                    energy.max(floor).ln()
                })
                .collect()
        })
        .collect()
}

/// Hz → Mel 변환 (Slaney)
fn hz_to_mel(hz: f32) -> f32 {
    2595.0 * (1.0 + hz / 700.0).log10()
}

/// Mel → Hz 변환 (Slaney)
fn mel_to_hz(mel: f32) -> f32 {
    700.0 * (10.0_f32.powf(mel / 2595.0) - 1.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hann_window() {
        let w = hann_window(4);
        assert_eq!(w.len(), 4);
        // Hann(4): [0, 0.75, 0.75, ~0]
        // w[n] = 0.5 * (1 - cos(2*pi*n/N))
        // w[0] = 0, w[1] = 0.5*(1-cos(pi/2)) = 0.5, w[2] = 0.5*(1-cos(pi)) = 1.0, w[3] = 0.5*(1-cos(3pi/2)) = 0.5
        // But for periodic Hann(4): w[n] = 0.5*(1 - cos(2*pi*n/4))
        assert!((w[0] - 0.0).abs() < 1e-6);
        // w[1] = 0.5 * (1 - cos(pi/2)) = 0.5
        assert!((w[1] - 0.5).abs() < 1e-6);
        // w[2] = 0.5 * (1 - cos(pi)) = 1.0
        assert!((w[2] - 1.0).abs() < 1e-6);
        // w[3] = 0.5 * (1 - cos(3pi/2)) = 0.5
        assert!((w[3] - 0.5).abs() < 1e-4);
    }

    #[test]
    fn test_mel_filterbank_shape() {
        let filters = mel_filterbank(80, 512, 16000);
        assert_eq!(filters.len(), 80);
        assert_eq!(filters[0].len(), 257); // 512/2 + 1
    }

    #[test]
    fn test_mel_spectrogram_shape() {
        // 1초 16kHz 무음
        let samples = vec![0.0f32; 16000];
        let mel = mel_spectrogram(&samples, 16000, 400, 160, 128);
        // (16000 - 400) / 160 + 1 = 98 frames
        assert_eq!(mel.len(), 98);
        assert_eq!(mel[0].len(), 128);
    }

    #[test]
    fn test_fbank_features_shape() {
        // 1초 16kHz 무음
        let samples = vec![0.0f32; 16000];
        let fbank = fbank_features(&samples, 16000, 80, 25.0, 10.0);
        // frame_length = 400, frame_shift = 160
        // (16000 - 400) / 160 + 1 = 98 frames
        assert_eq!(fbank.len(), 98);
        assert_eq!(fbank[0].len(), 80);
    }

    #[test]
    fn test_apply_cmvn() {
        let mut features = vec![vec![1.0, 2.0, 3.0], vec![4.0, 5.0, 6.0]];
        let mean = vec![1.0, 1.0, 1.0];
        let istd = vec![2.0, 2.0, 2.0];
        apply_cmvn(&mut features, &mean, &istd);
        assert_eq!(features[0], vec![0.0, 2.0, 4.0]);
        assert_eq!(features[1], vec![6.0, 8.0, 10.0]);
    }

    #[test]
    fn test_parse_cmvn_file() {
        let content = "[ 0.1 0.2 0.3 ]\n[ 1.0 1.0 1.0 ]\n";
        let (mean, istd) = parse_cmvn_file(content).unwrap();
        assert_eq!(mean.len(), 3);
        assert_eq!(istd.len(), 3);
        assert!((mean[0] - 0.1).abs() < 1e-6);
    }

    #[test]
    fn test_hz_mel_roundtrip() {
        let hz = 1000.0;
        let mel = hz_to_mel(hz);
        let back = mel_to_hz(mel);
        assert!((hz - back).abs() < 0.1);
    }
}
