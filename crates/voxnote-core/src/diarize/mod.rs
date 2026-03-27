use async_trait::async_trait;

use crate::error::VoxNoteError;

pub mod onnx;

/// 화자 분리 결과
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpeakerSegment {
    pub speaker_id: String,
    pub start_ms: i64,
    pub end_ms: i64,
    pub confidence: f32,
}

/// 화자 프로필
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SpeakerProfile {
    pub id: String,
    pub name: Option<String>,
    /// 192차원 ECAPA-TDNN 임베딩
    pub embedding: Vec<f32>,
}

/// Speaker Diarizer 트레이트 — 확장 포인트
///
/// 오디오에서 화자를 구분하고 임베딩을 추출합니다.
#[async_trait]
pub trait SpeakerDiarizer: Send + Sync {
    /// 오디오 프레임(16kHz mono)에서 화자 세그먼트 추출
    async fn diarize(&self, samples: &[f32]) -> Result<Vec<SpeakerSegment>, VoxNoteError>;

    /// 오디오 프레임에서 화자 임베딩 추출
    async fn extract_embedding(&self, samples: &[f32]) -> Result<Vec<f32>, VoxNoteError>;

    /// Provider 이름
    fn name(&self) -> &str;
}
