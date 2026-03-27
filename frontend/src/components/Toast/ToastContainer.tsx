import { useToastStore, type ToastType } from "../../stores/toastStore";

const ICONS: Record<ToastType, string> = {
  success: "M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z",
  error: "M10 14l2-2m0 0l2-2m-2 2l-2-2m2 2l2 2m7-2a9 9 0 11-18 0 9 9 0 0118 0z",
  info: "M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z",
  warning: "M12 9v2m0 4h.01m-6.938 4h13.856c1.54 0 2.502-1.667 1.732-2.5L13.732 4c-.77-.833-1.964-.833-2.732 0L4.082 16.5c-.77.833.192 2.5 1.732 2.5z",
};

const COLORS: Record<ToastType, string> = {
  success: "#22c55e",
  error: "#ef4444",
  info: "#6366f1",
  warning: "#f59e0b",
};

export function ToastContainer() {
  const { toasts, removeToast } = useToastStore();

  if (toasts.length === 0) return null;

  return (
    <div className="fixed bottom-4 right-4 z-[100] flex flex-col gap-2 max-w-[360px]">
      {toasts.map((toast) => (
        <div
          key={toast.id}
          className="glass-elevated rounded-xl px-4 py-3 flex items-start gap-3 animate-float-in cursor-pointer"
          onClick={() => removeToast(toast.id)}
          role="alert"
        >
          <svg
            width="18"
            height="18"
            viewBox="0 0 24 24"
            fill="none"
            stroke={COLORS[toast.type]}
            strokeWidth="2"
            strokeLinecap="round"
            strokeLinejoin="round"
            className="shrink-0 mt-0.5"
          >
            <path d={ICONS[toast.type]} />
          </svg>
          <p className="text-[13px] leading-snug" style={{ color: "var(--vn-text-primary)" }}>
            {toast.message}
          </p>
        </div>
      ))}
    </div>
  );
}
