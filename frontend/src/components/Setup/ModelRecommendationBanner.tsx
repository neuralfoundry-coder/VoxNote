import { useState } from "react";
import { useModelStore } from "../../stores/modelStore";
import { useViewStore } from "../../stores/viewStore";

export function ModelRecommendationBanner() {
  const models = useModelStore((s) => s.models);
  const getStatus = useModelStore((s) => s.getStatusByFeature);
  const setView = useViewStore((s) => s.setView);
  const [dismissed, setDismissed] = useState<Record<string, boolean>>(() => {
    try {
      const stored = sessionStorage.getItem("voxnote_banner_dismissed");
      return stored ? JSON.parse(stored) : {};
    } catch {
      return {};
    }
  });

  if (models.length === 0) return null;

  const stt = getStatus("stt");
  const llm = getStatus("llm");

  const banners: { key: string; message: string; level: "warn" | "info" }[] = [];

  if (!stt.hasDownloaded) {
    banners.push({
      key: "stt-none",
      message: "Recording requires an STT model. Download one to get started.",
      level: "warn",
    });
  } else if (!stt.hasActive) {
    banners.push({
      key: "stt-inactive",
      message: `STT model downloaded but not active. Activate a model to enable recording.`,
      level: "info",
    });
  }

  if (stt.hasActive && !llm.hasDownloaded) {
    banners.push({
      key: "llm-none",
      message: "Download an LLM model to enable meeting summaries.",
      level: "info",
    });
  } else if (llm.hasDownloaded && !llm.hasActive) {
    banners.push({
      key: "llm-inactive",
      message: "LLM model downloaded but not active. Activate to enable summaries.",
      level: "info",
    });
  }

  const visibleBanners = banners.filter((b) => !dismissed[b.key]);
  if (visibleBanners.length === 0) return null;

  const handleDismiss = (key: string) => {
    const next = { ...dismissed, [key]: true };
    setDismissed(next);
    sessionStorage.setItem("voxnote_banner_dismissed", JSON.stringify(next));
  };

  return (
    <div className="px-4 pt-3 space-y-2">
      {visibleBanners.map((b) => (
        <div
          key={b.key}
          className="flex items-center gap-3 px-4 py-2.5 rounded-xl text-xs"
          style={{
            border: "1px solid var(--vn-border)",
            background:
              b.level === "warn"
                ? "var(--vn-warning-bg, rgba(234,179,8,0.08))"
                : "var(--vn-info-bg, rgba(59,130,246,0.08))",
            color: "var(--vn-text-secondary)",
          }}
        >
          {b.level === "warn" ? (
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" className="shrink-0" style={{ color: "var(--vn-warning, #eab308)" }}>
              <path d="M10.29 3.86L1.82 18a2 2 0 0 0 1.71 3h16.94a2 2 0 0 0 1.71-3L13.71 3.86a2 2 0 0 0-3.42 0z" />
              <line x1="12" y1="9" x2="12" y2="13" />
              <line x1="12" y1="17" x2="12.01" y2="17" />
            </svg>
          ) : (
            <svg width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round" className="shrink-0" style={{ color: "var(--vn-primary)" }}>
              <circle cx="12" cy="12" r="10" />
              <line x1="12" y1="16" x2="12" y2="12" />
              <line x1="12" y1="8" x2="12.01" y2="8" />
            </svg>
          )}
          <span className="flex-1">{b.message}</span>
          <button
            onClick={() => setView("models")}
            className="shrink-0 px-2.5 py-1 rounded-lg text-xs font-medium transition-glass hover-glass"
            style={{ color: "var(--vn-primary)" }}
          >
            Model Manager
          </button>
          <button
            onClick={() => handleDismiss(b.key)}
            className="shrink-0 p-1 rounded-lg transition-glass hover-glass"
            style={{ color: "var(--vn-text-tertiary)" }}
            aria-label="Dismiss"
          >
            <svg width="12" height="12" viewBox="0 0 24 24" fill="none" stroke="currentColor" strokeWidth="2" strokeLinecap="round">
              <line x1="18" y1="6" x2="6" y2="18" />
              <line x1="6" y1="6" x2="18" y2="18" />
            </svg>
          </button>
        </div>
      ))}
    </div>
  );
}
