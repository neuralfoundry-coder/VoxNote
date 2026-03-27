import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

import { invoke } from "@tauri-apps/api/core";
import { useRecordingStore } from "./recordingStore";

const mockedInvoke = vi.mocked(invoke);

describe("recordingStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useRecordingStore.getState().reset();
  });

  it("should initialize as idle", () => {
    const state = useRecordingStore.getState();
    expect(state.state).toBe("idle");
    expect(state.sessionId).toBeNull();
    expect(state.noteId).toBeNull();
    expect(state.segments).toEqual([]);
  });

  it("should start recording", async () => {
    mockedInvoke.mockResolvedValueOnce({
      session_id: "sess-1",
      note_id: "note-1",
      state: "recording",
    });

    await useRecordingStore.getState().startRecording();

    const state = useRecordingStore.getState();
    expect(state.state).toBe("recording");
    expect(state.sessionId).toBe("sess-1");
    expect(state.noteId).toBe("note-1");
  });

  it("should stop recording", async () => {
    useRecordingStore.setState({ state: "recording" });
    mockedInvoke.mockResolvedValueOnce("stopped");

    await useRecordingStore.getState().stopRecording();
    expect(useRecordingStore.getState().state).toBe("stopped");
  });

  it("should pause and resume", async () => {
    useRecordingStore.setState({ state: "recording" });

    mockedInvoke.mockResolvedValueOnce("paused");
    await useRecordingStore.getState().pauseRecording();
    expect(useRecordingStore.getState().state).toBe("paused");

    mockedInvoke.mockResolvedValueOnce("recording");
    await useRecordingStore.getState().pauseRecording();
    expect(useRecordingStore.getState().state).toBe("recording");
  });

  it("should add segments", () => {
    const segment = {
      id: "s1", note_id: "n1", text: "Hello", start_ms: 0, end_ms: 1000,
      speaker_id: null, confidence: null,
    };
    useRecordingStore.getState().addSegment(segment);
    expect(useRecordingStore.getState().segments).toHaveLength(1);
    expect(useRecordingStore.getState().segments[0].text).toBe("Hello");
  });

  it("should reset state", () => {
    useRecordingStore.setState({
      state: "recording",
      sessionId: "s",
      noteId: "n",
      segments: [{ id: "1", note_id: "n", text: "t", start_ms: 0, end_ms: 0, speaker_id: null, confidence: null }],
    });

    useRecordingStore.getState().reset();
    const state = useRecordingStore.getState();
    expect(state.state).toBe("idle");
    expect(state.sessionId).toBeNull();
    expect(state.segments).toEqual([]);
  });
});
