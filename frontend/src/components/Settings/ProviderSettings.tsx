import { useEffect, useState } from "react";
import { useProviderStore } from "../../stores/providerStore";

export function ProviderSettings() {
  const { configs, availableProviders, fetchConfigs, fetchAvailableProviders, testProvider } =
    useProviderStore();
  const [testResult, setTestResult] = useState<string | null>(null);

  useEffect(() => {
    fetchConfigs();
    fetchAvailableProviders();
  }, [fetchConfigs, fetchAvailableProviders]);

  const handleTest = async (provider: string) => {
    try {
      const result = await testProvider(provider);
      setTestResult(result);
    } catch (err) {
      setTestResult(`Error: ${err}`);
    }
  };

  return (
    <div className="p-6 max-w-2xl">
      <h2 className="text-xl font-bold mb-6 text-gray-900 dark:text-gray-100">
        AI Providers
      </h2>

      {/* Engine-specific configs */}
      {["stt", "llm", "tts"].map((engine) => {
        const config = configs.find((c) => c.engine_type === engine);
        return (
          <section key={engine} className="mb-6 p-4 border border-gray-200 dark:border-gray-700 rounded-lg">
            <h3 className="text-sm font-semibold text-gray-500 uppercase mb-3">
              {engine.toUpperCase()} Engine
            </h3>
            <div className="flex items-center gap-3">
              <select
                value={config?.provider ?? ""}
                className="flex-1 px-2 py-1 text-sm border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800"
                onChange={() => {}}
              >
                <option value="">Select provider...</option>
                {availableProviders.map((p) => (
                  <option key={p} value={p}>{p}</option>
                ))}
              </select>
              <button
                onClick={() => config && handleTest(config.provider)}
                className="px-3 py-1 text-xs rounded bg-green-500 text-white hover:bg-green-600"
              >
                Test
              </button>
            </div>
            {config?.model_id && (
              <p className="text-xs text-gray-500 mt-2">
                Model: {config.model_id}
              </p>
            )}
          </section>
        );
      })}

      {testResult && (
        <div className="mt-4 p-3 text-sm bg-gray-100 dark:bg-gray-800 rounded">
          {testResult}
        </div>
      )}
    </div>
  );
}
