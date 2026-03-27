import { useState } from "react";
import { RecorderBar } from "../Recorder/RecorderBar";

type MobileTab = "notes" | "record" | "settings";

export function MobileLayout() {
  const [activeTab, setActiveTab] = useState<MobileTab>("notes");

  return (
    <div className="flex flex-col h-screen md:hidden">
      {/* Content */}
      <main className="flex-1 overflow-auto">
        {activeTab === "notes" && (
          <div className="p-4">
            <h1 className="text-xl font-bold mb-4">VoxNote</h1>
            <p className="text-gray-500">Select a note or start recording</p>
          </div>
        )}
        {activeTab === "record" && (
          <div className="p-4 flex items-center justify-center h-full">
            <RecorderBar />
          </div>
        )}
        {activeTab === "settings" && (
          <div className="p-4">
            <h1 className="text-xl font-bold mb-4">Settings</h1>
          </div>
        )}
      </main>

      {/* Bottom Tab Bar */}
      <nav className="border-t border-gray-200 dark:border-gray-700 bg-white dark:bg-gray-800 flex">
        {(["notes", "record", "settings"] as MobileTab[]).map((tab) => (
          <button
            key={tab}
            onClick={() => setActiveTab(tab)}
            className={`flex-1 py-3 text-xs font-medium capitalize ${
              activeTab === tab
                ? "text-blue-500"
                : "text-gray-500"
            }`}
          >
            {tab}
          </button>
        ))}
      </nav>
    </div>
  );
}
