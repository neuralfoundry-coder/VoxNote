import { useEffect, useRef, useState } from "react";
import { useNoteStore } from "../../stores/noteStore";
import { useViewStore } from "../../stores/viewStore";
import { useRecordingStore } from "../../stores/recordingStore";
import { useToastStore } from "../../stores/toastStore";
import type { Note } from "../../lib/types";

export function Sidebar() {
  const { notes, activeNote, setActiveNote, searchNotes, searchResults, createNote } =
    useNoteStore();
  const { activeView, setView } = useViewStore();
  const startRecording = useRecordingStore((s) => s.startRecording);
  const addToast = useToastStore((s) => s.addToast);
  const [searchQuery, setSearchQuery] = useState("");
  const [showUserMenu, setShowUserMenu] = useState(false);
  const menuRef = useRef<HTMLDivElement>(null);

  // Click outside to close menu
  useEffect(() => {
    if (!showUserMenu) return;
    const handleClick = (e: MouseEvent) => {
      if (menuRef.current && !menuRef.current.contains(e.target as Node)) {
        setShowUserMenu(false);
      }
    };
    document.addEventListener("mousedown", handleClick);
    return () => document.removeEventListener("mousedown", handleClick);
  }, [showUserMenu]);

  const handleSearch = (query: string) => {
    setSearchQuery(query);
    searchNotes(query);
  };

  const handleNewNote = async () => {
    const title = `Recording ${new Date().toLocaleString("ko-KR", { month: "short", day: "numeric", hour: "2-digit", minute: "2-digit" })}`;
    const note = await createNote(title);
    setActiveNote(note);
  };

  return (
    <div className="flex flex-col h-full">
      {/* Profile Header + Dropdown */}
      <div className="p-4 pb-3 relative" ref={menuRef}>
        <button
          onClick={() => setShowUserMenu(!showUserMenu)}
          className="flex items-center gap-3 w-full rounded-xl p-2 -m-2 transition-glass hover-glass"
          style={{ color: "var(--vn-text-primary)" }}
          aria-label="User menu"
          aria-expanded={showUserMenu}
          aria-haspopup="true"
        >
          <div className="w-9 h-9 rounded-full flex items-center justify-center text-white text-sm font-bold"
               style={{ background: "linear-gradient(135deg, var(--vn-primary), var(--vn-accent))" }}>
            V
          </div>
          <div className="flex-1 text-left">
            <div className="text-sm font-semibold">VoxNote</div>
            <div className="text-[11px]" style={{ color: "var(--vn-text-tertiary)" }}>Local AI</div>
          </div>
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ color: "var(--vn-text-tertiary)", transform: showUserMenu ? "rotate(180deg)" : "none", transition: "transform 0.2s" }}>
            <path d="M6 9l6 6 6-6"/>
          </svg>
        </button>

        {/* User menu dropdown */}
        {showUserMenu && (
          <div className="absolute left-3 right-3 top-full mt-1 glass-elevated rounded-xl py-1 z-50 animate-float-in" style={{ boxShadow: "var(--vn-shadow-lg)" }}>
            <MenuItem label="Settings" icon="M12.22 2h-.44a2 2 0 00-2 2v.18a2 2 0 01-1 1.73l-.43.25a2 2 0 01-2 0l-.15-.08a2 2 0 00-2.73.73l-.22.38a2 2 0 00.73 2.73l.15.1a2 2 0 011 1.72v.51a2 2 0 01-1 1.74l-.15.09a2 2 0 00-.73 2.73l.22.38a2 2 0 002.73.73l.15-.08a2 2 0 012 0l.43.25a2 2 0 011 1.73V20a2 2 0 002 2h.44a2 2 0 002-2v-.18a2 2 0 011-1.73l.43-.25a2 2 0 012 0l.15.08a2 2 0 002.73-.73l.22-.39a2 2 0 00-.73-2.73l-.15-.08a2 2 0 01-1-1.74v-.5a2 2 0 011-1.74l.15-.09a2 2 0 00.73-2.73l-.22-.38a2 2 0 00-2.73-.73l-.15.08a2 2 0 01-2 0l-.43-.25a2 2 0 01-1-1.73V4a2 2 0 00-2-2z M12 8a4 4 0 100 8 4 4 0 000-8z"
              onClick={() => { setView("settings"); setShowUserMenu(false); }} />
            <MenuItem label="Models" icon="M4 4h16v16H4z M9 9h6v6H9z M9 1v3M15 1v3M9 20v3M15 20v3M20 9h3M20 14h3M1 9h3M1 14h3"
              onClick={() => { setView("models"); setShowUserMenu(false); }} />
            <MenuItem label="AI Providers" icon="M18 10h-1.26A8 8 0 109 20h9a5 5 0 000-10z"
              onClick={() => { setView("providers"); setShowUserMenu(false); }} />
            <MenuItem label="Ask VoxNote" icon="M21 15a2 2 0 01-2 2H7l-4 4V5a2 2 0 012-2h14a2 2 0 012 2z"
              onClick={() => { setView("ask"); setShowUserMenu(false); }} />
            <div className="mx-3 my-1 h-px" style={{ background: "var(--vn-border)" }} />
            <MenuItem label="Account" icon="M20 21v-2a4 4 0 00-4-4H8a4 4 0 00-4 4v2 M12 3a4 4 0 100 8 4 4 0 000-8z"
              onClick={() => { setView("account"); setShowUserMenu(false); }} />
          </div>
        )}
      </div>

      {/* New Note Button */}
      <div className="px-4 pb-3">
        <button
          onClick={handleNewNote}
          className="w-full py-2.5 rounded-xl text-sm font-semibold text-white flex items-center justify-center gap-2 transition-glass hover:scale-[1.02] active:scale-[0.98]"
          style={{
            background: "linear-gradient(135deg, var(--vn-primary), var(--vn-primary-dark))",
            boxShadow: "var(--vn-shadow-glow)",
          }}
        >
          <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2.5" strokeLinecap="round"><path d="M12 18.5a6.5 6.5 0 1 0 0-13v0"/><path d="M12 2v3M12 19v3"/></svg>
          New Recording
        </button>
      </div>

      {/* Quick Actions */}
      <div className="px-4 pb-3 flex gap-2">
        <button
          onClick={async () => { try { await startRecording(); setView("notes"); addToast("info", "Recording started"); } catch { addToast("error", "Failed to start recording"); } }}
          className="flex-1 py-1.5 rounded-lg text-[11px] font-medium flex items-center justify-center gap-1.5 transition-glass hover-glass"
          style={{ background: "var(--vn-bg-glass)", color: "var(--vn-text-secondary)", border: "1px solid var(--vn-border)" }}
          aria-label="Start voice recording"
        >
          <span className="w-1.5 h-1.5 rounded-full bg-red-500" />
          Voice
        </button>
        <button
          onClick={() => addToast("info", "File import coming soon")}
          className="flex-1 py-1.5 rounded-lg text-[11px] font-medium flex items-center justify-center gap-1.5 transition-glass hover-glass"
          style={{ background: "var(--vn-bg-glass)", color: "var(--vn-text-secondary)", border: "1px solid var(--vn-border)" }}
          aria-label="Import audio file"
        >
          <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M14 2H6a2 2 0 0 0-2 2v16a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2V8z"/><path d="M14 2v6h6"/></svg>
          File
        </button>
      </div>

      {/* Search */}
      <div className="px-4 pb-2">
        <div className="relative">
          <svg className="absolute left-3 top-1/2 -translate-y-1/2" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" style={{ color: "var(--vn-text-tertiary)" }}>
            <circle cx="11" cy="11" r="8"/><path d="m21 21-4.35-4.35"/>
          </svg>
          <input
            type="text"
            placeholder="Search notes..."
            value={searchQuery}
            onChange={(e) => handleSearch(e.target.value)}
            className="w-full pl-9 pr-3 py-2 rounded-xl text-xs outline-none transition-glass"
            style={{
              background: "var(--vn-bg-glass)",
              border: "1px solid var(--vn-border)",
              color: "var(--vn-text-primary)",
            }}
            onFocus={(e) => e.target.style.borderColor = "var(--vn-primary)"}
            onBlur={(e) => e.target.style.borderColor = "var(--vn-border)"}
          />
        </div>
      </div>

      {/* Search Results */}
      {searchQuery.trim() && searchResults.length > 0 && (
        <div className="px-4 pb-2 animate-float-in">
          <p className="text-[10px] font-medium mb-1.5" style={{ color: "var(--vn-text-tertiary)" }}>
            {searchResults.length} results
          </p>
          {searchResults.slice(0, 5).map((result) => (
            <div
              key={result.segment_id}
              className="p-2 mb-1 rounded-lg cursor-pointer text-xs transition-glass hover-glass"
              style={{ color: "var(--vn-text-secondary)" }}
              onClick={() => {
                const note = notes.find((n) => n.id === result.note_id);
                if (note) { setActiveNote(note); setView("notes"); }
              }}
              dangerouslySetInnerHTML={{ __html: result.highlight.replace(/<(?!\/?mark\b)[^>]*>/gi, "") }}
            />
          ))}
        </div>
      )}

      {/* Navigation */}
      <div className="px-3 pt-1 pb-1 space-y-0.5">
        <NavItem icon="m3 9 9-7 9 7v11a2 2 0 01-2 2H5a2 2 0 01-2-2z" label="All Notes" view="notes" active={activeView === "notes"} onClick={() => setView("notes")} />
        <NavItem icon="M12.22 2h-.44a2 2 0 00-2 2v.18a2 2 0 01-1 1.73l-.43.25a2 2 0 01-2 0l-.15-.08a2 2 0 00-2.73.73l-.22.38a2 2 0 00.73 2.73l.15.1a2 2 0 011 1.72v.51a2 2 0 01-1 1.74l-.15.09a2 2 0 00-.73 2.73l.22.38a2 2 0 002.73.73l.15-.08a2 2 0 012 0l.43.25a2 2 0 011 1.73V20a2 2 0 002 2h.44a2 2 0 002-2v-.18a2 2 0 011-1.73l.43-.25a2 2 0 012 0l.15.08a2 2 0 002.73-.73l.22-.39a2 2 0 00-.73-2.73l-.15-.08a2 2 0 01-1-1.74v-.5a2 2 0 011-1.74l.15-.09a2 2 0 00.73-2.73l-.22-.38a2 2 0 00-2.73-.73l-.15.08a2 2 0 01-2 0l-.43-.25a2 2 0 01-1-1.73V4a2 2 0 00-2-2z" label="Settings" view="settings" active={activeView === "settings"} onClick={() => setView("settings")} />
      </div>

      {/* Note List */}
      <div className="flex-1 overflow-auto px-2">
        {!searchQuery.trim() && notes.length > 0 && (
          <div className="px-2 pt-2 pb-1">
            <p className="text-[10px] font-semibold uppercase tracking-wider" style={{ color: "var(--vn-text-tertiary)" }}>
              Recent
            </p>
          </div>
        )}
        {!searchQuery.trim() && notes.map((note) => (
          <NoteItem
            key={note.id}
            note={note}
            isActive={activeNote?.id === note.id}
            onClick={() => setActiveNote(note)}
          />
        ))}
        {notes.length === 0 && !searchQuery.trim() && (
          <div className="px-4 py-12 text-center">
            <div className="w-12 h-12 mx-auto mb-3 rounded-2xl flex items-center justify-center"
                 style={{ background: "var(--vn-bg-glass)", border: "1px solid var(--vn-border)" }}>
              <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" style={{ color: "var(--vn-text-tertiary)" }}>
                <path d="M12 18.5a6.5 6.5 0 1 0 0-13v0"/><path d="M12 2v3M12 19v3"/>
              </svg>
            </div>
            <p className="text-xs" style={{ color: "var(--vn-text-tertiary)" }}>
              Start your first recording
            </p>
          </div>
        )}
      </div>
    </div>
  );
}

function NoteItem({ note, isActive, onClick }: { note: Note; isActive: boolean; onClick: () => void }) {
  const statusConfig: Record<string, { color: string; label: string }> = {
    recording: { color: "#ef4444", label: "REC" },
    transcribing: { color: "#f59e0b", label: "STT" },
    summarizing: { color: "#6366f1", label: "AI" },
    done: { color: "#22c55e", label: "" },
    error: { color: "#94a3b8", label: "ERR" },
  };

  const status = statusConfig[note.status] || statusConfig.done;

  return (
    <div
      onClick={onClick}
      className="mx-1 px-3 py-2.5 rounded-xl cursor-pointer transition-glass group"
      style={{
        background: isActive ? "var(--vn-bg-glass)" : "transparent",
        border: isActive ? "1px solid var(--vn-border-glass)" : "1px solid transparent",
      }}
      onMouseEnter={(e) => { if (!isActive) (e.currentTarget as HTMLElement).style.background = "var(--vn-bg-glass)"; }}
      onMouseLeave={(e) => { if (!isActive) (e.currentTarget as HTMLElement).style.background = "transparent"; }}
    >
      <div className="flex items-start gap-2.5">
        <div className="mt-1 shrink-0">
          <span className="block w-2 h-2 rounded-full" style={{ background: status.color }} />
        </div>
        <div className="flex-1 min-w-0">
          <div className="text-[13px] font-medium truncate" style={{ color: "var(--vn-text-primary)" }}>
            {note.title}
          </div>
          <div className="flex items-center gap-2 mt-0.5">
            <span className="text-[10px]" style={{ color: "var(--vn-text-tertiary)" }}>
              {new Date(note.created_at).toLocaleDateString("ko-KR", { month: "short", day: "numeric" })}
            </span>
            {note.duration_ms != null && (
              <span className="text-[10px]" style={{ color: "var(--vn-text-tertiary)" }}>
                {Math.round(note.duration_ms / 60000)}min
              </span>
            )}
            {status.label && (
              <span className="text-[9px] font-bold px-1.5 py-0.5 rounded-md" style={{
                background: `${status.color}18`,
                color: status.color,
              }}>
                {status.label}
              </span>
            )}
          </div>
        </div>
      </div>
    </div>
  );
}

function MenuItem({ label, icon, onClick }: { label: string; icon: string; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className="flex items-center gap-2.5 w-full px-3 py-2 text-xs font-medium rounded-lg transition-glass hover-glass"
      style={{ color: "var(--vn-text-secondary)" }}
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
        <path d={icon} />
      </svg>
      {label}
    </button>
  );
}

function NavItem({ icon, label, active, onClick }: { icon: string; label: string; view: string; active: boolean; onClick: () => void }) {
  return (
    <button
      onClick={onClick}
      className="flex items-center gap-2 w-full py-1.5 px-2.5 text-xs font-medium rounded-lg transition-glass"
      style={{
        background: active ? "var(--vn-bg-glass)" : "transparent",
        color: active ? "var(--vn-text-primary)" : "var(--vn-text-secondary)",
        border: active ? "1px solid var(--vn-border)" : "1px solid transparent",
      }}
      onMouseEnter={(e) => { if (!active) (e.currentTarget as HTMLElement).style.background = "var(--vn-bg-glass)"; }}
      onMouseLeave={(e) => { if (!active) (e.currentTarget as HTMLElement).style.background = "transparent"; }}
    >
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5" strokeLinecap="round" strokeLinejoin="round">
        <path d={icon} />
      </svg>
      {label}
    </button>
  );
}
