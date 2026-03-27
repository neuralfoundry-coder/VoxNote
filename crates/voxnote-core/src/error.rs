use thiserror::Error;

/// VoxNote 통합 에러 타입
#[derive(Error, Debug)]
pub enum VoxNoteError {
    // 오디오 파이프라인
    #[error("Audio error: {0}")]
    Audio(#[from] AudioError),

    // STT 엔진
    #[error("STT error: {0}")]
    Stt(#[from] SttError),

    // LLM 엔진
    #[error("LLM error: {0}")]
    Llm(#[from] LlmError),

    // 저장소
    #[error("Storage error: {0}")]
    Storage(#[from] StorageError),

    // 모델 관리
    #[error("Model error: {0}")]
    Model(#[from] ModelError),

    // 암호화
    #[error("Crypto error: {0}")]
    Crypto(#[from] CryptoError),

    // 설정
    #[error("Config error: {0}")]
    Config(String),
}

#[derive(Error, Debug)]
pub enum AudioError {
    #[error("No audio device found")]
    NoDevice,

    #[error("Device not available: {0}")]
    DeviceNotAvailable(String),

    #[error("Stream error: {0}")]
    Stream(String),

    #[error("Resample error: {0}")]
    Resample(String),

    #[error("VAD error: {0}")]
    Vad(String),

    #[error("Buffer overflow: dropped {0} samples")]
    BufferOverflow(usize),
}

#[derive(Error, Debug)]
pub enum SttError {
    #[error("Model not loaded")]
    ModelNotLoaded,

    #[error("Inference failed: {0}")]
    Inference(String),

    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),

    #[error("Provider error: {0}")]
    Provider(String),
}

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("Model not loaded")]
    ModelNotLoaded,

    #[error("Inference failed: {0}")]
    Inference(String),

    #[error("Context overflow: {used} tokens exceeds {max} limit")]
    ContextOverflow { used: usize, max: usize },

    #[error("Provider error: {0}")]
    Provider(String),
}

#[derive(Error, Debug)]
pub enum StorageError {
    #[error("SQLite error: {0}")]
    Sqlite(#[from] rusqlite::Error),

    #[error("Migration failed: {0}")]
    Migration(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Serialization error: {0}")]
    Serialization(String),
}

#[derive(Error, Debug)]
pub enum ModelError {
    #[error("Model not found: {0}")]
    NotFound(String),

    #[error("Download failed: {0}")]
    Download(String),

    #[error("Integrity check failed: expected {expected}, got {actual}")]
    IntegrityCheck { expected: String, actual: String },

    #[error("Insufficient disk space: need {need_mb}MB, available {available_mb}MB")]
    InsufficientDisk { need_mb: u64, available_mb: u64 },

    #[error("Registry parse error: {0}")]
    Registry(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

#[derive(Error, Debug)]
pub enum CryptoError {
    #[error("Key derivation failed: {0}")]
    KeyDerivation(String),

    #[error("Encryption failed: {0}")]
    Encryption(String),

    #[error("Decryption failed: {0}")]
    Decryption(String),

    #[error("Invalid key")]
    InvalidKey,
}

pub type Result<T> = std::result::Result<T, VoxNoteError>;
