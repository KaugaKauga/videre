import { useEffect } from "react";

interface KeyboardShortcut {
  key: string;
  ctrlOrCmd?: boolean;
  shift?: boolean;
  alt?: boolean;
  handler: (event: KeyboardEvent) => void;
  preventDefault?: boolean;
}

/**
 * Detects if the user is on macOS
 */
const isMac = () => {
  if (typeof window === "undefined") return false;
  return navigator.platform.toUpperCase().indexOf("MAC") >= 0;
};

/**
 * Custom hook for managing keyboard shortcuts
 */
export function useKeyboardShortcuts(shortcuts: KeyboardShortcut[]) {
  useEffect(() => {
    const handleKeyDown = (event: KeyboardEvent) => {
      for (const shortcut of shortcuts) {
        const {
          key,
          ctrlOrCmd = false,
          shift = false,
          alt = false,
          handler,
          preventDefault = true,
        } = shortcut;

        // Check if the key matches
        const keyMatch = event.key.toLowerCase() === key.toLowerCase();

        // Check modifier keys
        const modifierKey = isMac() ? event.metaKey : event.ctrlKey;
        const ctrlOrCmdMatch = ctrlOrCmd ? modifierKey : !modifierKey;
        const shiftMatch = shift ? event.shiftKey : !event.shiftKey;
        const altMatch = alt ? event.altKey : !event.altKey;

        // If all conditions match, execute the handler
        if (keyMatch && ctrlOrCmdMatch && shiftMatch && altMatch) {
          if (preventDefault) {
            event.preventDefault();
          }
          handler(event);
          break; // Stop after first match
        }
      }
    };

    window.addEventListener("keydown", handleKeyDown);

    return () => {
      window.removeEventListener("keydown", handleKeyDown);
    };
  }, [shortcuts]);
}

/**
 * Helper to get the modifier key name based on platform
 */
export function getModifierKeyName(): string {
  return isMac() ? "⌘" : "Ctrl";
}
