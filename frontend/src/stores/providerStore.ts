import { create } from "zustand";
import { tauriInvoke } from "../hooks/useTauriIPC";

interface ProviderConfig {
  engine_type: string;
  provider: string;
  model_id: string | null;
  endpoint: string | null;
  is_active: boolean;
}

interface ProviderState {
  configs: ProviderConfig[];
  availableProviders: string[];
  isLoading: boolean;

  fetchConfigs: () => Promise<void>;
  fetchAvailableProviders: () => Promise<void>;
  setConfig: (config: ProviderConfig) => Promise<void>;
  testProvider: (provider: string) => Promise<string>;
}

export const useProviderStore = create<ProviderState>((set) => ({
  configs: [],
  availableProviders: [],
  isLoading: false,

  fetchConfigs: async () => {
    set({ isLoading: true });
    try {
      const configs = await tauriInvoke<ProviderConfig[]>("get_provider_config");
      set({ configs, isLoading: false });
    } catch {
      set({ isLoading: false });
    }
  },

  fetchAvailableProviders: async () => {
    const providers = await tauriInvoke<string[]>("list_available_providers");
    set({ availableProviders: providers });
  },

  setConfig: async (config) => {
    await tauriInvoke<void>("set_provider_config", { config });
    set((state) => ({
      configs: state.configs.map((c) =>
        c.engine_type === config.engine_type ? config : c
      ),
    }));
  },

  testProvider: async (provider) => {
    return tauriInvoke<string>("test_provider", { provider });
  },
}));
