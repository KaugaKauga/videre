import { Moon, Sun } from "lucide-react";
import { useEffect, useState } from "react";
import { getStoredMode, toggleMode, type Mode } from "../lib/theme";

export function ThemeToggle() {
  const [currentMode, setCurrentMode] = useState<Mode>("light");

  useEffect(() => {
    const mode = getStoredMode();
    setCurrentMode(mode);
  }, []);

  const handleToggle = () => {
    toggleMode();
    const newMode = currentMode === "dark" ? "light" : "dark";
    setCurrentMode(newMode);
  };

  return (
    <button
      onClick={handleToggle}
      className="p-2 rounded-md hover:bg-accent hover:text-accent-foreground transition-colors"
      aria-label={`Switch to ${currentMode === "dark" ? "light" : "dark"} mode`}
      title={`Switch to ${currentMode === "dark" ? "light" : "dark"} mode`}
    >
      {currentMode === "dark" ? (
        <Sun className="w-5 h-5" />
      ) : (
        <Moon className="w-5 h-5" />
      )}
    </button>
  );
}
