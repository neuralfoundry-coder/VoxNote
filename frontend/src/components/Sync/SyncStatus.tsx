import { useSyncStore } from "../../stores/syncStore";

export function SyncStatus() {
  const { status, pairedDevices, pendingDeltas, errorMessage } = useSyncStore();

  const statusColors: Record<string, string> = {
    connected: "bg-green-500",
    connecting: "bg-yellow-500 animate-pulse",
    disconnected: "bg-gray-400",
    error: "bg-red-500",
  };

  return (
    <div className="p-4">
      <div className="flex items-center gap-2 mb-4">
        <span className={`w-2 h-2 rounded-full ${statusColors[status]}`} />
        <span className="text-sm text-gray-700 dark:text-gray-300 capitalize">
          {status}
        </span>
        {pendingDeltas > 0 && (
          <span className="text-xs text-gray-500">
            ({pendingDeltas} pending)
          </span>
        )}
      </div>

      {errorMessage && (
        <p className="text-xs text-red-500 mb-3">{errorMessage}</p>
      )}

      <h4 className="text-xs font-semibold text-gray-500 uppercase mb-2">
        Paired Devices ({pairedDevices.length})
      </h4>

      {pairedDevices.length === 0 ? (
        <p className="text-xs text-gray-400">No paired devices</p>
      ) : (
        <div className="space-y-2">
          {pairedDevices.map((device) => (
            <div
              key={device.id}
              className="flex items-center justify-between p-2 bg-gray-50 dark:bg-gray-800 rounded"
            >
              <div>
                <p className="text-sm text-gray-700 dark:text-gray-300">
                  {device.name}
                </p>
                <p className="text-xs text-gray-400">
                  Last seen: {new Date(device.lastSeen).toLocaleString()}
                </p>
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
