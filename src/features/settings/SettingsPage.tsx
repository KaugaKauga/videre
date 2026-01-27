import { Palette, Moon, Sun } from "lucide-react";
import {
  getStoredTheme,
  getStoredMode,
  setTheme,
  setMode,
  type ThemeName,
  type Mode,
} from "../../lib/theme";
import { useState, useEffect } from "react";

const themes: { id: ThemeName; name: string; description: string }[] = [
  {
    id: "amethyst-haze",
    name: "Amethyst Haze",
    description: "Purple-tinted elegance",
  },
  { id: "solar-dusk", name: "Solar Dusk", description: "Warm sunset tones" },
  { id: "nature", name: "Nature", description: "Fresh green palette" },
];

const modes: {
  id: Mode;
  name: string;
  description: string;
  icon: typeof Moon;
}[] = [
  { id: "light", name: "Light", description: "Bright and clear", icon: Sun },
  { id: "dark", name: "Dark", description: "Easy on the eyes", icon: Moon },
];

export function SettingsPage() {
  const [currentTheme, setCurrentTheme] = useState<ThemeName>("amethyst-haze");
  const [currentMode, setCurrentMode] = useState<Mode>("light");

  useEffect(() => {
    const theme = getStoredTheme();
    const mode = getStoredMode();
    setCurrentTheme(theme);
    setCurrentMode(mode);
  }, []);

  const handleThemeChange = (themeId: ThemeName) => {
    setTheme(themeId);
    setCurrentTheme(themeId);
  };

  const handleModeChange = (modeId: Mode) => {
    setMode(modeId);
    setCurrentMode(modeId);
  };

  return (
    <div className="flex-1 h-full p-6 overflow-auto min-h-0">
      <div className="max-w-4xl mx-auto">
        <div className="mb-8">
          <h2 className="text-2xl font-semibold text-foreground mb-2">
            Settings
          </h2>
          <p className="text-sm text-muted-foreground">
            Customize your Videre experience
          </p>
        </div>

        {/* Appearance Section */}
        <div className="bg-card border border-border rounded-lg p-6 mb-6">
          <div className="flex items-center gap-2 mb-4">
            <Palette className="w-5 h-5 text-foreground" />
            <h3 className="text-lg font-semibold text-foreground">
              Appearance
            </h3>
          </div>

          {/* Mode Selection */}
          <div className="mb-8">
            <p className="text-sm text-muted-foreground mb-4">
              Choose between light and dark mode
            </p>
            <div className="grid grid-cols-2 gap-4">
              {modes.map((mode) => {
                const isActive = currentMode === mode.id;
                const Icon = mode.icon;
                return (
                  <button
                    key={mode.id}
                    onClick={() => handleModeChange(mode.id)}
                    className={`
                      relative p-4 rounded-lg border-2 text-left transition-all
                      ${
                        isActive
                          ? "border-primary bg-primary/10"
                          : "border-border bg-card hover:border-primary/50 hover:bg-accent/50"
                      }
                    `}
                  >
                    <div className="flex items-center gap-3">
                      <div
                        className={`p-2 rounded-md ${isActive ? "bg-primary text-primary-foreground" : "bg-muted text-muted-foreground"}`}
                      >
                        <Icon className="w-5 h-5" />
                      </div>
                      <div>
                        <h4 className="font-medium text-foreground mb-0.5">
                          {mode.name}
                        </h4>
                        <p className="text-xs text-muted-foreground">
                          {mode.description}
                        </p>
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>
          </div>

          {/* Theme Selection */}
          <div>
            <p className="text-sm text-muted-foreground mb-4">
              Choose a color theme
            </p>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              {themes.map((theme) => {
                const isActive = currentTheme === theme.id;
                return (
                  <button
                    key={theme.id}
                    onClick={() => handleThemeChange(theme.id)}
                    className={`
                      relative p-4 rounded-lg border-2 text-left transition-all
                      ${
                        isActive
                          ? "border-primary bg-primary/10"
                          : "border-border bg-card hover:border-primary/50 hover:bg-accent/50"
                      }
                    `}
                  >
                    <div className="flex flex-col">
                      <div className="flex items-start justify-between mb-3">
                        <div>
                          <h4 className="font-medium text-foreground mb-1">
                            {theme.name}
                          </h4>
                          <p className="text-xs text-muted-foreground">
                            {theme.description}
                          </p>
                        </div>
                        {isActive && (
                          <div className="flex items-center justify-center w-5 h-5 rounded-full bg-primary">
                            <div className="w-2 h-2 rounded-full bg-primary-foreground" />
                          </div>
                        )}
                      </div>

                      {/* Theme Preview Colors */}
                      <div className="flex gap-2">
                        <div
                          className={`flex-1 h-8 rounded border border-border ${getThemePreviewClass(theme.id, "primary")}`}
                        />
                        <div
                          className={`flex-1 h-8 rounded border border-border ${getThemePreviewClass(theme.id, "accent")}`}
                        />
                        <div
                          className={`flex-1 h-8 rounded border border-border ${getThemePreviewClass(theme.id, "muted")}`}
                        />
                      </div>
                    </div>
                  </button>
                );
              })}
            </div>
          </div>
        </div>

        {/* About Section */}
        <div className="bg-card border border-border rounded-lg p-6">
          <h3 className="text-lg font-semibold text-foreground mb-4">About</h3>
          <div className="space-y-2 text-sm">
            <div className="flex justify-between">
              <span className="text-muted-foreground">Version</span>
              <span className="text-foreground font-medium">0.1.0</span>
            </div>
            <div className="flex justify-between">
              <span className="text-muted-foreground">Database Inspector</span>
              <span className="text-foreground font-medium">Videre</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}

function getThemePreviewClass(
  themeId: ThemeName,
  type: "primary" | "accent" | "muted",
): string {
  const previews: Record<ThemeName, Record<string, string>> = {
    "amethyst-haze": {
      primary: "bg-purple-600",
      accent: "bg-pink-400",
      muted: "bg-purple-100",
    },
    "solar-dusk": {
      primary: "bg-orange-600",
      accent: "bg-yellow-400",
      muted: "bg-orange-100",
    },
    nature: {
      primary: "bg-green-600",
      accent: "bg-green-400",
      muted: "bg-green-100",
    },
  };

  return previews[themeId]?.[type] || "bg-gray-200";
}
