use async_trait::async_trait;
use reqwest::Client;
use std::sync::Mutex;
use tracing::debug;

use super::{Language, SttProvider};
use crate::audio::AudioChunk;
use crate::error::SttError;
use crate::models::Segment;

/// OpenAI Whisper API 기반 클라우드 STT
pub struct OpenAiSttProvider {
    client: Client,
    api_key: String,
    model: String,
    languages: Vec<Language>,
    language: Mutex<Option<String>>,
}

impl OpenAiSttProvider {
    pub fn new(api_key: String) -> Self {
        Self {
            client: Client::new(),
            api_key,
            model: "whisper-1".to_string(),
            languages: vec![
                Language::auto(),
                Language::new("ko", "Korean"),
                Language::new("en", "English"),
                Language::new("ja", "Japanese"),
            ],
            language: Mutex::new(None),
        }
    }
}

#[async_trait]
impl SttProvider for OpenAiSttProvider {
    async fn transcribe(
        &self,
        audio: &AudioChunk,
        note_id: &str,
    ) -> Result<Vec<Segment>, SttError> {
        // PCM → WAV 변환
        let wav_data = pcm_to_wav(&audio.samples, audio.sample_rate);

        let mut form = reqwest::multipart::Form::new()
            .text("model", self.model.clone())
            .text("response_format", "verbose_json")
            .part(
                "file",
                reqwest::multipart::Part::bytes(wav_data)
                    .file_name("audio.wav")
                    .mime_str("audio/wav")
                    .map_err(|e| SttError::Provider(e.to_string()))?,
            );

        let language = self.language.lock().unwrap().clone();
        if let Some(ref lang) = language {
            if lang != "auto" {
                form = form.text("language", lang.clone());
            }
        }

        let resp = self
            .client
            .post("https://api.openai.com/v1/audio/transcriptions")
            .bearer_auth(&self.api_key)
            .multipart(form)
            .send()
            .await
            .map_err(|e| SttError::Provider(e.to_string()))?;

        if !resp.status().is_success() {
            let body = resp.text().await.unwrap_or_default();
            return Err(SttError::Provider(format!("OpenAI STT error: {}", body)));
        }

        let json: serde_json::Value = resp
            .json()
            .await
            .map_err(|e| SttError::Provider(e.to_string()))?;

        let mut segments = Vec::new();
        if let Some(segs) = json["segments"].as_array() {
            for seg in segs {
                let text = seg["text"].as_str().unwrap_or("").trim();
                if text.is_empty() {
                    continue;
                }
                let start = seg["start"].as_f64().unwrap_or(0.0);
                let end = seg["end"].as_f64().unwrap_or(0.0);

                segments.push(Segment::new(
                    note_id,
                    text,
                    audio.timestamp_ms + (start * 1000.0) as i64,
                    audio.timestamp_ms + (end * 1000.0) as i64,
                ));
            }
        } else if let Some(text) = json["text"].as_str() {
            if !text.trim().is_empty() {
                segments.push(Segment::new(
                    note_id,
                    text.trim(),
                    audio.timestamp_ms,
                    audio.timestamp_ms + (audio.duration_secs() * 1000.0) as i64,
                ));
            }
        }

        debug!("OpenAI STT: {} segments", segments.len());
        Ok(segments)
    }

    fn supported_languages(&self) -> &[Language] {
        &self.languages
    }

    fn set_initial_prompt(&self, _prompt: &str) {
        // OpenAI Whisper API는 prompt 파라미터 지원
    }

    fn set_language(&self, language: Option<&str>) {
        *self.language.lock().unwrap() = language.map(String::from);
    }

    fn name(&self) -> &str {
        "openai-stt"
    }
}

/// PCM f32 → WAV 바이트 변환
fn pcm_to_wav(samples: &[f32], sample_rate: u32) -> Vec<u8> {
    let num_samples = samples.len();
    let byte_rate = sample_rate * 2; // 16-bit mono
    let data_size = num_samples * 2;
    let file_size = 36 + data_size;

    let mut wav = Vec::with_capacity(44 + data_size);

    // RIFF header
    wav.extend_from_slice(b"RIFF");
    wav.extend_from_slice(&(file_size as u32).to_le_bytes());
    wav.extend_from_slice(b"WAVE");

    // fmt chunk
    wav.extend_from_slice(b"fmt ");
    wav.extend_from_slice(&16u32.to_le_bytes()); // chunk size
    wav.extend_from_slice(&1u16.to_le_bytes()); // PCM
    wav.extend_from_slice(&1u16.to_le_bytes()); // mono
    wav.extend_from_slice(&sample_rate.to_le_bytes());
    wav.extend_from_slice(&byte_rate.to_le_bytes());
    wav.extend_from_slice(&2u16.to_le_bytes()); // block align
    wav.extend_from_slice(&16u16.to_le_bytes()); // bits per sample

    // data chunk
    wav.extend_from_slice(b"data");
    wav.extend_from_slice(&(data_size as u32).to_le_bytes());

    // f32 → i16 변환
    for &sample in samples {
        let clamped = sample.clamp(-1.0, 1.0);
        let i16_val = (clamped * 32767.0) as i16;
        wav.extend_from_slice(&i16_val.to_le_bytes());
    }

    wav
}
