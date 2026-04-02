import { create } from "zustand";
import { tauriInvoke } from "../hooks/useTauriIPC";
import type { AppConfig } from "../lib/types";

interface SettingsState {
  config: AppConfig | null;
  isLoading: boolean;

  fetchSettings: () => Promise<void>;
  updateSettings: (config: AppConfig) => Promise<void>;
}

const defaultConfig: AppConfig = {
  audio: {
    input_device: null,
    sample_rate: 48000,
    vad_threshold: 0.5,
    window_size_secs: 3.0,
    overlap_secs: 0.5,
  },
  stt: {
    model_id: null,
    provider: null,
    language: null,
    use_gpu: true,
    translate: false,
  },
  storage: {
    data_dir: null,
    encryption_enabled: true,
  },
  model: {
    models_dir: null,
    max_cache_mb: 10240,
  },
};

export const useSettingsStore = create<SettingsState>((set) => ({
  config: null,
  isLoading: false,

  fetchSettings: async () => {
    set({ isLoading: true });
    try {
      const config = await tauriInvoke<AppConfig>("get_settings");
      set({ config, isLoading: false });
    } catch {
      set({ config: defaultConfig, isLoading: false });
    }
  },

  updateSettings: async (config) => {
    await tauriInvoke<void>("update_settings", { config });
    set({ config });
  },
}));
