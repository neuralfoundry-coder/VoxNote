import { useSummaryStore } from "../../stores/summaryStore";
import { useNoteStore } from "../../stores/noteStore";

const QUICK_PROMPTS = [
  "Tell me the key points",
  "List action items",
  "What were the decisions?",
];

const TEMPLATES = [
  { id: "meeting-notes", name: "Meeting Notes" },
  { id: "brainstorming", name: "Brainstorming" },
  { id: "lecture-notes", name: "Lecture Notes" },
  { id: "one-on-one", name: "1:1 Meeting" },
];

export function SummaryPanel() {
  const activeNote = useNoteStore((s) => s.activeNote);
  const { currentSummary, isGenerating, templateId, generateSummary, setTemplateId } =
    useSummaryStore();

  if (!activeNote) return null;

  return (
    <div className="flex flex-col h-full">
      <div className="px-4 pt-4 pb-3">
        <h3 className="text-[11px] font-semibold uppercase tracking-wider mb-3"
            style={{ color: "var(--vn-text-tertiary)" }}>
          AI Assistant
        </h3>
        <div className="space-y-1.5">
          {QUICK_PROMPTS.map((prompt) => (
            <button
              key={prompt}
              onClick={() => generateSummary(activeNote.id, templateId)}
              className="w-full text-left px-3 py-2 rounded-xl text-xs font-medium transition-glass hover:scale-[1.01]"
              style={{ background: "var(--vn-bg-glass)", border: "1px solid var(--vn-border)", color: "var(--vn-text-secondary)" }}
            >
              {prompt}
            </button>
          ))}
        </div>
      </div>

      <div className="mx-4 h-px" style={{ background: "var(--vn-border)" }} />

      <div className="px-4 py-3">
        <p className="text-[10px] font-semibold uppercase tracking-wider mb-2"
           style={{ color: "var(--vn-text-tertiary)" }}>Template</p>
        <div className="grid grid-cols-2 gap-1.5">
          {TEMPLATES.map((t) => (
            <button key={t.id} onClick={() => setTemplateId(t.id)}
              className="px-2 py-1.5 rounded-lg text-[10px] font-medium transition-glass"
              style={{
                background: templateId === t.id ? "rgba(99,102,241,0.1)" : "var(--vn-bg-glass)",
                border: templateId === t.id ? "1px solid rgba(99,102,241,0.3)" : "1px solid var(--vn-border)",
                color: templateId === t.id ? "var(--vn-primary)" : "var(--vn-text-tertiary)",
              }}>
              {t.name}
            </button>
          ))}
        </div>
      </div>

      <div className="px-4 pb-3">
        <button onClick={() => generateSummary(activeNote.id, templateId)} disabled={isGenerating}
          className="w-full py-2.5 rounded-xl text-xs font-semibold text-white flex items-center justify-center gap-2 transition-glass hover:scale-[1.02] disabled:opacity-50"
          style={{ background: "linear-gradient(135deg, var(--vn-primary), var(--vn-accent))", boxShadow: "var(--vn-shadow-glow)" }}>
          {isGenerating ? "Generating..." : "Generate Document"}
        </button>
      </div>

      {currentSummary && (
        <>
          <div className="mx-4 h-px" style={{ background: "var(--vn-border)" }} />
          <div className="flex-1 overflow-auto px-4 py-3 animate-float-in">
            <div className="text-[13px] leading-relaxed whitespace-pre-wrap"
                 style={{ color: "var(--vn-text-primary)" }}>
              {currentSummary}
            </div>
          </div>
        </>
      )}
    </div>
  );
}
