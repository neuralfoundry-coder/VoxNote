import { create } from "zustand";

type SyncStatus = "disconnected" | "connecting" | "connected" | "error";

interface PairedDevice {
  id: string;
  name: string;
  lastSeen: string;
}

interface SyncState {
  status: SyncStatus;
  pairedDevices: PairedDevice[];
  pendingDeltas: number;
  errorMessage: string | null;

  setStatus: (status: SyncStatus) => void;
  addDevice: (device: PairedDevice) => void;
  removeDevice: (id: string) => void;
  setPendingDeltas: (count: number) => void;
  setError: (message: string | null) => void;
}

export const useSyncStore = create<SyncState>((set) => ({
  status: "disconnected",
  pairedDevices: [],
  pendingDeltas: 0,
  errorMessage: null,

  setStatus: (status) => set({ status, errorMessage: null }),
  addDevice: (device) =>
    set((state) => ({
      pairedDevices: [...state.pairedDevices, device],
    })),
  removeDevice: (id) =>
    set((state) => ({
      pairedDevices: state.pairedDevices.filter((d) => d.id !== id),
    })),
  setPendingDeltas: (count) => set({ pendingDeltas: count }),
  setError: (message) => set({ errorMessage: message, status: "error" }),
}));
