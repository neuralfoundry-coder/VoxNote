import { useEffect, useRef, useState } from "react";
import { useRecordingStore } from "../../stores/recordingStore";
import { useViewStore } from "../../stores/viewStore";
import { useToastStore } from "../../stores/toastStore";
import { useSettingsStore } from "../../stores/settingsStore";
import { useSummaryStore } from "../../stores/summaryStore";

export function RecorderBar() {
  const { state, startRecording, stopRecording, pauseRecording, noteId } =
    useRecordingStore();
  const [elapsed, setElapsed] = useState(0);
  const timerRef = useRef<ReturnType<typeof setInterval> | undefined>(undefined);

  useEffect(() => {
    if (state === "recording") {
      const start = Date.now() - elapsed;
      timerRef.current = setInterval(() => {
        setElapsed(Date.now() - start);
      }, 100);
    } else {
      if (timerRef.current) clearInterval(timerRef.current);
      if (state === "idle") setElapsed(0);
    }
    return () => {
      if (timerRef.current) clearInterval(timerRef.current);
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [state]);

  const handleToggle = async () => {
    try {
      if (state === "idle" || state === "stopped") {
        setElapsed(0);
        await startRecording();
      } else if (state === "recording" || state === "paused") {
        await stopRecording();
      }
    } catch (err) {
      console.error("Recording error:", err);
    }
  };

  const isActive = state === "recording" || state === "paused";

  return (
    <div
      className="glass-elevated rounded-2xl px-5 py-3 transition-glass"
      style={{
        boxShadow: isActive ? "var(--vn-shadow-glow)" : "var(--vn-shadow-sm)",
        border: isActive ? "1px solid rgba(99, 102, 241, 0.2)" : "1px solid var(--vn-border)",
      }}
    >
      <div className="flex items-center gap-4">
        {/* Left: Status + Timer */}
        <div className="flex items-center gap-3 min-w-[140px]">
          {state === "recording" && (
            <div className="relative">
              <span className="block w-3 h-3 rounded-full bg-red-500" />
              <span className="absolute inset-0 w-3 h-3 rounded-full bg-red-500 animate-pulse-ring" />
            </div>
          )}
          {state === "paused" && (
            <span className="block w-3 h-3 rounded-full bg-amber-400" />
          )}

          <span className="text-sm font-mono font-medium tabular-nums" style={{ color: isActive ? "var(--vn-text-primary)" : "var(--vn-text-tertiary)" }}>
            {formatElapsed(elapsed)}
          </span>

          {isActive && (
            <span className="text-[10px] font-semibold uppercase tracking-wider px-2 py-0.5 rounded-md"
                  style={{
                    background: state === "recording" ? "rgba(239, 68, 68, 0.1)" : "rgba(245, 158, 11, 0.1)",
                    color: state === "recording" ? "#ef4444" : "#f59e0b",
                  }}>
              {state === "recording" ? "REC" : "PAUSED"}
            </span>
          )}
        </div>

        {/* Center: Waveform / Controls */}
        <div className="flex-1 flex items-center justify-center gap-3">
          {/* Mic selector */}
          {isActive && (
            <button className="p-2 rounded-xl transition-glass"
                    style={{ color: "var(--vn-text-secondary)" }}
                    onMouseEnter={(e) => e.currentTarget.style.background = "var(--vn-bg-glass)"}
                    onMouseLeave={(e) => e.currentTarget.style.background = "transparent"}>
              <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
                <path d="M12 2a3 3 0 0 0-3 3v7a3 3 0 0 0 6 0V5a3 3 0 0 0-3-3Z"/><path d="M19 10v2a7 7 0 0 1-14 0v-2"/><line x1="12" x2="12" y1="19" y2="22"/>
              </svg>
            </button>
          )}

          {/* Waveform visualization */}
          {state === "recording" && (
            <div className="flex items-end gap-[3px] h-6 mx-2">
              {[...Array(12)].map((_, i) => (
                <div
                  key={i}
                  className="w-[3px] rounded-full"
                  style={{
                    background: "var(--vn-primary)",
                    height: `${4 + Math.random() * 18}px`,
                    animation: `wave ${0.8 + Math.random() * 0.6}s ease-in-out ${i * 0.08}s infinite`,
                    opacity: 0.6 + Math.random() * 0.4,
                  }}
                />
              ))}
            </div>
          )}

          {/* Language badge */}
          {isActive && (
            <span className="text-[10px] font-medium px-2 py-1 rounded-lg"
                  style={{ background: "var(--vn-bg-glass)", color: "var(--vn-text-tertiary)", border: "1px solid var(--vn-border)" }}>
              {useSettingsStore.getState().config?.stt?.language?.toUpperCase() || "AUTO"}
            </span>
          )}

          {/* Pause / Resume */}
          {state === "recording" && (
            <button
              onClick={pauseRecording}
              className="p-2.5 rounded-xl transition-glass hover:scale-105 active:scale-95"
              style={{ background: "var(--vn-bg-glass)", border: "1px solid var(--vn-border)", color: "var(--vn-text-secondary)" }}
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><rect x="6" y="4" width="4" height="16" rx="1"/><rect x="14" y="4" width="4" height="16" rx="1"/></svg>
            </button>
          )}
          {state === "paused" && (
            <button
              onClick={pauseRecording}
              className="p-2.5 rounded-xl transition-glass hover:scale-105 active:scale-95"
              style={{ background: "rgba(99, 102, 241, 0.1)", border: "1px solid rgba(99, 102, 241, 0.2)", color: "var(--vn-primary)" }}
            >
              <svg width="14" height="14" viewBox="0 0 24 24" fill="currentColor"><polygon points="5 3 19 12 5 21"/></svg>
            </button>
          )}

          {/* Record / Stop */}
          <button
            onClick={handleToggle}
            className="px-5 py-2.5 rounded-xl text-sm font-semibold text-white transition-glass hover:scale-[1.03] active:scale-[0.97]"
            style={{
              background: isActive
                ? "linear-gradient(135deg, #1e293b, #334155)"
                : "linear-gradient(135deg, var(--vn-primary), var(--vn-primary-dark))",
              boxShadow: isActive ? "var(--vn-shadow-md)" : "var(--vn-shadow-glow)",
            }}
          >
            {isActive ? "Stop" : "Record"}
          </button>
        </div>

        {/* Right: Extra actions */}
        <div className="flex items-center gap-2 min-w-[140px] justify-end">
          {isActive && (
            <>
              {/* Template selector */}
              <button
                onClick={() => { if (noteId) { useSummaryStore.getState().generateSummary(noteId); useToastStore.getState().addToast("info", "Generating document..."); } }}
                className="px-3 py-2 rounded-xl text-[11px] font-medium flex items-center gap-1.5 transition-glass hover:scale-[1.02]"
                style={{ background: "linear-gradient(135deg, var(--vn-accent-warm), #ea580c)", color: "white" }}
                aria-label="Generate document from recording"
              >
                <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2"><path d="M12 3l1.9 5.5H20l-4.9 3.6 1.9 5.5L12 14l-4.9 3.6 1.9-5.5L4 8.5h6.1z"/></svg>
                Generate Doc
              </button>

              {/* Export */}
              <button
                onClick={() => { if (noteId) useViewStore.getState().openModal("export", { noteId }); }}
                className="p-2 rounded-xl transition-glass hover-glass"
                style={{ color: "var(--vn-text-tertiary)" }}
                aria-label="Export note"
              >
                <svg width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
                  <path d="M4 12v8a2 2 0 0 0 2 2h12a2 2 0 0 0 2-2v-8"/><polyline points="16 6 12 2 8 6"/><line x1="12" x2="12" y1="2" y2="15"/>
                </svg>
              </button>
            </>
          )}
        </div>
      </div>
    </div>
  );
}

function formatElapsed(ms: number): string {
  const totalSecs = Math.floor(ms / 1000);
  const hrs = Math.floor(totalSecs / 3600);
  const mins = Math.floor((totalSecs % 3600) / 60);
  const secs = totalSecs % 60;
  if (hrs > 0) {
    return `${hrs}:${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
  }
  return `${mins.toString().padStart(2, "0")}:${secs.toString().padStart(2, "0")}`;
}
