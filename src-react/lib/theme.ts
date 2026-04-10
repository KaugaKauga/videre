/**
 * Theme utilities for managing themes and dark/light mode
 */

export type ThemeName = "amethyst-haze" | "solar-dusk" | "nature";
export type Mode = "light" | "dark";

const THEME_STORAGE_KEY = "videre-theme";
const MODE_STORAGE_KEY = "videre-mode";

/**
 * Gets the current theme preference from localStorage
 */
export function getStoredTheme(): ThemeName {
  if (typeof window === "undefined") return "amethyst-haze";

  const stored = localStorage.getItem(THEME_STORAGE_KEY);
  if (
    stored === "amethyst-haze" ||
    stored === "solar-dusk" ||
    stored === "nature"
  ) {
    return stored;
  }
  return "amethyst-haze";
}

/**
 * Gets the current mode preference from localStorage
 */
export function getStoredMode(): Mode {
  if (typeof window === "undefined") return "light";

  const stored = localStorage.getItem(MODE_STORAGE_KEY);
  if (stored === "light" || stored === "dark") {
    return stored;
  }

  // Check system preference as fallback
  return getSystemMode();
}

/**
 * Saves theme preference to localStorage
 */
export function setStoredTheme(theme: ThemeName): void {
  if (typeof window === "undefined") return;
  localStorage.setItem(THEME_STORAGE_KEY, theme);
}

/**
 * Saves mode preference to localStorage
 */
export function setStoredMode(mode: Mode): void {
  if (typeof window === "undefined") return;
  localStorage.setItem(MODE_STORAGE_KEY, mode);
}

/**
 * Gets the system's preferred color scheme
 */
export function getSystemMode(): Mode {
  if (typeof window === "undefined") return "light";

  return window.matchMedia("(prefers-color-scheme: dark)").matches
    ? "dark"
    : "light";
}

/**
 * Applies the theme and mode to the document
 */
export function applyTheme(theme: ThemeName, mode: Mode): void {
  if (typeof window === "undefined") return;

  const root = document.documentElement;

  // Remove all theme classes
  root.classList.remove("amethyst-haze", "solar-dusk", "nature");

  // Remove dark mode class
  root.classList.remove("dark");

  // Apply the selected theme
  root.classList.add(theme);

  // Apply dark mode if selected
  if (mode === "dark") {
    root.classList.add("dark");
  }
}

/**
 * Initializes theme on app load
 */
export function initializeTheme(): void {
  const theme = getStoredTheme();
  const mode = getStoredMode();
  applyTheme(theme, mode);
}

/**
 * Sets and applies a new theme (keeps current mode)
 */
export function setTheme(theme: ThemeName): void {
  const currentMode = getStoredMode();
  setStoredTheme(theme);
  applyTheme(theme, currentMode);
}

/**
 * Sets and applies a new mode (keeps current theme)
 */
export function setMode(mode: Mode): void {
  const currentTheme = getStoredTheme();
  setStoredMode(mode);
  applyTheme(currentTheme, mode);
}

/**
 * Toggles between light and dark mode
 */
export function toggleMode(): void {
  const currentMode = getStoredMode();
  const newMode: Mode = currentMode === "dark" ? "light" : "dark";
  setMode(newMode);
}

/**
 * Listens for system theme changes
 */
export function watchSystemMode(callback: (mode: Mode) => void): () => void {
  if (typeof window === "undefined") return () => {};

  const mediaQuery = window.matchMedia("(prefers-color-scheme: dark)");

  const handler = (e: MediaQueryListEvent) => {
    const mode = e.matches ? "dark" : "light";
    callback(mode);
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
