import { useEffect, useState } from "react";
import { tauriInvoke, useTauriEvent } from "../../hooks/useTauriIPC";
import { useModelStore } from "../../stores/modelStore";
import type { DownloadProgress, ModelInfo, ModelTestResult } from "../../lib/types";

export function ModelManager() {
  const models = useModelStore((s) => s.models);
  const fetchModels = useModelStore((s) => s.fetchModels);
  const [downloading, setDownloading] = useState<Record<string, number>>({});
  const [activating, setActivating] = useState<string | null>(null);
  const [testing, setTesting] = useState<Record<string, boolean>>({});
  const [testResults, setTestResults] = useState<Record<string, ModelTestResult>>({});

  useEffect(() => {
    fetchModels();
  }, [fetchModels]);

  useTauriEvent<DownloadProgress>("model:download-progress", (progress) => {
    setDownloading((prev) => ({
      ...prev,
      [progress.model_id]: progress.percentage,
    }));
    if (progress.percentage >= 100) {
      setDownloading((prev) => {
        const next = { ...prev };
        delete next[progress.model_id];
        return next;
      });
      fetchModels();
    }
  });

  useTauriEvent<{ model_id: string }>("model:download-complete", () => {
    fetchModels();
  });

  const handleDownload = async (modelId: string) => {
    try {
      await tauriInvoke<string>("download_model", { modelId });
    } catch (err) {
      console.error("Download failed:", err);
    }
  };

  const handleDelete = async (modelId: string) => {
    try {
      await tauriInvoke<void>("delete_model", { modelId });
      fetchModels();
    } catch (err) {
      console.error("Delete failed:", err);
    }
  };

  const handleActivate = async (modelId: string) => {
    setActivating(modelId);
    try {
      await tauriInvoke<string>("activate_model", { modelId });
      fetchModels();
    } catch (err) {
      console.error("Activate failed:", err);
    } finally {
      setActivating(null);
    }
  };

  const handleTest = async (modelId: string) => {
    setTesting((prev) => ({ ...prev, [modelId]: true }));
    setTestResults((prev) => {
      const next = { ...prev };
      delete next[modelId];
      return next;
    });
    try {
      const result = await tauriInvoke<ModelTestResult>("test_model", { modelId });
      setTestResults((prev) => ({ ...prev, [modelId]: result }));
      // Auto-clear after 15 seconds
      setTimeout(() => {
        setTestResults((prev) => {
          const next = { ...prev };
          delete next[modelId];
          return next;
        });
      }, 15000);
    } catch (err) {
      setTestResults((prev) => ({
        ...prev,
        [modelId]: { success: false, output: String(err), duration_ms: 0 },
      }));
    } finally {
      setTesting((prev) => ({ ...prev, [modelId]: false }));
    }
  };

  const sttModels = models.filter((m) => m.model_type === "stt");
  const llmModels = models.filter((m) => m.model_type === "llm");
  const otherModels = models.filter(
    (m) => m.model_type !== "stt" && m.model_type !== "llm"
  );

  return (
    <div className="space-y-8">
      <ModelSection
        title="Speech-to-Text (STT)"
        models={sttModels}
        downloading={downloading}
        activating={activating}
        testing={testing}
        testResults={testResults}
        onDownload={handleDownload}
        onDelete={handleDelete}
        onActivate={handleActivate}
        onTest={handleTest}
      />

      <ModelSection
        title="LLM (Summarization)"
        models={llmModels}
        downloading={downloading}
        activating={activating}
        testing={testing}
        testResults={testResults}
        onDownload={handleDownload}
        onDelete={handleDelete}
        onActivate={handleActivate}
        onTest={handleTest}
      />

      {otherModels.length > 0 && (
        <ModelSection
          title="Other"
          models={otherModels}
          downloading={downloading}
          activating={activating}
          testing={testing}
          testResults={testResults}
          onDownload={handleDownload}
          onDelete={handleDelete}
          onActivate={handleActivate}
          onTest={handleTest}
        />
      )}

      {models.length === 0 && (
        <p
          className="text-sm text-center py-8"
          style={{ color: "var(--vn-text-tertiary)" }}
        >
          No models available. Check registry.toml.
        </p>
      )}
    </div>
  );
}

function ModelSection({
  title,
  models,
  downloading,
  activating,
  testing,
  testResults,
  onDownload,
  onDelete,
  onActivate,
  onTest,
}: {
  title: string;
  models: ModelInfo[];
  downloading: Record<string, number>;
  activating: string | null;
  testing: Record<string, boolean>;
  testResults: Record<string, ModelTestResult>;
  onDownload: (id: string) => void;
  onDelete: (id: string) => void;
  onActivate: (id: string) => void;
  onTest: (id: string) => void;
}) {
  if (models.length === 0) return null;

  const activeModel = models.find((m) => m.is_active);

  return (
    <section>
      <div className="flex items-center gap-2 mb-3">
        <h3
          className="text-sm font-semibold uppercase"
          style={{ color: "var(--vn-text-tertiary)" }}
        >
          {title}
        </h3>
        {activeModel ? (
          <span
            className="text-xs px-2 py-0.5 rounded-full"
            style={{
              background: "rgba(59,130,246,0.12)",
              color: "var(--vn-primary)",
            }}
          >
            Active: {activeModel.name}
          </span>
        ) : (
          <span
            className="text-xs px-2 py-0.5 rounded-full"
            style={{
              background: "rgba(234,179,8,0.12)",
              color: "var(--vn-warning, #eab308)",
            }}
          >
            No active model
          </span>
        )}
      </div>
      <div className="space-y-3">
        {models.map((model) => (
          <ModelCard
            key={model.id}
            model={model}
            downloadProgress={downloading[model.id]}
            isActivating={activating === model.id}
            isTesting={testing[model.id] ?? false}
            testResult={testResults[model.id]}
            onDownload={() => onDownload(model.id)}
            onDelete={() => onDelete(model.id)}
            onActivate={() => onActivate(model.id)}
            onTest={() => onTest(model.id)}
          />
        ))}
      </div>
    </section>
  );
}

function ModelCard({
  model,
  downloadProgress,
  isActivating,
  isTesting,
  testResult,
  onDownload,
  onDelete,
  onActivate,
  onTest,
}: {
  model: ModelInfo;
  downloadProgress: number | undefined;
  isActivating: boolean;
  isTesting: boolean;
  testResult: ModelTestResult | undefined;
  onDownload: () => void;
  onDelete: () => void;
  onActivate: () => void;
  onTest: () => void;
}) {
  const isDownloading = downloadProgress != null;

  return (
    <div
      className="p-4 rounded-xl transition-all"
      style={{
        border: model.is_active
          ? "1px solid var(--vn-primary)"
          : "1px solid var(--vn-border)",
        background: model.is_active
          ? "rgba(59,130,246,0.06)"
          : undefined,
      }}
    >
      <div className="flex items-start justify-between">
        <div className="flex-1 min-w-0">
          <div className="flex items-center gap-2">
            <h3
              className="text-sm font-medium truncate"
              style={{ color: "var(--vn-text-primary)" }}
            >
              {model.name}
            </h3>
            {model.is_active && (
              <span className="shrink-0 text-xs px-2 py-0.5 bg-blue-500 text-white rounded-full">
                Active
              </span>
            )}
          </div>
          <div className="flex items-center gap-2 mt-1">
            <span className="text-xs" style={{ color: "var(--vn-text-tertiary)" }}>
              {model.size_display}
            </span>
            <ModelTypeBadge type={model.model_type} name={model.name} />
            {model.gpu_recommended && (
              <span className="text-xs px-1.5 py-0.5 bg-yellow-100 text-yellow-700 dark:bg-yellow-900 dark:text-yellow-300 rounded">
                GPU
              </span>
            )}
          </div>
          {model.description && (
            <p
              className="text-xs mt-1 line-clamp-2"
              style={{ color: "var(--vn-text-tertiary)" }}
            >
              {model.description}
            </p>
          )}
          {/* Test result inline */}
          {testResult && (
            <div
              className="mt-2 text-xs px-2 py-1 rounded-lg inline-block"
              style={{
                background: testResult.success
                  ? "rgba(34,197,94,0.1)"
                  : "rgba(239,68,68,0.1)",
                color: testResult.success
                  ? "var(--vn-success, #22c55e)"
                  : "var(--vn-error, #ef4444)",
              }}
            >
              {testResult.success ? "✓" : "✗"} {testResult.output}
              {testResult.duration_ms > 0 && ` (${(testResult.duration_ms / 1000).toFixed(1)}s)`}
            </div>
          )}
        </div>

        <div className="ml-3 flex items-center gap-2 shrink-0">
          {isDownloading ? (
            <div className="w-28">
              <div className="h-2 rounded-full overflow-hidden" style={{ background: "var(--vn-border)" }}>
                <div
                  className="h-full transition-all"
                  style={{ width: `${downloadProgress}%`, background: "var(--vn-primary)" }}
                />
              </div>
              <span className="text-xs mt-1" style={{ color: "var(--vn-text-tertiary)" }}>
                {Math.round(downloadProgress ?? 0)}%
              </span>
            </div>
          ) : model.is_downloaded ? (
            <>
              <button
                onClick={onTest}
                disabled={isTesting}
                className="px-3 py-1 text-xs rounded-lg transition-glass hover-glass disabled:opacity-50"
                style={{
                  border: "1px solid var(--vn-border)",
                  color: "var(--vn-text-secondary)",
                }}
              >
                {isTesting ? "Testing..." : "Test"}
              </button>
              {!model.is_active && (
                <button
                  onClick={onActivate}
                  disabled={isActivating}
                  className="px-3 py-1 text-xs rounded-lg text-white disabled:opacity-50"
                  style={{ background: "var(--vn-primary)" }}
                >
                  {isActivating ? "..." : "Activate"}
                </button>
              )}
              <button
                onClick={onDelete}
                className="px-3 py-1 text-xs rounded-lg transition-glass hover-glass"
                style={{
                  border: "1px solid var(--vn-border)",
                  color: "var(--vn-text-tertiary)",
                }}
              >
                Delete
              </button>
            </>
          ) : (
            <button
              onClick={onDownload}
              className="px-3 py-1 text-xs rounded-lg text-white"
              style={{ background: "var(--vn-primary)" }}
            >
              Download
            </button>
          )}
        </div>
      </div>
    </div>
  );
}

function ModelTypeBadge({ type, name }: { type: string; name: string }) {
  const lowerName = name.toLowerCase();

  let label = type.toUpperCase();
  let colorClass =
    "bg-gray-100 text-gray-600 dark:bg-gray-700 dark:text-gray-400";

  if (type === "stt") {
    if (lowerName.includes("sensevoice")) {
      label = "SenseVoice";
      colorClass =
        "bg-purple-100 text-purple-700 dark:bg-purple-900 dark:text-purple-300";
    } else if (lowerName.includes("qwen") && lowerName.includes("asr")) {
      label = "Qwen-ASR";
      colorClass =
        "bg-blue-100 text-blue-700 dark:bg-blue-900 dark:text-blue-300";
    } else {
      label = "Whisper";
      colorClass =
        "bg-green-100 text-green-700 dark:bg-green-900 dark:text-green-300";
    }
  } else if (type === "llm") {
    label = "LLM";
    colorClass =
      "bg-orange-100 text-orange-700 dark:bg-orange-900 dark:text-orange-300";
  }

  return (
    <span className={`text-xs px-1.5 py-0.5 rounded ${colorClass}`}>
      {label}
    </span>
  );
}
