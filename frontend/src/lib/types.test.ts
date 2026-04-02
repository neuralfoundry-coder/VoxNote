import { describe, it, expect } from "vitest";
import type { Note, Segment, AppConfig, RecordingState } from "./types";

describe("TypeScript type contracts", () => {
  it("Note type should have required fields", () => {
    const note: Note = {
      id: "test-id",
      title: "Test Note",
      status: "done",
      folder_id: null,
      duration_ms: 60000,
      language: "ko",
      created_at: "2024-01-01T00:00:00Z",
      updated_at: "2024-01-01T00:00:00Z",
    };
    expect(note.id).toBe("test-id");
    expect(note.status).toBe("done");
  });

  it("Segment type should have required fields", () => {
    const segment: Segment = {
      id: "seg-1",
      note_id: "note-1",
      text: "Hello",
      start_ms: 0,
      end_ms: 1000,
      speaker_id: "Alice",
      confidence: 0.95,
    };
    expect(segment.start_ms).toBe(0);
    expect(segment.speaker_id).toBe("Alice");
  });

  it("RecordingState should be valid union type", () => {
    const states: RecordingState[] = ["idle", "recording", "paused", "stopped"];
    expect(states).toHaveLength(4);
  });

  it("AppConfig should have default-compatible structure", () => {
    const config: AppConfig = {
      audio: { input_device: null, sample_rate: 48000, vad_threshold: 0.5, window_size_secs: 3.0, overlap_secs: 0.5 },
      stt: { model_id: null, provider: null, language: null, use_gpu: true, translate: false },
      storage: { data_dir: null, encryption_enabled: true },
      model: { models_dir: null, max_cache_mb: 10240 },
    };
    expect(config.audio.sample_rate).toBe(48000);
    expect(config.storage.encryption_enabled).toBe(true);
  });
});
