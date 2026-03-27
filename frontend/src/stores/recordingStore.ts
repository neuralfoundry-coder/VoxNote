import { create } from "zustand";
import { tauriInvoke } from "../hooks/useTauriIPC";
import type { RecordingResponse, RecordingState, Segment } from "../lib/types";

interface RecordingStoreState {
  state: RecordingState;
  sessionId: string | null;
  noteId: string | null;
  elapsedMs: number;
  segments: Segment[];

  startRecording: () => Promise<void>;
  stopRecording: () => Promise<void>;
  pauseRecording: () => Promise<void>;
  addSegment: (segment: Segment) => void;
  reset: () => void;
}

export const useRecordingStore = create<RecordingStoreState>((set) => ({
  state: "idle",
  sessionId: null,
  noteId: null,
  elapsedMs: 0,
  segments: [],

  startRecording: async () => {
    const response = await tauriInvoke<RecordingResponse>("start_recording");
    set({
      state: "recording",
      sessionId: response.session_id,
      noteId: response.note_id,
      elapsedMs: 0,
      segments: [],
    });
  },

  stopRecording: async () => {
    await tauriInvoke<string>("stop_recording");
    set({ state: "stopped" });
  },

  pauseRecording: async () => {
    const result = await tauriInvoke<string>("pause_recording");
    set({ state: result === "paused" ? "paused" : "recording" });
  },

  addSegment: (segment) => {
    set((state) => ({
      segments: [...state.segments, segment],
    }));
  },

  reset: () => {
    set({
      state: "idle",
      sessionId: null,
      noteId: null,
      elapsedMs: 0,
      segments: [],
    });
  },
}));
