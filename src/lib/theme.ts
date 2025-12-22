/**
 * Theme utilities for managing light/dark mode
 */

export type Theme = "light" | "dark" | "system";

const THEME_STORAGE_KEY = "daedalus-theme";

/**
 * Gets the current theme preference from localStorage
 */
export function getStoredTheme(): Theme {
  if (typeof window === "undefined") return "system";

  const stored = localStorage.getItem(THEME_STORAGE_KEY);
  if (stored === "light" || stored === "dark" || stored === "system") {
    return stored;
  }
  return "system";
}

/**
 * Saves theme preference to localStorage
 */
export function setStoredTheme(theme: Theme): void {
  if (typeof window === "undefined") return;
  localStorage.setItem(THEME_STORAGE_KEY, theme);
}

/**
 * Gets the system's preferred color scheme
 */
export function getSystemTheme(): "light" | "dark" {
  if (typeof window === "undefined") return "light";

  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

/**
 * Applies the theme to the document
 */
export function applyTheme(theme: Theme): void {
  if (typeof window === "undefined") return;

  const root = document.documentElement;
  const effectiveTheme = theme === "system" ? getSystemTheme() : theme;

  if (effectiveTheme === "dark") {
    root.classList.add("dark");
  } else {
    root.classList.remove("dark");
  }
}

/**
 * Initializes theme on app load
 */
export function initializeTheme(): void {
  const theme = getStoredTheme();
  applyTheme(theme);
}

/**
 * Sets and applies a new theme
 */
export function setTheme(theme: Theme): void {
  setStoredTheme(theme);
  applyTheme(theme);
}

/**
 * Toggles between light and dark theme
 */
export function toggleTheme(): void {
  const current = getStoredTheme();
  const effectiveTheme = current === "system" ? getSystemTheme() : current;
  const newTheme: Theme = effectiveTheme === "dark" ? "light" : "dark";
  setTheme(newTheme);
}

/**
 * Listens for system theme changes (when theme is set to "system")
 */
export function watchSystemTheme(callback: (theme: "light" | "dark") => void): () => void {
  if (typeof window === "undefined") return () => {};

  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

  const handler = (e: MediaQueryListEvent) => {
    const currentTheme = getStoredTheme();
    if (currentTheme === "system") {
      const theme = e.matches ? "dark" : "light";
      applyTheme(theme);
      callback(theme);
    }
  };

  // Modern browsers
  if (mediaQuery.addEventListener) {
    mediaQuery.addEventListener("change", handler);
    return () => mediaQuery.removeEventListener("change", handler);
  }
  // Fallback for older browsers
  else {
    mediaQuery.addListener(handler);
    return () => mediaQuery.removeListener(handler);
  }
}
