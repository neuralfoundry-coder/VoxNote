import { useViewStore } from "../../stores/viewStore";

interface SetupGuideDialogProps {
  onClose: () => void;
}

export function SetupGuideDialog({ onClose }: SetupGuideDialogProps) {
  const setView = useViewStore((s) => s.setView);

  const handleGoToModels = () => {
    onClose();
    setView("models");
  };

  const handleDismiss = () => {
    localStorage.setItem("voxnote_setup_dismissed", "true");
    onClose();
  };

  return (
    <div className="fixed inset-0 z-50 flex items-center justify-center p-4">
      {/* Backdrop */}
      <div
        className="absolute inset-0 bg-black/40 backdrop-blur-sm"
        onClick={handleDismiss}
      />

      {/* Dialog */}
      <div
        className="relative w-full max-w-md glass-elevated rounded-2xl p-6 animate-float-in"
        style={{ border: "1px solid var(--vn-border)" }}
      >
        {/* Icon */}
        <div
          className="w-12 h-12 rounded-xl flex items-center justify-center mb-4"
          style={{ background: "var(--vn-primary)", color: "#fff" }}
        >
          <svg
            width="24"
            height="24"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            strokeWidth="1.5"
          >
            <rect x="4" y="4" width="16" height="16" rx="2" />
            <rect x="9" y="9" width="6" height="6" />
            <path d="M9 1v3M15 1v3M9 20v3M15 20v3M20 9h3M20 14h3M1 9h3M1 14h3" />
          </svg>
        </div>

        <h2
          className="text-lg font-bold mb-2"
          style={{ color: "var(--vn-text-primary)" }}
        >
          AI Model Setup
        </h2>

        <p
          className="text-sm mb-5"
          style={{ color: "var(--vn-text-secondary)" }}
        >
          VoxNote needs AI models to work. Download models to enable core
          features.
        </p>

        {/* Feature cards */}
        <div className="space-y-3 mb-6">
          <FeatureCard
            label="Speech-to-Text"
            tag="Required"
            tagColor="bg-red-100 text-red-700 dark:bg-red-900/40 dark:text-red-400"
            description="Transcribe audio recordings into text"
          />
          <FeatureCard
            label="LLM (Summarization)"
            tag="Optional"
            tagColor="bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400"
            description="Generate meeting notes and summaries"
          />
        </div>

        {/* Actions */}
        <div className="flex gap-3">
          <button
            onClick={handleGoToModels}
            className="flex-1 px-4 py-2.5 text-sm font-medium rounded-xl text-white transition-all hover:opacity-90"
            style={{ background: "var(--vn-primary)" }}
          >
            Go to Model Manager
          </button>
          <button
            onClick={handleDismiss}
            className="px-4 py-2.5 text-sm font-medium rounded-xl transition-glass hover-glass"
            style={{
              color: "var(--vn-text-secondary)",
              border: "1px solid var(--vn-border)",
            }}
          >
            Later
          </button>
        </div>
      </div>
    </div>
  );
}

function FeatureCard({
  label,
  tag,
  tagColor,
  description,
}: {
  label: string;
  tag: string;
  tagColor: string;
  description: string;
}) {
  return (
    <div
      className="p-3 rounded-xl"
      style={{ border: "1px solid var(--vn-border)" }}
    >
      <div className="flex items-center gap-2 mb-1">
        <span
          className="text-sm font-semibold"
          style={{ color: "var(--vn-text-primary)" }}
        >
          {label}
        </span>
        <span className={`text-[10px] px-1.5 py-0.5 rounded-full font-medium ${tagColor}`}>
          {tag}
        </span>
      </div>
      <p className="text-xs" style={{ color: "var(--vn-text-tertiary)" }}>
        {description}
      </p>
    </div>
  );
}
