use serde::{Deserialize, Serialize};

use crate::error::ModelError;

/// 모델 타입
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ModelType {
    Stt,
    Llm,
    Tts,
    Diarization,
    Embedding,
}

/// 모델 엔트리 (registry.toml 파싱 결과)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelEntry {
    pub id: String,
    pub name: String,
    #[serde(rename = "type")]
    pub model_type: ModelType,
    pub size_bytes: u64,
    pub quantization: Option<String>,
    pub languages: Vec<String>,
    pub min_ram_mb: u32,
    pub gpu_recommended: bool,
    pub download_url: String,
    pub sha256: String,
    pub description: Option<String>,
}

/// 모델 레지스트리 (registry.toml 전체)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelRegistry {
    #[serde(rename = "model")]
    pub models: Vec<ModelEntry>,
}

impl ModelRegistry {
    /// registry.toml 파일에서 로드
    pub fn load(path: &std::path::Path) -> Result<Self, ModelError> {
        let content = std::fs::read_to_string(path)
            .map_err(|e| ModelError::Registry(format!("Failed to read registry: {}", e)))?;
        Self::parse(&content)
    }

    /// TOML 문자열에서 파싱
    pub fn parse(content: &str) -> Result<Self, ModelError> {
        toml::from_str(content)
            .map_err(|e| ModelError::Registry(format!("Failed to parse registry: {}", e)))
    }

    /// 타입별 모델 목록
    pub fn models_by_type(&self, model_type: &ModelType) -> Vec<&ModelEntry> {
        self.models
            .iter()
            .filter(|m| &m.model_type == model_type)
            .collect()
    }

    /// ID로 모델 검색
    pub fn get_model(&self, id: &str) -> Option<&ModelEntry> {
        self.models.iter().find(|m| m.id == id)
    }

    /// 사용 가능 RAM으로 필터링
    pub fn models_for_ram(&self, available_ram_mb: u32) -> Vec<&ModelEntry> {
        self.models
            .iter()
            .filter(|m| m.min_ram_mb <= available_ram_mb)
            .collect()
    }
}

impl ModelEntry {
    /// 사람이 읽기 좋은 크기 표시
    pub fn size_display(&self) -> String {
        let mb = self.size_bytes as f64 / 1_048_576.0;
        if mb >= 1024.0 {
            format!("{:.1} GB", mb / 1024.0)
        } else {
            format!("{:.0} MB", mb)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_registry() {
        let toml = r#"
[[model]]
id = "whisper-tiny-q8"
name = "Whisper Tiny"
type = "stt"
size_bytes = 78000000
quantization = "Q8_0"
languages = ["auto", "ko", "en", "ja"]
min_ram_mb = 512
gpu_recommended = false
download_url = "https://cdn.example.com/whisper-tiny-q8.bin"
sha256 = "abc123"
description = "Smallest Whisper model"

[[model]]
id = "whisper-large-v3-turbo"
name = "Whisper Large V3 Turbo"
type = "stt"
size_bytes = 891289600
quantization = "Q5_0"
languages = ["auto", "ko", "en", "ja", "zh", "es", "fr", "de"]
min_ram_mb = 3072
gpu_recommended = true
download_url = "https://cdn.example.com/whisper-large-v3-turbo.bin"
sha256 = "def456"
"#;
        let registry = ModelRegistry::parse(toml).unwrap();
        assert_eq!(registry.models.len(), 2);

        let stt_models = registry.models_by_type(&ModelType::Stt);
        assert_eq!(stt_models.len(), 2);

        let tiny = registry.get_model("whisper-tiny-q8").unwrap();
        assert_eq!(tiny.name, "Whisper Tiny");
        assert_eq!(tiny.size_display(), "74 MB");
        assert!(!tiny.gpu_recommended);

        let for_2gb = registry.models_for_ram(2048);
        assert_eq!(for_2gb.len(), 1); // tiny만
    }
}
