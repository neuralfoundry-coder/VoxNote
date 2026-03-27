use async_trait::async_trait;

use crate::error::VoxNoteError;

#[cfg(feature = "tts")]
pub mod onnx;

#[cfg(feature = "cloud-providers")]
pub mod cloud;

/// TTS Provider 트레이트 — 확장 포인트
///
/// 로컬(Piper ONNX)과 클라우드(OpenAI TTS 등)
/// 구현을 동일한 인터페이스로 통합합니다.
#[async_trait]
pub trait TtsProvider: Send + Sync {
    /// 텍스트를 음성으로 합성 (PCM f32, 지정 샘플레이트)
    async fn synthesize(&self, text: &str, language: &str)
        -> Result<TtsOutput, VoxNoteError>;

    /// 지원 언어 목록
    fn supported_languages(&self) -> &[String];

    /// Provider 이름
    fn name(&self) -> &str;
}

/// TTS 출력
#[derive(Debug, Clone)]
pub struct TtsOutput {
    pub samples: Vec<f32>,
    pub sample_rate: u32,
}
