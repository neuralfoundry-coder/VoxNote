//! Qwen3-ASR-0.6B ONNX 기반 STT Provider
//!
//! Autoregressive encoder-decoder 모델.
//! andrewleech/qwen3-asr-0.6b-onnx ONNX 변환 사용.
//! feature gate: `stt-onnx`

use async_trait::async_trait;
use std::path::Path;
use std::sync::Mutex;
use tracing::{debug, info};

use super::features::mel_spectrogram;
use super::{Language, SttProvider};
use crate::audio::AudioChunk;
use crate::error::SttError;
use crate::models::Segment;

/// 특수 토큰 ID
const EOS_TOKEN: i64 = 151645; // <|endoftext|> / <|im_end|>
const MAX_DECODE_STEPS: usize = 448;

/// Qwen3-ASR ONNX STT Provider
///
/// 3개의 ONNX 세션으로 구성:
/// - encoder: 오디오 → hidden states
/// - decoder_init: 첫 번째 디코딩 스텝 (KV cache 초기화)
/// - decoder_step: 이후 디코딩 스텝 (KV cache 업데이트)
pub struct QwenAsrSttProvider {
    encoder: Mutex<ort::session::Session>,
    decoder_init: Mutex<ort::session::Session>,
    decoder_step: Mutex<ort::session::Session>,
    languages: Vec<Language>,
    language: Mutex<Option<String>>,
    initial_prompt: Mutex<String>,
    vocab: Vec<String>,
}

impl QwenAsrSttProvider {
    /// 모델 디렉토리에서 로드
    ///
    /// 필요 파일:
    /// - encoder.onnx (또는 encoder.int4.onnx)
    /// - decoder_init.onnx (또는 decoder_init.int4.onnx) + decoder_weights.*.data
    /// - decoder_step.onnx (또는 decoder_step.int4.onnx) + decoder_weights.*.data
    /// - vocab.json 또는 tokenizer.json
    pub fn new(model_dir: &Path) -> Result<Self, SttError> {
        let find_and_load = |base_name: &str| -> Result<ort::session::Session, SttError> {
            // int4 variant 우선, 없으면 기본
            let candidates = [
                format!("{}.int4.onnx", base_name),
                format!("{}.onnx", base_name),
            ];
            let path = candidates
                .iter()
                .map(|n| model_dir.join(n))
                .find(|p| p.exists())
                .ok_or_else(|| SttError::Inference(format!("Missing {} in {:?}", base_name, model_dir)))?;

            info!("Loading ONNX session: {:?}", path);
            let mut builder = ort::session::Session::builder()
                .map_err(|e| SttError::Inference(format!("Builder failed for {}: {}", base_name, e)))?;
            builder = builder.with_intra_threads(4)
                .map_err(|e| SttError::Inference(format!("Thread config failed for {}: {}", base_name, e)))?;
            builder.commit_from_file(&path)
                .map_err(|e| SttError::Inference(format!("Failed to load {:?}: {}", path, e)))
        };

        info!("Loading Qwen3-ASR model from {:?}", model_dir);

        let encoder = find_and_load("encoder")?;
        let decoder_init = find_and_load("decoder_init")?;
        let decoder_step = find_and_load("decoder_step")?;

        info!("Qwen3-ASR ONNX sessions loaded");

        let vocab = load_vocab(model_dir)?;
        info!("Qwen3-ASR vocabulary loaded: {} tokens", vocab.len());

        let languages = vec![
            Language::auto(),
            Language::new("ko", "Korean"),
            Language::new("en", "English"),
            Language::new("ja", "Japanese"),
            Language::new("zh", "Chinese"),
            Language::new("es", "Spanish"),
            Language::new("fr", "French"),
            Language::new("de", "German"),
        ];

        Ok(Self {
            encoder: Mutex::new(encoder),
            decoder_init: Mutex::new(decoder_init),
            decoder_step: Mutex::new(decoder_step),
            languages,
            language: Mutex::new(None),
            initial_prompt: Mutex::new(String::new()),
            vocab,
        })
    }

    /// 추론 실행 (동기)
    fn transcribe_sync(
        &self,
        audio: &AudioChunk,
        note_id: &str,
    ) -> Result<Vec<Segment>, SttError> {
        // 1. Mel spectrogram (128 bins, Whisper 호환)
        let mel = mel_spectrogram(&audio.samples, audio.sample_rate, 400, 160, 128);

        if mel.is_empty() {
            return Ok(Vec::new());
        }

        let n_frames = mel.len();
        let n_mels = mel[0].len();

        // [1, n_mels, n_frames] (채널 우선 — encoder 입력 형식)
        let mut flat_mel = vec![0.0f32; n_mels * n_frames];
        for (t, frame) in mel.iter().enumerate() {
            for (m, &val) in frame.iter().enumerate() {
                flat_mel[m * n_frames + t] = val;
            }
        }

        let mel_tensor =
            ort::value::Tensor::from_array(([1usize, n_mels, n_frames], flat_mel))
                .map_err(|e| SttError::Inference(format!("Mel tensor failed: {}", e)))?;

        // 2. Encoder
        let encoder_values: Vec<(String, ort::value::DynValue)> = {
            let mut encoder = self.encoder.lock().map_err(|e| {
                SttError::Inference(format!("Encoder lock failed: {}", e))
            })?;
            let out = encoder
                .run(ort::inputs![mel_tensor])
                .map_err(|e| SttError::Inference(format!("Encoder failed: {}", e)))?;
            out.into_iter().map(|(k, v)| (k.to_string(), v)).collect()
        };

        let encoder_hidden = encoder_values
            .into_iter()
            .next()
            .map(|(_, v)| v)
            .ok_or_else(|| SttError::Inference("No encoder output".to_string()))?;

        // 3. Decoder init
        let start_token = ort::value::Tensor::from_array(([1i64, 1], vec![EOS_TOKEN]))
            .map_err(|e| SttError::Inference(format!("Start token tensor failed: {}", e)))?;

        // 소유된 DynValue 벡터로 수집하여 세션 borrow 해제
        let (first_logits_data, mut kv_cache): (Vec<f32>, Vec<(String, ort::value::DynValue)>) = {
            let mut decoder_init = self.decoder_init.lock().map_err(|e| {
                SttError::Inference(format!("Decoder init lock failed: {}", e))
            })?;
            let out = decoder_init
                .run(ort::inputs!["encoder_hidden_states" => encoder_hidden, "input_ids" => start_token])
                .map_err(|e| SttError::Inference(format!("Decoder init failed: {}", e)))?;

            let mut owned: Vec<(String, ort::value::DynValue)> = out
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect();

            // 첫 번째 = logits, 나머지 = KV cache
            let logits_val = owned.remove(0);
            let (_shape, data) = logits_val.1
                .try_extract_tensor::<f32>()
                .map_err(|e| SttError::Inference(format!("Init logits extraction failed: {}", e)))?;
            (data.to_vec(), owned)
        };

        // 4. Autoregressive 디코딩
        let mut generated_tokens: Vec<i64> = Vec::new();

        let first_token = argmax_from_logits(&first_logits_data);
        if first_token == EOS_TOKEN {
            return Ok(Vec::new());
        }
        generated_tokens.push(first_token);

        let mut decoder_step = self.decoder_step.lock().map_err(|e| {
            SttError::Inference(format!("Decoder step lock failed: {}", e))
        })?;

        let step_input_names: Vec<String> = decoder_step
            .inputs()
            .iter()
            .map(|i| i.name().to_string())
            .collect();

        for step in 1..MAX_DECODE_STEPS {
            let prev_token = *generated_tokens.last().unwrap();

            let token_tensor =
                ort::value::Tensor::from_array(([1i64, 1], vec![prev_token]))
                    .map_err(|e| {
                        SttError::Inference(format!("Token tensor step {} failed: {}", step, e))
                    })?;

            // input_ids + KV caches를 named inputs로 구성
            let mut step_inputs: Vec<(
                std::borrow::Cow<'_, str>,
                ort::session::SessionInputValue<'_>,
            )> = Vec::new();

            step_inputs.push((
                std::borrow::Cow::Owned(step_input_names[0].clone()),
                token_tensor.into(),
            ));

            // KV cache 전달
            for (i, (_name, val)) in kv_cache.drain(..).enumerate() {
                let input_name = if i + 1 < step_input_names.len() {
                    step_input_names[i + 1].clone()
                } else {
                    _name
                };
                step_inputs.push((
                    std::borrow::Cow::Owned(input_name),
                    val.into(),
                ));
            }

            let step_out = decoder_step
                .run(step_inputs)
                .map_err(|e| {
                    SttError::Inference(format!("Decoder step {} failed: {}", step, e))
                })?;

            // 출력을 소유된 벡터로 수집
            let mut step_owned: Vec<(String, ort::value::DynValue)> = step_out
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect();

            // logits (첫 번째) 추출
            let logits_val = step_owned.remove(0);
            let (_shape, logits_data) = logits_val.1
                .try_extract_tensor::<f32>()
                .map_err(|e| SttError::Inference(format!("Step logits failed: {}", e)))?;

            let next_token = argmax_from_logits(logits_data);

            if next_token == EOS_TOKEN {
                break;
            }

            generated_tokens.push(next_token);
            kv_cache = step_owned;
        }

        drop(decoder_step);

        // 5. 토큰 → 텍스트
        let text = decode_tokens(&generated_tokens, &self.vocab);
        let text = text.trim();

        if text.is_empty() {
            return Ok(Vec::new());
        }

        let segment = Segment::new(
            note_id,
            text,
            audio.timestamp_ms,
            audio.timestamp_ms + audio.duration_ms(),
        );

        debug!(
            "Qwen3-ASR transcribed: '{}' ({:.1}s audio, {} tokens)",
            segment.text,
            audio.duration_secs(),
            generated_tokens.len()
        );

        Ok(vec![segment])
    }
}

#[async_trait]
impl SttProvider for QwenAsrSttProvider {
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
        "qwen-asr-local"
    }
}

// ── 유틸리티 함수 ──────────────────────────────────────────────

/// 어휘 파일 로드 (vocab.json 또는 tokenizer.json)
fn load_vocab(model_dir: &Path) -> Result<Vec<String>, SttError> {
    // vocab.json
    let vocab_json = model_dir.join("vocab.json");
    if vocab_json.exists() {
        let content = std::fs::read_to_string(&vocab_json)
            .map_err(|e| SttError::Inference(format!("Failed to read vocab.json: {}", e)))?;
        let map: std::collections::HashMap<String, usize> = serde_json::from_str(&content)
            .map_err(|e| SttError::Inference(format!("Failed to parse vocab.json: {}", e)))?;

        let max_id = map.values().copied().max().unwrap_or(0);
        let mut vocab = vec![String::new(); max_id + 1];
        for (token, id) in map {
            vocab[id] = token;
        }
        return Ok(vocab);
    }

    // tokenizer.json (HuggingFace 형식)
    let tokenizer_json = model_dir.join("tokenizer.json");
    if tokenizer_json.exists() {
        let content = std::fs::read_to_string(&tokenizer_json)
            .map_err(|e| SttError::Inference(format!("Failed to read tokenizer.json: {}", e)))?;
        let parsed: serde_json::Value = serde_json::from_str(&content)
            .map_err(|e| SttError::Inference(format!("Failed to parse tokenizer.json: {}", e)))?;

        if let Some(vocab_obj) = parsed
            .get("model")
            .and_then(|m| m.get("vocab"))
            .and_then(|v| v.as_object())
        {
            let max_id = vocab_obj
                .values()
                .filter_map(|v| v.as_u64())
                .max()
                .unwrap_or(0) as usize;
            let mut vocab = vec![String::new(); max_id + 1];
            for (token, id) in vocab_obj {
                if let Some(id) = id.as_u64() {
                    vocab[id as usize] = token.clone();
                }
            }
            return Ok(vocab);
        }
    }

    Err(SttError::Inference(format!(
        "No vocabulary file found in {:?}",
        model_dir
    )))
}

/// logits 슬라이스에서 argmax 토큰 추출
fn argmax_from_logits(data: &[f32]) -> i64 {
    if data.is_empty() {
        return EOS_TOKEN;
    }
    let (max_idx, _) = data
        .iter()
        .enumerate()
        .max_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
        .unwrap_or((0, &0.0));
    max_idx as i64
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
        if token.starts_with("<|") && token.ends_with("|>") {
            continue;
        }
        if token == "<unk>" || token == "<pad>" || token == "<s>" || token == "</s>" {
            continue;
        }
        result.push_str(token);
    }
    // Qwen 토크나이저의 ▁(U+2581) → 공백 변환
    result.replace('\u{2581}', " ").trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_tokens_basic() {
        let vocab = vec![
            "<|endoftext|>".to_string(),
            "Hello".to_string(),
            "\u{2581}world".to_string(),
            "<|im_end|>".to_string(),
        ];
        let tokens = vec![1i64, 2, 3];
        let text = decode_tokens(&tokens, &vocab);
        assert_eq!(text, "Hello world");
    }

    #[test]
    fn test_decode_tokens_special_only() {
        let vocab = vec![
            "<|endoftext|>".to_string(),
            "<|im_start|>".to_string(),
        ];
        let tokens = vec![0i64, 1];
        let text = decode_tokens(&tokens, &vocab);
        assert_eq!(text, "");
    }
}
