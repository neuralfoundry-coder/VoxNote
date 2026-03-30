use async_trait::async_trait;

use crate::audio::AudioChunk;
use crate::error::SttError;
use crate::models::Segment;

#[cfg(feature = "stt")]
pub mod whisper;

#[cfg(feature = "cloud-providers")]
pub mod cloud;

/// 지원 언어
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Language {
    pub code: String,
    pub name: String,
}

impl Language {
    pub fn new(code: impl Into<String>, name: impl Into<String>) -> Self {
        Self {
            code: code.into(),
            name: name.into(),
        }
    }

    pub fn auto() -> Self {
        Self::new("auto", "Auto-detect")
    }
}

/// STT Provider 트레이트 — 확장 포인트
///
/// 로컬(whisper.cpp)과 클라우드(OpenAI Whisper API, Google STT 등)
/// 구현을 동일한 인터페이스로 통합합니다.
#[async_trait]
pub trait SttProvider: Send + Sync {
    /// 오디오 청크를 전사하여 세그먼트 목록 반환
    async fn transcribe(
        &self,
        audio: &AudioChunk,
        note_id: &str,
    ) -> Result<Vec<Segment>, SttError>;

    /// 지원 언어 목록
    fn supported_languages(&self) -> &[Language];

    /// 문맥 연속성을 위한 initial_prompt 설정
    fn set_initial_prompt(&self, prompt: &str);

    /// 언어 설정 (None = 자동 감지)
    fn set_language(&self, language: Option<&str>);

    /// Provider 이름
    fn name(&self) -> &str;
}
