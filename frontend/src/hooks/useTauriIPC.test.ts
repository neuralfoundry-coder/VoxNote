import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

import { invoke } from "@tauri-apps/api/core";
import { tauriInvoke } from "./useTauriIPC";

const mockedInvoke = vi.mocked(invoke);

describe("tauriInvoke", () => {
  beforeEach(() => {
    vi.clearAllMocks();
  });

  it("should call invoke with command name", async () => {
    mockedInvoke.mockResolvedValueOnce({ id: "1", title: "Test" });

    const result = await tauriInvoke<{ id: string; title: string }>("get_note", { id: "1" });

    expect(mockedInvoke).toHaveBeenCalledWith("get_note", { id: "1" });
    expect(result.id).toBe("1");
    expect(result.title).toBe("Test");
  });

  it("should call invoke without args", async () => {
    mockedInvoke.mockResolvedValueOnce([]);

    await tauriInvoke<unknown[]>("list_notes");
    expect(mockedInvoke).toHaveBeenCalledWith("list_notes", undefined);
  });

  it("should propagate errors", async () => {
    mockedInvoke.mockRejectedValueOnce("Not found");

    await expect(tauriInvoke("get_note", { id: "bad" })).rejects.toBe("Not found");
  });

  it("should handle typed responses", async () => {
    mockedInvoke.mockResolvedValueOnce({
      session_id: "s1",
      note_id: "n1",
      state: "recording",
    });

    const result = await tauriInvoke<{ session_id: string }>("start_recording");
    expect(result.session_id).toBe("s1");
  });
});

describe("useTauriEvent", () => {
  it("should be importable", async () => {
    const { useTauriEvent } = await import("./useTauriIPC");
    expect(useTauriEvent).toBeDefined();
  });
});
