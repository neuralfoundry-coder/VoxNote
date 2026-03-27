use async_trait::async_trait;
use tracing::debug;

use super::{SpeakerDiarizer, SpeakerProfile, SpeakerSegment};
use crate::error::VoxNoteError;

/// ECAPA-TDNN 기반 화자 분리 (ONNX Runtime)
pub struct OnnxDiarizer {
    /// 코사인 유사도 임계값
    threshold: f32,
    /// 등록된 화자 프로필
    profiles: Vec<SpeakerProfile>,
    /// 현재 세션 클러스터
    clusters: Vec<SpeakerCluster>,
}

struct SpeakerCluster {
    id: String,
    centroid: Vec<f32>,
    count: usize,
}

impl OnnxDiarizer {
    pub fn new(threshold: f32) -> Self {
        Self {
            threshold: threshold.clamp(0.5, 0.95),
            profiles: Vec::new(),
            clusters: Vec::new(),
        }
    }

    pub fn set_threshold(&mut self, threshold: f32) {
        self.threshold = threshold.clamp(0.5, 0.95);
    }

    pub fn register_profile(&mut self, profile: SpeakerProfile) {
        self.profiles.push(profile);
    }

    /// 온라인 응집 클러스터링
    pub fn assign_cluster(&mut self, embedding: &[f32]) -> String {
        let mut best_sim = 0.0f32;
        let mut best_idx = None;

        // 기존 클러스터와 유사도 비교
        for (i, cluster) in self.clusters.iter().enumerate() {
            let sim = cosine_similarity(&cluster.centroid, embedding);
            if sim > best_sim {
                best_sim = sim;
                best_idx = Some(i);
            }
        }

        // 등록된 프로필과도 비교
        for profile in &self.profiles {
            let sim = cosine_similarity(&profile.embedding, embedding);
            if sim > best_sim {
                best_sim = sim;
                // 프로필 매칭 시 해당 이름 반환
                if sim >= self.threshold {
                    return profile.name.clone().unwrap_or_else(|| profile.id.clone());
                }
            }
        }

        if best_sim >= self.threshold {
            if let Some(idx) = best_idx {
                // 기존 클러스터에 병합
                let cluster = &mut self.clusters[idx];
                update_centroid(&mut cluster.centroid, embedding, cluster.count);
                cluster.count += 1;
                return cluster.id.clone();
            }
        }

        // 새 클러스터 생성
        let new_id = format!("speaker-{}", self.clusters.len() + 1);
        self.clusters.push(SpeakerCluster {
            id: new_id.clone(),
            centroid: embedding.to_vec(),
            count: 1,
        });
        new_id
    }

    pub fn reset_session(&mut self) {
        self.clusters.clear();
    }
}

#[async_trait]
impl SpeakerDiarizer for OnnxDiarizer {
    async fn diarize(&self, samples: &[f32]) -> Result<Vec<SpeakerSegment>, VoxNoteError> {
        // ONNX Runtime으로 임베딩 추출 후 클러스터링
        // 현재는 스텁 — ort crate 연동 시 실구현
        debug!("Diarize called with {} samples", samples.len());
        Ok(Vec::new())
    }

    async fn extract_embedding(&self, samples: &[f32]) -> Result<Vec<f32>, VoxNoteError> {
        // ECAPA-TDNN ONNX 모델로 192차원 임베딩 추출
        debug!("Extract embedding from {} samples", samples.len());
        Ok(vec![0.0; 192]) // placeholder
    }

    fn name(&self) -> &str {
        "ecapa-tdnn-onnx"
    }
}

pub fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    if a.len() != b.len() || a.is_empty() {
        return 0.0;
    }
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}

pub fn update_centroid(centroid: &mut [f32], new_embedding: &[f32], count: usize) {
    let w = count as f32;
    for (c, &e) in centroid.iter_mut().zip(new_embedding.iter()) {
        *c = (*c * w + e) / (w + 1.0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 1e-6);

        let c = vec![0.0, 1.0, 0.0];
        assert!((cosine_similarity(&a, &c)).abs() < 1e-6);
    }

    #[test]
    fn test_cluster_assignment() {
        let mut diarizer = OnnxDiarizer::new(0.7);

        let emb1 = vec![1.0; 192];
        let speaker1 = diarizer.assign_cluster(&emb1);

        let emb2 = vec![1.0; 192]; // same speaker
        let speaker2 = diarizer.assign_cluster(&emb2);
        assert_eq!(speaker1, speaker2);

        let mut emb3 = vec![0.0; 192]; // different speaker
        emb3[0] = 1.0;
        let speaker3 = diarizer.assign_cluster(&emb3);
        assert_ne!(speaker1, speaker3);
    }
}
