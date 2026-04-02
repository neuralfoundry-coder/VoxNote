import { useEffect, useState } from "react";
import { useProviderStore } from "../../stores/providerStore";
import { useModelStore } from "../../stores/modelStore";

/** Provider를 engine type별로 분류 */
const PROVIDER_ENGINE_MAP: Record<string, string> = {
  "whisper-local": "stt",
  "sensevoice-local": "stt",
  "qwen-asr-local": "stt",
  "llama-local": "llm",
  "openai": "llm",
  "anthropic": "llm",
  "gemini": "llm",
  "ollama": "llm",
};

/** 로컬 모델 provider 목록 (model_id 필요) */
const LOCAL_PROVIDERS = new Set([
  "whisper-local",
  "sensevoice-local",
  "qwen-asr-local",
  "llama-local",
]);

export function ProviderSettings() {
  const {
    configs,
    availableProviders,
    fetchConfigs,
    fetchAvailableProviders,
    setConfig,
    testProvider,
  } = useProviderStore();
  const [testResult, setTestResult] = useState<string | null>(null);
  const models = useModelStore((s) => s.models);
  const fetchModels = useModelStore((s) => s.fetchModels);

  useEffect(() => {
    fetchConfigs();
    fetchAvailableProviders();
    fetchModels();
  }, [fetchConfigs, fetchAvailableProviders, fetchModels]);

  const handleTest = async (provider: string) => {
    setTestResult("Testing...");
    try {
      const result = await testProvider(provider);
      setTestResult(result);
    } catch (err) {
      setTestResult(`Error: ${err}`);
    }
  };

  const handleProviderChange = async (engine: string, provider: string) => {
    if (!provider) return;

    // 로컬 모델 provider인 경우 활성 모델 ID 자동 매칭
    let modelId: string | null = null;
    if (LOCAL_PROVIDERS.has(provider)) {
      const activeModel = models.find(
        (m) => m.is_active && m.model_type === engine && m.is_downloaded
      );
      modelId = activeModel?.id ?? null;
    }

    await setConfig({
      engine_type: engine,
      provider,
      model_id: modelId,
      endpoint: null,
      is_active: true,
    });
    fetchConfigs();
  };

  const engineLabels: Record<string, string> = {
    stt: "Speech-to-Text",
    llm: "LLM (Summarization)",
    tts: "Text-to-Speech",
  };

  return (
    <div className="p-6 max-w-2xl">
      <h2 className="text-xl font-bold mb-6 text-gray-900 dark:text-gray-100">
        AI Providers
      </h2>

      {["stt", "llm"].map((engine) => {
        const config = configs.find((c) => c.engine_type === engine);
        const engineProviders = availableProviders.filter(
          (p) => PROVIDER_ENGINE_MAP[p] === engine
        );
        const downloadedModels = models.filter(
          (m) => m.model_type === engine && m.is_downloaded
        );
        const activeModel = models.find(
          (m) => m.model_type === engine && m.is_active
        );

        return (
          <section
            key={engine}
            className="mb-6 p-4 border border-gray-200 dark:border-gray-700 rounded-lg"
          >
            <h3 className="text-sm font-semibold text-gray-500 uppercase mb-3">
              {engineLabels[engine] ?? engine.toUpperCase()}
            </h3>

            {/* Provider 선택 */}
            <div className="flex items-center gap-3 mb-3">
              <label className="text-xs text-gray-500 w-16">Provider</label>
              <select
                value={config?.provider ?? ""}
                className="flex-1 px-2 py-1 text-sm border border-gray-300 dark:border-gray-600 rounded bg-white dark:bg-gray-800 text-gray-900 dark:text-gray-100"
                onChange={(e) => handleProviderChange(engine, e.target.value)}
              >
                <option value="">Select provider...</option>
                {engineProviders.map((p) => (
                  <option key={p} value={p}>
                    {p}
                  </option>
                ))}
              </select>
              {config && engine === "llm" && !LOCAL_PROVIDERS.has(config.provider) && (
                <button
                  onClick={() => handleTest(config.provider)}
                  className="px-3 py-1 text-xs rounded bg-green-500 text-white hover:bg-green-600"
                >
                  Test
                </button>
              )}
            </div>

            {/* 활성 모델 표시 */}
            {activeModel ? (
              <div className="flex items-center gap-2 px-3 py-2 bg-blue-50 dark:bg-blue-950 rounded text-xs">
                <span className="text-blue-600 dark:text-blue-400 font-medium">
                  Active Model:
                </span>
                <span className="text-gray-700 dark:text-gray-300">
                  {activeModel.name}
                </span>
                <span className="text-gray-400">({activeModel.size_display})</span>
              </div>
            ) : downloadedModels.length > 0 ? (
              <div className="px-3 py-2 bg-yellow-50 dark:bg-yellow-950 rounded text-xs text-yellow-700 dark:text-yellow-400">
                {downloadedModels.length} model(s) downloaded — activate one from
                Model Manager
              </div>
            ) : (
              <div className="px-3 py-2 bg-gray-50 dark:bg-gray-800 rounded text-xs text-gray-400">
                No models downloaded — download from Model Manager
              </div>
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
