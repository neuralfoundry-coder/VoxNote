import { useState } from "react";
import { useNoteStore } from "../../stores/noteStore";
import { TranscriptView } from "./TranscriptView";

export function NoteEditor() {
  const activeNote = useNoteStore((s) => s.activeNote);
  const [activeTab, setActiveTab] = useState<"transcript" | "script" | "summary">("transcript");

  if (!activeNote) {
    return (
      <div className="flex items-center justify-center h-full">
        <div className="text-center animate-float-in">
          {/* Animated logo */}
          <div className="w-20 h-20 mx-auto mb-6 rounded-3xl flex items-center justify-center animate-glow"
               style={{
                 background: "linear-gradient(135deg, var(--vn-primary), var(--vn-accent))",
                 boxShadow: "var(--vn-shadow-glow)",
               }}>
            <svg width="36" height="36" viewBox="0 0 24 24" fill="none" stroke="white" strokeWidth="1.5" strokeLinecap="round">
              <path d="M12 18.5a6.5 6.5 0 1 0 0-13v0"/>
              <path d="M12 2v3M12 19v3"/>
              <path d="M5 12H2M22 12h-3"/>
            </svg>
          </div>
          <h2 className="text-xl font-bold mb-2" style={{ color: "var(--vn-text-primary)" }}>
            Ready to Record
          </h2>
          <p className="text-sm max-w-xs mx-auto" style={{ color: "var(--vn-text-tertiary)" }}>
            Select a note from the sidebar or start a new recording with
            <kbd className="mx-1 px-1.5 py-0.5 rounded text-[10px] font-mono" style={{ background: "var(--vn-bg-glass)", border: "1px solid var(--vn-border)" }}>
              Cmd+Shift+R
            </kbd>
          </p>
        </div>
      </div>
    );
  }

  return (
    <div className="h-full flex flex-col animate-float-in">
      {/* Note Header */}
      <div className="px-8 pt-6 pb-4">
        <h1 className="text-2xl font-bold mb-3" style={{ color: "var(--vn-text-primary)" }}>
          {activeNote.title}
        </h1>

        {/* Metadata Pills */}
        <div className="flex items-center gap-2 flex-wrap">
          {activeNote.language && (
            <MetaPill icon={<LangIcon />} label={activeNote.language.toUpperCase()} />
          )}
          <MetaPill
            icon={<CalendarIcon />}
            label={new Date(activeNote.created_at).toLocaleDateString("ko-KR", {
              year: "2-digit", month: "2-digit", day: "2-digit",
              hour: "2-digit", minute: "2-digit",
            })}
          />
          {activeNote.duration_ms != null && (
            <MetaPill
              icon={<ClockIcon />}
              label={`${Math.floor(activeNote.duration_ms / 60000)}m ${Math.floor((activeNote.duration_ms % 60000) / 1000)}s`}
            />
          )}
          <MetaPill icon={<FolderIcon />} label="Add to folder" clickable />
        </div>
      </div>

      {/* Tabs */}
      <div className="px-8 flex items-center gap-1 mb-1">
        {(["transcript", "script", "summary"] as const).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className="px-3 py-1.5 rounded-lg text-xs font-medium transition-glass"
            style={{
              background: activeTab === tab ? "var(--vn-primary)" : "transparent",
              color: activeTab === tab ? "white" : "var(--vn-text-tertiary)",
            }}
          >
            {tab === "transcript" ? "Conversation" : tab === "script" ? "Script" : "Summary"}
          </button>
        ))}

        <div className="flex-1" />
        <button
          onClick={async () => {
            // Copy all transcript text
            const segments = document.querySelector('[data-transcript]')?.textContent || '';
            if (segments) {
              await navigator.clipboard.writeText(segments);
              const { useToastStore } = await import('../../stores/toastStore');
              useToastStore.getState().addToast("success", "Transcript copied to clipboard");
            }
          }}
          className="p-1.5 rounded-lg transition-glass hover-glass"
          style={{ color: "var(--vn-text-tertiary)" }}
          aria-label="Copy transcript"
        >
          <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2">
            <rect x="9" y="9" width="13" height="13" rx="2"/><path d="M5 15H4a2 2 0 0 1-2-2V4a2 2 0 0 1 2-2h9a2 2 0 0 1 2 2v1"/>
          </svg>
        </button>
      </div>

      {/* Divider */}
      <div className="mx-8 h-px" style={{ background: "var(--vn-border)" }} />

      {/* Content */}
      <div className="flex-1 overflow-auto px-8 py-4">
        {activeTab === "transcript" && <TranscriptView noteId={activeNote.id} />}
        {activeTab === "script" && (
          <p className="text-sm italic" style={{ color: "var(--vn-text-tertiary)" }}>
            Full script view coming soon...
          </p>
        )}
        {activeTab === "summary" && (
          <p className="text-sm italic" style={{ color: "var(--vn-text-tertiary)" }}>
            AI summary will appear here after generation.
          </p>
        )}
      </div>
    </div>
  );
}

function MetaPill({ icon, label, clickable }: { icon: React.ReactNode; label: string; clickable?: boolean }) {
  return (
    <span
      className={`inline-flex items-center gap-1.5 px-2.5 py-1 rounded-lg text-[11px] font-medium transition-glass ${clickable ? "cursor-pointer" : ""}`}
      style={{
        background: "var(--vn-bg-glass)",
        border: "1px solid var(--vn-border)",
        color: "var(--vn-text-secondary)",
      }}
    >
      {icon}
      {label}
    </span>
  );
}

// Inline micro icons
const LangIcon = () => <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="10"/><path d="M2 12h20M12 2a15.3 15.3 0 0 1 4 10 15.3 15.3 0 0 1-4 10 15.3 15.3 0 0 1-4-10 15.3 15.3 0 0 1 4-10z"/></svg>;
const CalendarIcon = () => <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><rect x="3" y="4" width="18" height="18" rx="2"/><path d="M16 2v4M8 2v4M3 10h18"/></svg>;
const ClockIcon = () => <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><circle cx="12" cy="12" r="10"/><path d="M12 6v6l4 2"/></svg>;
const FolderIcon = () => <svg width="11" height="11" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M22 19a2 2 0 0 1-2 2H4a2 2 0 0 1-2-2V5a2 2 0 0 1 2-2h5l2 3h9a2 2 0 0 1 2 2z"/></svg>;
