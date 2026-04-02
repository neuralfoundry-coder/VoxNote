use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// VoxNote 앱 설정
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    pub audio: AudioConfig,
    pub stt: SttConfig,
    pub storage: StorageConfig,
    pub model: ModelConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioConfig {
    /// 입력 디바이스 이름 (None = 기본 디바이스)
    pub input_device: Option<String>,

    /// 캡처 샘플레이트 (기본: 48000)
    #[serde(default = "default_sample_rate")]
    pub sample_rate: u32,

    /// VAD 임계값 (0.0 ~ 1.0, 기본: 0.5)
    #[serde(default = "default_vad_threshold")]
    pub vad_threshold: f32,

    /// 슬라이딩 윈도우 크기 (초, 기본: 3.0)
    #[serde(default = "default_window_size")]
    pub window_size_secs: f32,

    /// 윈도우 오버랩 (초, 기본: 0.5)
    #[serde(default = "default_overlap")]
    pub overlap_secs: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SttConfig {
    /// 사용할 STT 모델 ID
    pub model_id: Option<String>,

    /// STT Provider 타입 ("whisper" | "sensevoice" | "qwen-asr")
    #[serde(default)]
    pub provider: Option<String>,

    /// 언어 설정 (None = 자동 감지)
    pub language: Option<String>,

    /// GPU 가속 사용 여부
    #[serde(default = "default_true")]
    pub use_gpu: bool,

    /// 번역 모드 (소스 → 영어)
    #[serde(default)]
    pub translate: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// 데이터 저장 경로 (기본: ~/.voxnote/data)
    pub data_dir: Option<PathBuf>,

    /// 암호화 활성화
    #[serde(default = "default_true")]
    pub encryption_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelConfig {
    /// 모델 저장 경로 (기본: ~/.voxnote/models)
    pub models_dir: Option<PathBuf>,

    /// 최대 모델 캐시 크기 (MB)
    #[serde(default = "default_max_cache_mb")]
    pub max_cache_mb: u64,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            audio: AudioConfig::default(),
            stt: SttConfig::default(),
            storage: StorageConfig::default(),
            model: ModelConfig::default(),
        }
    }
}

impl Default for AudioConfig {
    fn default() -> Self {
        Self {
            input_device: None,
            sample_rate: default_sample_rate(),
            vad_threshold: default_vad_threshold(),
            window_size_secs: default_window_size(),
            overlap_secs: default_overlap(),
        }
    }
}

impl Default for SttConfig {
    fn default() -> Self {
        Self {
            model_id: None,
            provider: None,
            language: None,
            use_gpu: true,
            translate: false,
        }
    }
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            data_dir: None,
            encryption_enabled: true,
        }
    }
}

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            models_dir: None,
            max_cache_mb: default_max_cache_mb(),
        }
    }
}

impl AppConfig {
    /// 설정 파일에서 로드 (없으면 기본값)
    pub fn load(path: &std::path::Path) -> crate::error::Result<Self> {
        if path.exists() {
            let content = std::fs::read_to_string(path)
                .map_err(|e| crate::error::VoxNoteError::Config(e.to_string()))?;
            toml::from_str(&content)
                .map_err(|e| crate::error::VoxNoteError::Config(e.to_string()))
        } else {
            Ok(Self::default())
        }
    }

    /// 설정 파일에 저장
    pub fn save(&self, path: &std::path::Path) -> crate::error::Result<()> {
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .map_err(|e| crate::error::VoxNoteError::Config(e.to_string()))?;
        }
        let content = toml::to_string_pretty(self)
            .map_err(|e| crate::error::VoxNoteError::Config(e.to_string()))?;
        std::fs::write(path, content)
            .map_err(|e| crate::error::VoxNoteError::Config(e.to_string()))
    }

    /// 데이터 디렉토리 (기본: ~/.voxnote/data)
    pub fn data_dir(&self) -> PathBuf {
        self.storage
            .data_dir
            .clone()
            .unwrap_or_else(|| default_base_dir().join("data"))
    }

    /// 모델 디렉토리 (기본: ~/.voxnote/models)
    pub fn models_dir(&self) -> PathBuf {
        self.model
            .models_dir
            .clone()
            .unwrap_or_else(|| default_base_dir().join("models"))
    }
}

fn default_base_dir() -> PathBuf {
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".voxnote")
}

fn default_sample_rate() -> u32 {
    48000
}
fn default_vad_threshold() -> f32 {
    0.5
}
fn default_window_size() -> f32 {
    3.0
}
fn default_overlap() -> f32 {
    0.5
}
fn default_true() -> bool {
    true
}
fn default_max_cache_mb() -> u64 {
    10240 // 10GB
}
