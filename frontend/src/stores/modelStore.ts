import { create } from "zustand";
import { tauriInvoke } from "../hooks/useTauriIPC";
import type { ModelInfo, ModelTestResult } from "../lib/types";

export interface FeatureModelStatus {
  hasDownloaded: boolean;
  hasActive: boolean;
  activeModel: ModelInfo | null;
  downloadedModels: ModelInfo[];
  allModels: ModelInfo[];
}

interface ModelState {
  models: ModelInfo[];
  isLoading: boolean;

  fetchModels: () => Promise<void>;
  getStatusByFeature: (feature: string) => FeatureModelStatus;
  testModel: (modelId: string) => Promise<ModelTestResult>;
}

export const useModelStore = create<ModelState>((set, get) => ({
  models: [],
  isLoading: false,

  fetchModels: async () => {
    set({ isLoading: true });
    try {
      const models = await tauriInvoke<ModelInfo[]>("list_models");
      set({ models });
    } catch (err) {
      console.error("Failed to fetch models:", err);
    } finally {
      set({ isLoading: false });
    }
  },

  getStatusByFeature: (feature: string): FeatureModelStatus => {
    const all = get().models.filter((m) => m.model_type === feature);
    const downloaded = all.filter((m) => m.is_downloaded);
    const active = all.find((m) => m.is_active) ?? null;
    return {
      hasDownloaded: downloaded.length > 0,
      hasActive: active !== null,
      activeModel: active,
      downloadedModels: downloaded,
      allModels: all,
    };
  },

  testModel: async (modelId: string): Promise<ModelTestResult> => {
    return tauriInvoke<ModelTestResult>("test_model", { modelId });
  },
}));

/** STT 모델이 하나도 다운로드되지 않았는지 (핵심 기능 불가 상태) */
export function useNeedsSetup(): boolean {
  const models = useModelStore((s) => s.models);
  const isLoading = useModelStore((s) => s.isLoading);
  if (isLoading || models.length === 0) return false;
  return !models.some((m) => m.model_type === "stt" && m.is_downloaded);
}
