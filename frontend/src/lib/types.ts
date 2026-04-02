/** Rust 타입과 매칭되는 TypeScript 인터페이스 */

export type NoteStatus =
  | "recording"
  | "transcribing"
  | "summarizing"
  | "done"
  | "error";

export interface Note {
  id: string;
  title: string;
  status: NoteStatus;
  folder_id: string | null;
  duration_ms: number | null;
  language: string | null;
  created_at: string;
  updated_at: string;
}

export interface Segment {
  id: string;
  note_id: string;
  text: string;
  start_ms: number;
  end_ms: number;
  speaker_id: string | null;
  confidence: number | null;
}

export interface Folder {
  id: string;
  name: string;
  parent_id: string | null;
  created_at: string;
  updated_at: string;
}

export interface SearchResult {
  segment_id: string;
  note_id: string;
  text: string;
  highlight: string;
  rank: number;
}

export interface ModelInfo {
  id: string;
  name: string;
  model_type: string;
  size_display: string;
  is_downloaded: boolean;
  is_active: boolean;
  gpu_recommended: boolean;
  description: string | null;
}

export interface DownloadProgress {
  model_id: string;
  downloaded_bytes: number;
  total_bytes: number;
  percentage: number;
}

export interface ModelTestResult {
  success: boolean;
  output: string;
  duration_ms: number;
}

export interface RecordingResponse {
  session_id: string;
  note_id: string;
  state: string;
}

export type RecordingState = "idle" | "recording" | "paused" | "stopped";

export interface AppConfig {
  audio: {
    input_device: string | null;
    sample_rate: number;
    vad_threshold: number;
    window_size_secs: number;
    overlap_secs: number;
  };
  stt: {
    model_id: string | null;
    provider: string | null;
    language: string | null;
    use_gpu: boolean;
    translate: boolean;
  };
  storage: {
    data_dir: string | null;
    encryption_enabled: boolean;
  };
  model: {
    models_dir: string | null;
    max_cache_mb: number;
  };
}
