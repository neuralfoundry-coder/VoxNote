use std::path::PathBuf;

use crate::error::ModelError;
use super::registry::ModelType;

/// 모델 로더 — 모델 파일을 엔진에 로드
///
/// Phase 1: Whisper 모델만 지원
/// Phase 2: LLM, TTS, Diarization 모델 추가
pub struct ModelLoader {
    models_dir: PathBuf,
}

impl ModelLoader {
    pub fn new(models_dir: PathBuf) -> Self {
        Self { models_dir }
    }

    /// 모델 파일 경로 반환 (존재 여부 확인)
    pub fn resolve_model_path(&self, model_id: &str) -> Result<PathBuf, ModelError> {
        let path = self.models_dir.join(model_id);
        if !path.exists() {
            return Err(ModelError::NotFound(model_id.to_string()));
        }
        Ok(path)
    }

    /// 모델 타입에 따른 로드 가능 여부 확인
    pub fn can_load(&self, model_type: &ModelType) -> bool {
        match model_type {
            ModelType::Stt => cfg!(feature = "stt"),
            ModelType::Llm => cfg!(feature = "llm"),
            ModelType::Tts => cfg!(feature = "tts"),
            ModelType::Diarization => cfg!(feature = "diarize"),
            ModelType::Embedding => true,
        }
    }
}
