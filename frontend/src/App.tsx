import { useEffect } from "react";
import { useNoteStore } from "./stores/noteStore";
import { useThemeStore } from "./stores/themeStore";
import { useViewStore, type ViewId } from "./stores/viewStore";
import { Sidebar } from "./components/Sidebar/NoteList";
import { NoteEditor } from "./components/Editor/NoteEditor";
import { RecorderBar } from "./components/Recorder/RecorderBar";
import { SummaryPanel } from "./components/Summary/SummaryPanel";
import { SettingsPanel } from "./components/Settings/SettingsPanel";
import { ModelManager } from "./components/Settings/ModelManager";
import { ProviderSettings } from "./components/Settings/ProviderSettings";
import { AccountSettings } from "./components/Settings/AccountSettings";
import { ChatPanel } from "./components/AskVoxNote/ChatPanel";
import { ExportDialog } from "./components/Export/ExportDialog";
import { ToastContainer } from "./components/Toast/ToastContainer";

function App() {
  const fetchNotes = useNoteStore((s) => s.fetchNotes);
  const activeView = useViewStore((s) => s.activeView);
  const activeModal = useViewStore((s) => s.activeModal);
  const modalProps = useViewStore((s) => s.modalProps);
  const closeModal = useViewStore((s) => s.closeModal);
  useThemeStore();

  useEffect(() => { fetchNotes(); }, [fetchNotes]);

  useEffect(() => {
    const handleKey = (e: KeyboardEvent) => {
      if ((e.metaKey || e.ctrlKey) && e.key === ",") {
        e.preventDefault();
        useViewStore.getState().setView("settings");
      }
    };
    window.addEventListener("keydown", handleKey);
    return () => window.removeEventListener("keydown", handleKey);
  }, []);

  return (
    <div className="mesh-bg flex flex-col h-screen overflow-hidden">
      <div className="drag-region h-8 shrink-0 flex items-center justify-between px-4">
        <div className="w-20" />
        <span className="no-drag text-[10px] font-medium tracking-wider uppercase" style={{ color: "var(--vn-text-tertiary)" }}>VoxNote</span>
        <div className="no-drag flex items-center gap-1"><ThemeToggle /></div>
      </div>

      <div className="flex flex-1 min-h-0 px-2 pb-2 gap-2">
        <aside className="w-[280px] shrink-0 glass-surface rounded-2xl flex flex-col overflow-hidden">
          <Sidebar />
        </aside>
        <main className="flex-1 flex flex-col min-w-0 gap-2">
          <div className="flex-1 glass-elevated rounded-2xl overflow-hidden flex min-h-0">
            <ViewRenderer view={activeView} />
          </div>
          {activeView === "notes" && <RecorderBar />}
        </main>
      </div>

      {activeModal === "export" && <ExportDialog noteId={modalProps.noteId as string} onClose={closeModal} />}
      <ToastContainer />
    </div>
  );
}

function ViewRenderer({ view }: { view: ViewId }) {
  const activeNote = useNoteStore((s) => s.activeNote);
  switch (view) {
    case "notes":
      return (
        <>
          <div className="flex-1 overflow-auto"><NoteEditor /></div>
          {activeNote && <div className="w-[300px] shrink-0 border-l overflow-auto" style={{ borderColor: "var(--vn-border)" }}><SummaryPanel /></div>}
        </>
      );
    case "settings": return <SettingsView />;
    case "models": return <PageShell title="Model Manager" onBack><ModelManager /></PageShell>;
    case "providers": return <PageShell title="AI Providers" onBack><ProviderSettings /></PageShell>;
    case "account": return <PageShell title="Account" onBack><AccountSettings /></PageShell>;
    case "ask": return <PageShell title="Ask VoxNote" onBack><ChatPanel /></PageShell>;
    default: return null;
  }
}

function SettingsView() {
  const setView = useViewStore((s) => s.setView);
  return (
    <div className="flex-1 overflow-auto p-8 animate-float-in">
      <div className="max-w-2xl mx-auto">
        <BackButton />
        <h1 className="text-2xl font-bold mb-6" style={{ color: "var(--vn-text-primary)" }}>Settings</h1>
        <div className="grid grid-cols-2 gap-3 mb-8">
          <SettingsCard icon={<CpuIcon />} title="Models" desc="Download and manage AI models" onClick={() => setView("models")} />
          <SettingsCard icon={<CloudIcon />} title="AI Providers" desc="Configure cloud AI services" onClick={() => setView("providers")} />
          <SettingsCard icon={<UserIcon />} title="Account" desc="Sign in and manage profile" onClick={() => setView("account")} />
          <SettingsCard icon={<ChatIcon />} title="Ask VoxNote" desc="Chat with your meeting notes" onClick={() => setView("ask")} />
        </div>
        <SettingsPanel />
      </div>
    </div>
  );
}

function SettingsCard({ icon, title, desc, onClick }: { icon: React.ReactNode; title: string; desc: string; onClick: () => void }) {
  return (
    <button onClick={onClick} className="text-left p-4 rounded-xl transition-glass hover-glass" style={{ border: "1px solid var(--vn-border)" }}>
      <div className="mb-2" style={{ color: "var(--vn-primary)" }}>{icon}</div>
      <div className="text-sm font-semibold mb-0.5" style={{ color: "var(--vn-text-primary)" }}>{title}</div>
      <div className="text-[11px]" style={{ color: "var(--vn-text-tertiary)" }}>{desc}</div>
    </button>
  );
}

function PageShell({ title, onBack, children }: { title: string; onBack?: boolean; children: React.ReactNode }) {
  return (
    <div className="flex-1 overflow-auto p-8 animate-float-in">
      <div className="max-w-2xl mx-auto">
        {onBack && <BackButton />}
        <h1 className="text-2xl font-bold mb-6" style={{ color: "var(--vn-text-primary)" }}>{title}</h1>
        {children}
      </div>
    </div>
  );
}

function BackButton() {
  const goBack = useViewStore((s) => s.goBack);
  return (
    <button onClick={goBack} className="flex items-center gap-1.5 text-xs font-medium mb-4 px-2 py-1 rounded-lg transition-glass hover-glass" style={{ color: "var(--vn-text-secondary)" }} aria-label="Go back">
      <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><path d="M19 12H5M12 19l-7-7 7-7" /></svg>
      Back
    </button>
  );
}

function ThemeToggle() {
  const { resolved, setTheme } = useThemeStore();
  return (
    <button onClick={() => setTheme(resolved === "dark" ? "light" : "dark")} className="w-7 h-7 rounded-lg flex items-center justify-center transition-glass hover-glass" style={{ color: "var(--vn-text-secondary)" }} aria-label={`Switch to ${resolved === "dark" ? "light" : "dark"} mode`}>
      {resolved === "dark" ? (
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><circle cx="12" cy="12" r="5"/><path d="M12 1v2M12 21v2M4.22 4.22l1.42 1.42M18.36 18.36l1.42 1.42M1 12h2M21 12h2M4.22 19.78l1.42-1.42M18.36 5.64l1.42-1.42"/></svg>
      ) : (
        <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round"><path d="M21 12.79A9 9 0 1 1 11.21 3 7 7 0 0 0 21 12.79z"/></svg>
      )}
    </button>
  );
}

const CpuIcon = () => <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><rect x="4" y="4" width="16" height="16" rx="2"/><rect x="9" y="9" width="6" height="6"/><path d="M9 1v3M15 1v3M9 20v3M15 20v3M20 9h3M20 14h3M1 9h3M1 14h3"/></svg>;
const CloudIcon = () => <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M18 10h-1.26A8 8 0 1 0 9 20h9a5 5 0 0 0 0-10z"/></svg>;
const UserIcon = () => <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M20 21v-2a4 4 0 0 0-4-4H8a4 4 0 0 0-4 4v2"/><circle cx="12" cy="7" r="4"/></svg>;
const ChatIcon = () => <svg width="20" height="20" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="1.5"><path d="M21 15a2 2 0 0 1-2 2H7l-4 4V5a2 2 0 0 1 2-2h14a2 2 0 0 1 2 2z"/></svg>;

export default App;
