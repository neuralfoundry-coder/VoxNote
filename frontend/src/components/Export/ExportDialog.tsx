import { useState } from "react";
import { tauriInvoke } from "../../hooks/useTauriIPC";

interface Props {
  noteId: string;
  onClose: () => void;
}

const FORMATS = [
  { id: "markdown", label: "Markdown (.md)" },
  { id: "pdf", label: "PDF (.pdf)" },
  { id: "docx", label: "DOCX (.docx)" },
];

export function ExportDialog({ noteId, onClose }: Props) {
  const [format, setFormat] = useState("markdown");
  const [isExporting, setIsExporting] = useState(false);

  const handleExport = async () => {
    setIsExporting(true);
    try {
      const data = await tauriInvoke<number[]>("export_note", {
        noteId,
        format,
      });
      // 브라우저 다운로드로 처리
      const blob = new Blob([new Uint8Array(data)], {
        type: format === "markdown" ? "text/markdown" : "application/octet-stream",
      });
      const url = URL.createObjectURL(blob);
      const a = document.createElement("a");
      a.href = url;
      a.download = `note.${format === "markdown" ? "md" : format}`;
      a.click();
      URL.revokeObjectURL(url);
      onClose();
    } catch (err) {
      console.error("Export failed:", err);
    } finally {
      setIsExporting(false);
    }
  };

  return (
    <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
      <div className="bg-white dark:bg-gray-800 rounded-lg p-6 w-96 shadow-xl">
        <h3 className="text-lg font-bold mb-4 text-gray-900 dark:text-gray-100">
          Export Note
        </h3>

        <div className="space-y-2 mb-6">
          {FORMATS.map((f) => (
            <label key={f.id} className="flex items-center gap-2 cursor-pointer">
              <input
                type="radio"
                name="format"
                value={f.id}
                checked={format === f.id}
                onChange={() => setFormat(f.id)}
                className="text-blue-500"
              />
              <span className="text-sm text-gray-700 dark:text-gray-300">
                {f.label}
              </span>
            </label>
          ))}
        </div>

        <div className="flex justify-end gap-2">
          <button
            onClick={onClose}
            className="px-4 py-2 text-sm rounded-lg bg-gray-100 dark:bg-gray-700 text-gray-700 dark:text-gray-300"
          >
            Cancel
          </button>
          <button
            onClick={handleExport}
            disabled={isExporting}
            className="px-4 py-2 text-sm font-medium rounded-lg bg-blue-500 text-white hover:bg-blue-600 disabled:opacity-50"
          >
            {isExporting ? "Exporting..." : "Export"}
          </button>
        </div>
      </div>
    </div>
  );
}
