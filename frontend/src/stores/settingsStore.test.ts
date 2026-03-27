import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

import { invoke } from "@tauri-apps/api/core";
import { useSettingsStore } from "./settingsStore";

const mockedInvoke = vi.mocked(invoke);

describe("settingsStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useSettingsStore.setState({ config: null, isLoading: false });
  });

  it("should fetch settings from backend", async () => {
    const mockConfig = {
      audio: { input_device: null, sample_rate: 48000, vad_threshold: 0.5, window_size_secs: 3.0, overlap_secs: 0.5 },
      stt: { model_id: "whisper-tiny", language: "ko", use_gpu: true, translate: false },
      storage: { data_dir: null, encryption_enabled: true },
      model: { models_dir: null, max_cache_mb: 10240 },
    };
    mockedInvoke.mockResolvedValueOnce(mockConfig);

    await useSettingsStore.getState().fetchSettings();

    expect(mockedInvoke).toHaveBeenCalledWith("get_settings", undefined);
    expect(useSettingsStore.getState().config).toEqual(mockConfig);
    expect(useSettingsStore.getState().isLoading).toBe(false);
  });

  it("should fallback to defaults on error", async () => {
    mockedInvoke.mockRejectedValueOnce("Backend unavailable");

    await useSettingsStore.getState().fetchSettings();

    const config = useSettingsStore.getState().config;
    expect(config).not.toBeNull();
    expect(config!.audio.sample_rate).toBe(48000);
  });

  it("should update settings", async () => {
    const config = {
      audio: { input_device: null, sample_rate: 48000, vad_threshold: 0.7, window_size_secs: 2.0, overlap_secs: 0.5 },
      stt: { model_id: null, language: null, use_gpu: false, translate: false },
      storage: { data_dir: null, encryption_enabled: false },
      model: { models_dir: null, max_cache_mb: 5120 },
    };
    mockedInvoke.mockResolvedValueOnce(undefined);

    await useSettingsStore.getState().updateSettings(config);

    expect(mockedInvoke).toHaveBeenCalledWith("update_settings", { config });
    expect(useSettingsStore.getState().config).toEqual(config);
  });
});
