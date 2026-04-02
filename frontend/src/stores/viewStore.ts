import { create } from "zustand";

export type ViewId = "notes" | "settings" | "models" | "providers" | "account" | "ask";
export type ModalId = "export" | "device-pairing" | "welcome" | null;

interface ViewState {
  activeView: ViewId;
  previousView: ViewId | null;
  activeModal: ModalId;
  modalProps: Record<string, unknown>;

  setView: (view: ViewId) => void;
  goBack: () => void;
  openModal: (modal: Exclude<ModalId, null>, props?: Record<string, unknown>) => void;
  closeModal: () => void;
}

export const useViewStore = create<ViewState>((set) => ({
  activeView: "notes",
  previousView: null,
  activeModal: null,
  modalProps: {},

  setView: (view) =>
    set((s) => ({
      activeView: view,
      previousView: s.activeView !== view ? s.activeView : s.previousView,
    })),

  goBack: () =>
    set((s) => ({
      activeView: s.previousView ?? "notes",
      previousView: null,
    })),

  openModal: (modal, props = {}) =>
    set({ activeModal: modal, modalProps: props }),

  closeModal: () =>
    set({ activeModal: null, modalProps: {} }),
}));
