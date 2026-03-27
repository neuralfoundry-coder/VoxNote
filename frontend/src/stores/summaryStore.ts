import { create } from "zustand";
import { tauriInvoke } from "../hooks/useTauriIPC";

interface SummaryResponse {
  summary: string;
  template_id: string;
  model_used: string;
}

interface SummaryState {
  currentSummary: string | null;
  isGenerating: boolean;
  templateId: string;

  generateSummary: (noteId: string, templateId?: string) => Promise<void>;
  setTemplateId: (id: string) => void;
  clear: () => void;
}

export const useSummaryStore = create<SummaryState>((set) => ({
  currentSummary: null,
  isGenerating: false,
  templateId: "meeting-notes",

  generateSummary: async (noteId, templateId) => {
    set({ isGenerating: true });
    try {
      const response = await tauriInvoke<SummaryResponse>("generate_summary", {
        request: { note_id: noteId, template_id: templateId },
      });
      set({ currentSummary: response.summary, isGenerating: false });
    } catch {
      set({ isGenerating: false });
    }
  },

  setTemplateId: (id) => set({ templateId: id }),
  clear: () => set({ currentSummary: null }),
}));
