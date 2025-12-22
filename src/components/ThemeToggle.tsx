import { Moon, Sun } from "lucide-react";
import { useEffect, useState } from "react";
import { getStoredTheme, setTheme, watchSystemTheme } from "../lib/theme";

export function ThemeToggle() {
  const [currentTheme, setCurrentTheme] = useState<"light" | "dark">("light");

  useEffect(() => {
    // Get initial theme
    const storedTheme = getStoredTheme();
    const effectiveTheme =
      storedTheme === "system"
        ? window.matchMedia("(prefers-color-scheme: dark)").matches
          ? "dark"
          : "light"
        : storedTheme;
    setCurrentTheme(effectiveTheme);

    // Watch for system theme changes
    const cleanup = watchSystemTheme((theme) => {
      setCurrentTheme(theme);
    });

    return cleanup;
  }, []);

  const handleToggle = () => {
    const newTheme = currentTheme === "dark" ? "light" : "dark";
    setTheme(newTheme);
    setCurrentTheme(newTheme);
  };

  return (
    <button
      onClick={handleToggle}
      className="p-2 rounded-md hover:bg-accent hover:text-accent-foreground transition-colors"
      aria-label={`Switch to ${currentTheme === "dark" ? "light" : "dark"} mode`}
      title={`Switch to ${currentTheme === "dark" ? "light" : "dark"} mode`}
    >
      {currentTheme === "dark" ? (
        <Sun className="w-5 h-5" />
      ) : (
        <Moon className="w-5 h-5" />
      )}
    </button>
  );
}
