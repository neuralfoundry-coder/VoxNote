import { describe, it, expect, vi, beforeEach } from "vitest";

vi.mock("@tauri-apps/api/core", () => ({
  invoke: vi.fn(),
}));
vi.mock("@tauri-apps/api/event", () => ({
  listen: vi.fn(() => Promise.resolve(() => {})),
}));

import { invoke } from "@tauri-apps/api/core";
import { useSummaryStore } from "./summaryStore";

const mockedInvoke = vi.mocked(invoke);

describe("summaryStore", () => {
  beforeEach(() => {
    vi.clearAllMocks();
    useSummaryStore.getState().clear();
  });

  it("should initialize with null summary", () => {
    expect(useSummaryStore.getState().currentSummary).toBeNull();
    expect(useSummaryStore.getState().isGenerating).toBe(false);
    expect(useSummaryStore.getState().templateId).toBe("meeting-notes");
  });

  it("should generate summary", async () => {
    mockedInvoke.mockResolvedValueOnce({
      summary: "## Meeting Summary\n- Discussed project timeline",
      template_id: "meeting-notes",
      model_used: "qwen-7b",
    });

    await useSummaryStore.getState().generateSummary("note-123", "meeting-notes");

    expect(useSummaryStore.getState().currentSummary).toContain("Meeting Summary");
    expect(useSummaryStore.getState().isGenerating).toBe(false);
  });

  it("should set template", () => {
    useSummaryStore.getState().setTemplateId("brainstorming");
    expect(useSummaryStore.getState().templateId).toBe("brainstorming");
  });

  it("should clear summary", () => {
    useSummaryStore.setState({ currentSummary: "some summary" });
    useSummaryStore.getState().clear();
    expect(useSummaryStore.getState().currentSummary).toBeNull();
  });
});
