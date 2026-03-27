pub mod audio;
pub mod config;
pub mod error;
pub mod models;
pub mod pipeline;
pub mod storage;
pub mod model_manager;

// STT engine
pub mod stt;

// LLM engine
pub mod llm;

// TTS engine
pub mod tts;

// Speaker diarization
pub mod diarize;

// Provider management
pub mod provider;

// Post-processing
pub mod post_processor;

// RAG (Ask VoxNote)
pub mod rag;

// Export (Markdown/PDF/DOCX)
pub mod export;

// Device sync (CRDT + E2EE)
pub mod sync;
