import { create } from "zustand";

type Theme = "light" | "dark" | "system";

interface ThemeState {
  theme: Theme;
  resolved: "light" | "dark";
  setTheme: (theme: Theme) => void;
}

function getSystemTheme(): "light" | "dark" {
  if (typeof window !== "undefined" && window.matchMedia("(prefers-color-scheme: dark)").matches) {
    return "dark";
  }
  return "light";
}

function applyTheme(resolved: "light" | "dark") {
  document.documentElement.classList.toggle("dark", resolved === "dark");
}

export const useThemeStore = create<ThemeState>((set) => {
  const saved = (typeof localStorage !== "undefined" && localStorage.getItem("vn-theme")) as Theme | null;
  const theme = saved || "system";
  const resolved = theme === "system" ? getSystemTheme() : theme;
  // Apply on load
  if (typeof document !== "undefined") applyTheme(resolved);

  return {
    theme,
    resolved,
    setTheme: (theme) => {
      const resolved = theme === "system" ? getSystemTheme() : theme;
      localStorage.setItem("vn-theme", theme);
      applyTheme(resolved);
      set({ theme, resolved });
    },
  };
});
