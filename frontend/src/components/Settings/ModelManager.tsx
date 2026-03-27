import { useEffect, useState } from "react";
import { tauriInvoke, useTauriEvent } from "../../hooks/useTauriIPC";
import type { DownloadProgress, ModelInfo } from "../../lib/types";

export function ModelManager() {
  const [models, setModels] = useState<ModelInfo[]>([]);
  const [downloading, setDownloading] = useState<Record<string, number>>({});

  useEffect(() => {
    tauriInvoke<ModelInfo[]>("list_models").then(setModels).catch(console.error);
  }, []);

  useTauriEvent<DownloadProgress>("model:download_progress", (progress) => {
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
      // Refresh model list
      tauriInvoke<ModelInfo[]>("list_models").then(setModels);
    }
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
      setModels((prev) =>
        prev.map((m) =>
          m.id === modelId ? { ...m, is_downloaded: false } : m
        )
      );
    } catch (err) {
      console.error("Delete failed:", err);
    }
  };

  return (
    <div className="p-6 max-w-2xl">
      <h2 className="text-xl font-bold mb-6 text-gray-900 dark:text-gray-100">
        Model Manager
      </h2>

      <div className="space-y-3">
        {models.map((model) => (
          <div
            key={model.id}
            className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg"
          >
            <div className="flex items-center justify-between">
              <div>
                <h3 className="text-sm font-medium text-gray-900 dark:text-gray-100">
                  {model.name}
                </h3>
                <p className="text-xs text-gray-500 mt-1">
                  {model.size_display} · {model.model_type}
                  {model.gpu_recommended && " · GPU recommended"}
                </p>
                {model.description && (
                  <p className="text-xs text-gray-400 mt-1">
                    {model.description}
                  </p>
                )}
              </div>

              <div>
                {downloading[model.id] != null ? (
                  <div className="w-32">
                    <div className="h-2 bg-gray-200 rounded-full overflow-hidden">
                      <div
                        className="h-full bg-blue-500 transition-all"
                        style={{ width: `${downloading[model.id]}%` }}
                      />
                    </div>
                    <span className="text-xs text-gray-500 mt-1">
                      {Math.round(downloading[model.id])}%
                    </span>
                  </div>
                ) : model.is_downloaded ? (
                  <button
                    onClick={() => handleDelete(model.id)}
                    className="px-3 py-1 text-xs rounded bg-gray-100 dark:bg-gray-700 text-gray-600 dark:text-gray-400 hover:bg-red-100 hover:text-red-600"
                  >
                    Delete
                  </button>
                ) : (
                  <button
                    onClick={() => handleDownload(model.id)}
                    className="px-3 py-1 text-xs rounded bg-blue-500 text-white hover:bg-blue-600"
                  >
                    Download
                  </button>
                )}
              </div>
            </div>
          </div>
        ))}

        {models.length === 0 && (
          <p className="text-sm text-gray-400 text-center py-8">
            No models available. Check registry.toml.
          </p>
        )}
      </div>
    </div>
  );
}
