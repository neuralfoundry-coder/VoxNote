import { useEffect } from "react";
import { useSettingsStore } from "../../stores/settingsStore";
// AppConfig type is used via settingsStore

export function SettingsPanel() {
  const { config, fetchSettings, updateSettings, isLoading } =
    useSettingsStore();

  useEffect(() => {
    fetchSettings();
  }, [fetchSettings]);

  if (isLoading || !config) {
    return <p className="p-4 text-gray-400">Loading settings...</p>;
  }

  const update = (path: string, value: unknown) => {
    const newConfig = structuredClone(config);
    const [section, key] = path.split(".");
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    (newConfig as any)[section][key] = value;
    updateSettings(newConfig);
  };

  return (
    <div className="p-6 max-w-2xl">
      <h2 className="text-xl font-bold mb-6 text-gray-900 dark:text-gray-100">
        Settings
      </h2>

      {/* Audio */}
      <section className="mb-8">
        <h3 className="text-sm font-semibold text-gray-500 uppercase mb-3">
          Audio
        </h3>
        <div className="space-y-4">
          <label className="block">
            <span className="text-sm text-gray-700 dark:text-gray-300">
              VAD Threshold
            </span>
            <input
              type="range"
              min="0"
              max="1"
              step="0.05"
              value={config.audio.vad_threshold}
              onChange={(e) =>
                update("audio.vad_threshold", parseFloat(e.target.value))
              }
              className="w-full mt-1"
            />
            <span className="text-xs text-gray-500">
              {config.audio.vad_threshold}
            </span>
          </label>
        </div>
      </section>

      {/* STT */}
      <section className="mb-8">
        <h3 className="text-sm font-semibold text-gray-500 uppercase mb-3">
          Speech to Text
        </h3>
        <div className="space-y-4">
          <label className="flex items-center gap-2">
            <input
              type="checkbox"
              checked={config.stt.use_gpu}
              onChange={(e) => update("stt.use_gpu", e.target.checked)}
              className="rounded"
            />
            <span className="text-sm text-gray-700 dark:text-gray-300">
              Use GPU acceleration
            </span>
          </label>
        </div>
      </section>

      {/* Storage */}
      <section className="mb-8">
        <h3 className="text-sm font-semibold text-gray-500 uppercase mb-3">
          Storage
        </h3>
        <label className="flex items-center gap-2">
          <input
            type="checkbox"
            checked={config.storage.encryption_enabled}
            onChange={(e) =>
              update("storage.encryption_enabled", e.target.checked)
            }
            className="rounded"
          />
          <span className="text-sm text-gray-700 dark:text-gray-300">
            Enable E2E encryption
          </span>
        </label>
      </section>
    </div>
  );
}
