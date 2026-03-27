import { describe, it, expect, vi, beforeEach } from "vitest";

// Tauri IPC mock
vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

import { invoke } from "@tauri-apps/api/core";
import { useNoteStore } from "./noteStore";

const mockedInvoke = vi.mocked(invoke);

describe("noteStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useNoteStore.setState({
      notes: [],
      activeNote: null,
      searchResults: [],
      isLoading: false,
    });
  });

  it("should initialize with empty state", () => {
    const state = useNoteStore.getState();
    expect(state.notes).toEqual([]);
    expect(state.activeNote).toBeNull();
    expect(state.isLoading).toBe(false);
  });

  it("should fetch notes", async () => {
    const mockNotes = [
      { id: "1", title: "Note 1", status: "done", folder_id: null, duration_ms: null, language: null, created_at: "2024-01-01", updated_at: "2024-01-01" },
      { id: "2", title: "Note 2", status: "recording", folder_id: null, duration_ms: null, language: null, created_at: "2024-01-02", updated_at: "2024-01-02" },
    ];
    mockedInvoke.mockResolvedValueOnce(mockNotes);

    await useNoteStore.getState().fetchNotes();

    expect(mockedInvoke).toHaveBeenCalledWith("list_notes", { folderId: null });
    expect(useNoteStore.getState().notes).toEqual(mockNotes);
    expect(useNoteStore.getState().isLoading).toBe(false);
  });

  it("should create a note", async () => {
    const newNote = { id: "3", title: "New Note", status: "recording", folder_id: null, duration_ms: null, language: null, created_at: "2024-01-03", updated_at: "2024-01-03" };
    mockedInvoke.mockResolvedValueOnce(newNote);

    const result = await useNoteStore.getState().createNote("New Note");

    expect(mockedInvoke).toHaveBeenCalledWith("create_note", { title: "New Note" });
    expect(result).toEqual(newNote);
    expect(useNoteStore.getState().notes).toContainEqual(newNote);
  });

  it("should delete a note", async () => {
    useNoteStore.setState({
      notes: [
        { id: "1", title: "Keep", status: "done" as const, folder_id: null, duration_ms: null, language: null, created_at: "2024-01-01", updated_at: "2024-01-01" },
        { id: "2", title: "Delete", status: "done" as const, folder_id: null, duration_ms: null, language: null, created_at: "2024-01-02", updated_at: "2024-01-02" },
      ],
    });
    mockedInvoke.mockResolvedValueOnce(undefined);

    await useNoteStore.getState().deleteNote("2");

    expect(useNoteStore.getState().notes).toHaveLength(1);
    expect(useNoteStore.getState().notes[0].id).toBe("1");
  });

  it("should set active note", () => {
    const note = { id: "1", title: "Active", status: "done" as const, folder_id: null, duration_ms: null, language: null, created_at: "2024-01-01", updated_at: "2024-01-01" };
    useNoteStore.getState().setActiveNote(note);
    expect(useNoteStore.getState().activeNote).toEqual(note);
  });

  it("should search notes", async () => {
    const results = [
      { segment_id: "s1", note_id: "n1", text: "meeting", highlight: "<mark>meeting</mark>", rank: 1.0 },
    ];
    mockedInvoke.mockResolvedValueOnce(results);

    await useNoteStore.getState().searchNotes("meeting");

    expect(mockedInvoke).toHaveBeenCalledWith("search_notes", { query: "meeting" });
    expect(useNoteStore.getState().searchResults).toEqual(results);
  });

  it("should clear search for empty query", async () => {
    useNoteStore.setState({ searchResults: [{ segment_id: "s1", note_id: "n1", text: "t", highlight: "h", rank: 1 }] });
    await useNoteStore.getState().searchNotes("");
    expect(useNoteStore.getState().searchResults).toEqual([]);
  });
});
