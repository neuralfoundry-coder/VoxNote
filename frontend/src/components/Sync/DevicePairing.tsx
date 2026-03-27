import { useState } from "react";

export function DevicePairing() {
  const [pairingCode, setPairingCode] = useState<string | null>(null);
  const [inputCode, setInputCode] = useState("");

  const generateCode = () => {
    // 6자리 페어링 코드 생성
    const code = Math.floor(100000 + Math.random() * 900000).toString();
    setPairingCode(code);
  };

  const handlePair = () => {
    if (inputCode.length === 6) {
      // TODO: 페어링 코드로 디바이스 연결
      console.log("Pairing with code:", inputCode);
    }
  };

  return (
    <div className="p-6 max-w-md">
      <h3 className="text-lg font-bold mb-4 text-gray-900 dark:text-gray-100">
        Pair New Device
      </h3>

      {/* Show pairing code */}
      <section className="mb-8">
        <h4 className="text-sm font-semibold text-gray-500 mb-2">
          Show Code on This Device
        </h4>
        <button
          onClick={generateCode}
          className="px-4 py-2 text-sm rounded-lg bg-blue-500 text-white hover:bg-blue-600"
        >
          Generate Pairing Code
        </button>
        {pairingCode && (
          <div className="mt-4 p-6 bg-gray-100 dark:bg-gray-800 rounded-lg text-center">
            <p className="text-3xl font-mono font-bold tracking-widest text-gray-900 dark:text-gray-100">
              {pairingCode}
            </p>
            <p className="text-xs text-gray-500 mt-2">
              Enter this code on your other device
            </p>
          </div>
        )}
      </section>

      {/* Enter pairing code */}
      <section>
        <h4 className="text-sm font-semibold text-gray-500 mb-2">
          Enter Code from Another Device
        </h4>
        <div className="flex gap-2">
          <input
            type="text"
            value={inputCode}
            onChange={(e) => setInputCode(e.target.value.replace(/\D/g, "").slice(0, 6))}
            placeholder="000000"
            maxLength={6}
            className="flex-1 px-3 py-2 text-center text-lg font-mono border border-gray-300 dark:border-gray-600 rounded-lg bg-white dark:bg-gray-800 tracking-widest"
          />
          <button
            onClick={handlePair}
            disabled={inputCode.length !== 6}
            className="px-4 py-2 text-sm rounded-lg bg-green-500 text-white hover:bg-green-600 disabled:opacity-50"
          >
            Pair
          </button>
        </div>
      </section>
    </div>
  );
}
