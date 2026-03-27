import { create } from "zustand";
import { tauriInvoke } from "../hooks/useTauriIPC";
import type { Note, SearchResult } from "../lib/types";

interface NoteState {
  notes: Note[];
  activeNote: Note | null;
  searchResults: SearchResult[];
  isLoading: boolean;

  fetchNotes: (folderId?: string) => Promise<void>;
  getNote: (id: string) => Promise<void>;
  createNote: (title: string) => Promise<Note>;
  updateNote: (note: Note) => Promise<void>;
  deleteNote: (id: string) => Promise<void>;
  searchNotes: (query: string) => Promise<void>;
  setActiveNote: (note: Note | null) => void;
}

export const useNoteStore = create<NoteState>((set) => ({
  notes: [],
  activeNote: null,
  searchResults: [],
  isLoading: false,

  fetchNotes: async (folderId) => {
    set({ isLoading: true });
    try {
      const notes = await tauriInvoke<Note[]>("list_notes", {
        folderId: folderId ?? null,
      });
      set({ notes, isLoading: false });
    } catch {
      set({ isLoading: false });
    }
  },

  getNote: async (id) => {
    const note = await tauriInvoke<Note | null>("get_note", { id });
    if (note) {
      set({ activeNote: note });
    }
  },

  createNote: async (title) => {
    const note = await tauriInvoke<Note>("create_note", { title });
    set((state) => ({ notes: [note, ...state.notes] }));
    return note;
  },

  updateNote: async (note) => {
    await tauriInvoke<void>("update_note", { note });
    set((state) => ({
      notes: state.notes.map((n) => (n.id === note.id ? note : n)),
      activeNote: state.activeNote?.id === note.id ? note : state.activeNote,
    }));
  },

  deleteNote: async (id) => {
    await tauriInvoke<void>("delete_note", { id });
    set((state) => ({
      notes: state.notes.filter((n) => n.id !== id),
      activeNote: state.activeNote?.id === id ? null : state.activeNote,
    }));
  },

  searchNotes: async (query) => {
    if (!query.trim()) {
      set({ searchResults: [] });
      return;
    }
    const results = await tauriInvoke<SearchResult[]>("search_notes", {
      query,
    });
    set({ searchResults: results });
  },

  setActiveNote: (note) => set({ activeNote: note }),
}));
