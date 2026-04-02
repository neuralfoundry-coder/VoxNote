//! SenseVoiceSmall ONNX 기반 STT Provider
//!
//! Non-autoregressive 단일 forward pass 모델.
//! sherpa-onnx int8 ONNX 모델 사용.
//! feature gate: `stt-onnx`

use async_trait::async_trait;
use std::path::Path;
use std::sync::Mutex;
use tracing::{debug, info};

use super::features::fbank_features;
use super::{Language, SttProvider};
use crate::audio::AudioChunk;
use crate::error::SttError;
use crate::models::Segment;

/// SenseVoiceSmall 언어 코드 → 정수 매핑
const LANG_AUTO: i32 = 0;
const LANG_ZH: i32 = 3;
const LANG_EN: i32 = 4;
const LANG_JA: i32 = 11;
const LANG_KO: i32 = 7;
const LANG_YUE: i32 = 13;

/// SenseVoice ONNX STT Provider
///
/// CMVN 정규화는 ONNX 그래프에 내장되어 있으므로 별도 am.mvn 불필요.
pub struct SenseVoiceSttProvider {
    session: Mutex<ort::session::Session>,
    languages: Vec<Language>,
    language: Mutex<Option<String>>,
    initial_prompt: Mutex<String>,
    vocab: Vec<String>,
}

impl SenseVoiceSttProvider {
    /// 모델 디렉토리에서 로드
    ///
    /// 디렉토리 구조:
    /// - model.int8.onnx (또는 model.onnx)
    /// - tokens.txt
    ///
    /// Note: CMVN 정규화는 ONNX 그래프에 내장됨 (별도 am.mvn 불필요)
    pub fn new(model_dir: &Path) -> Result<Self, SttError> {
        // ONNX 모델 파일 탐색
        let model_path = find_onnx_model(model_dir)?;

        info!("Loading SenseVoice model: {:?}", model_path);

        let mut builder = ort::session::Session::builder()
            .map_err(|e| SttError::Inference(format!("Session builder failed: {}", e)))?;
        builder = builder.with_intra_threads(4)
            .map_err(|e| SttError::Inference(format!("Thread config failed: {}", e)))?;
        let session = builder.commit_from_file(&model_path)
            .map_err(|e| SttError::Inference(format!("Failed to load ONNX model: {}", e)))?;

        // tokens.txt 로드
        let tokens_path = model_dir.join("tokens.txt");
        let vocab = load_vocabulary(&tokens_path)?;
        info!("SenseVoice vocabulary loaded: {} tokens", vocab.len());

        let languages = vec![
            Language::auto(),
            Language::new("ko", "Korean"),
            Language::new("en", "English"),
            Language::new("ja", "Japanese"),
            Language::new("zh", "Chinese"),
            Language::new("yue", "Cantonese"),
        ];

        Ok(Self {
            session: Mutex::new(session),
            languages,
            language: Mutex::new(None),
            initial_prompt: Mutex::new(String::new()),
            vocab,
        })
    }

    /// 언어 코드 → SenseVoice 정수 매핑
    fn language_id(&self) -> i32 {
        let lang = self.language.lock().unwrap();
        match lang.as_deref() {
            Some("zh") => LANG_ZH,
            Some("en") => LANG_EN,
            Some("ja") => LANG_JA,
            Some("ko") => LANG_KO,
            Some("yue") => LANG_YUE,
            _ => LANG_AUTO,
        }
    }

    /// 추론 실행 (동기)
    fn transcribe_sync(
        &self,
        audio: &AudioChunk,
        note_id: &str,
    ) -> Result<Vec<Segment>, SttError> {
        // 1. Fbank 특징 추출 (80 bins, 25ms frame, 10ms shift)
        // Note: CMVN 정규화는 ONNX 그래프 내부에서 처리됨
        let features = fbank_features(&audio.samples, audio.sample_rate, 80, 25.0, 10.0);

        if features.is_empty() {
            return Ok(Vec::new());
        }

        let n_frames = features.len();
        let n_bins = features[0].len();

        // 3. 텐서 구성: [1, n_frames, n_bins]
        let flat_features: Vec<f32> = features.into_iter().flatten().collect();

        let x = ort::value::Tensor::from_array(([1usize, n_frames, n_bins], flat_features))
            .map_err(|e| SttError::Inference(format!("Tensor creation failed: {}", e)))?;

        let x_length = ort::value::Tensor::from_array(([1usize], vec![n_frames as i32]))
            .map_err(|e| SttError::Inference(format!("Tensor creation failed: {}", e)))?;

        let language_id = self.language_id();
        let language_tensor = ort::value::Tensor::from_array(([1usize], vec![language_id]))
            .map_err(|e| SttError::Inference(format!("Tensor creation failed: {}", e)))?;

        // text_norm: 1 = with inverse text normalization
        let text_norm = ort::value::Tensor::from_array(([1usize], vec![1i32]))
            .map_err(|e| SttError::Inference(format!("Tensor creation failed: {}", e)))?;

        // 4. 추론
        let mut session = self.session.lock().map_err(|e| {
            SttError::Inference(format!("Session lock failed: {}", e))
        })?;

        let outputs = session
            .run(ort::inputs![x, x_length, language_tensor, text_norm])
            .map_err(|e| SttError::Inference(format!("SenseVoice inference failed: {}", e)))?;

        // 5. 출력 디코딩 — logits에서 token IDs 추출
        let (_shape, logits_data) = outputs[0]
            .try_extract_tensor::<i64>()
            .map_err(|e| SttError::Inference(format!("Output extraction failed: {}", e)))?;

        let text = decode_tokens(logits_data, &self.vocab);
        let text = clean_sensevoice_output(&text);

        if text.trim().is_empty() {
            return Ok(Vec::new());
        }

        // SenseVoice는 정밀 타임스탬프 미지원 → 청크 경계 기반 근사
        let segment = Segment::new(
            note_id,
            text.trim(),
            audio.timestamp_ms,
            audio.timestamp_ms + audio.duration_ms(),
        );

        debug!(
            "SenseVoice transcribed: '{}' ({:.1}s audio)",
            segment.text,
            audio.duration_secs()
        );

        Ok(vec![segment])
    }
}

#[async_trait]
impl SttProvider for SenseVoiceSttProvider {
    async fn transcribe(
        &self,
        audio: &AudioChunk,
        note_id: &str,
    ) -> Result<Vec<Segment>, SttError> {
        self.transcribe_sync(audio, note_id)
    }

    fn supported_languages(&self) -> &[Language] {
        &self.languages
    }

    fn set_initial_prompt(&self, prompt: &str) {
        *self.initial_prompt.lock().unwrap() = prompt.to_string();
    }

    fn set_language(&self, language: Option<&str>) {
        *self.language.lock().unwrap() = language.map(String::from);
    }

    fn name(&self) -> &str {
        "sensevoice-local"
    }
}

// ── 유틸리티 함수 ──────────────────────────────────────────────

/// 모델 디렉토리에서 ONNX 파일 탐색
fn find_onnx_model(model_dir: &Path) -> Result<std::path::PathBuf, SttError> {
    let candidates = ["model.int8.onnx", "model.onnx"];

    for name in &candidates {
        let path = model_dir.join(name);
        if path.exists() {
            return Ok(path);
        }
    }

    // 아무 .onnx 파일이라도 찾기
    if let Ok(entries) = std::fs::read_dir(model_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().map(|e| e == "onnx").unwrap_or(false) {
                return Ok(path);
            }
        }
    }

    Err(SttError::Inference(format!(
        "No ONNX model found in {:?}",
        model_dir
    )))
}

/// tokens.txt 로드 (한 줄에 하나의 토큰, 탭으로 ID/토큰 분리)
fn load_vocabulary(path: &Path) -> Result<Vec<String>, SttError> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| SttError::Inference(format!("Failed to read tokens.txt: {}", e)))?;

    let mut vocab = Vec::new();

    // 형식: "token id" (sherpa-onnx tokens.txt)
    for line in content.lines() {
        let parts: Vec<&str> = line.split_whitespace().collect();
        if parts.len() >= 2 {
            if let Ok(id) = parts.last().unwrap().parse::<usize>() {
                let token = parts[..parts.len() - 1].join(" ");
                if id >= vocab.len() {
                    vocab.resize(id + 1, String::new());
                }
                vocab[id] = token;
            }
        }
    }

    if vocab.is_empty() {
        return Err(SttError::Inference("Empty vocabulary".to_string()));
    }

    Ok(vocab)
}

/// 토큰 ID → 텍스트 디코딩
fn decode_tokens(token_ids: &[i64], vocab: &[String]) -> String {
    let mut result = String::new();
    for &id in token_ids {
        let id = id as usize;
        if id >= vocab.len() || vocab[id].is_empty() {
            continue;
        }
        let token = &vocab[id];
        // 특수 토큰 건너뛰기
        if token.starts_with('<') && token.ends_with('>') {
            continue;
        }
        // SentencePiece 언더스코어 → 공백
        if token.starts_with('\u{2581}') {
            result.push(' ');
            result.push_str(&token[3..]); // ▁ is 3 bytes in UTF-8
        } else {
            result.push_str(token);
        }
    }
    result
}

/// SenseVoice 출력 정리 (감정 태그, 이벤트 태그 제거)
fn clean_sensevoice_output(text: &str) -> String {
    let mut result = text.to_string();
    let tags = [
        "<|HAPPY|>", "<|SAD|>", "<|ANGRY|>", "<|NEUTRAL|>",
        "<|BGM|>", "<|Applause|>", "<|Laughter|>", "<|Crying|>",
        "<|Speech|>", "<|Silence|>",
    ];
    for tag in &tags {
        result = result.replace(tag, "");
    }
    result.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_sensevoice_output() {
        let input = "<|HAPPY|><|Speech|>Hello world<|BGM|>";
        assert_eq!(clean_sensevoice_output(input), "Hello world");
    }

    #[test]
    fn test_decode_tokens_skips_special() {
        let vocab = vec![
            "<sos>".to_string(),
            "\u{2581}Hello".to_string(),
            "\u{2581}world".to_string(),
            "<eos>".to_string(),
        ];
        let tokens = vec![0i64, 1, 2, 3];
        let text = decode_tokens(&tokens, &vocab);
        assert_eq!(text.trim(), "Hello world");
    }
}
