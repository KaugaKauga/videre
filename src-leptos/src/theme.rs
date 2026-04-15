//! Theme & mode management for Videre.
//!
//! Mirrors the React `lib/theme.ts` — reads/writes localStorage and toggles
//! CSS classes on `<html>` to switch between themes and light/dark mode.



// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Available color themes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeName {
    AmethystHaze,
    SolarDusk,
    Nature,
}

impl ThemeName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AmethystHaze => "amethyst-haze",
            Self::SolarDusk => "solar-dusk",
            Self::Nature => "nature",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "solar-dusk" => Self::SolarDusk,
            "nature" => Self::Nature,
            _ => Self::AmethystHaze,
        }
    }

    pub const ALL: [ThemeName; 3] = [Self::AmethystHaze, Self::SolarDusk, Self::Nature];

    pub fn display_name(self) -> &'static str {
        match self {
            Self::AmethystHaze => "Amethyst Haze",
            Self::SolarDusk => "Solar Dusk",
            Self::Nature => "Nature",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::AmethystHaze => "Purple-tinted elegance",
            Self::SolarDusk => "Warm sunset tones",
            Self::Nature => "Fresh green palette",
        }
    }
}

/// Light or dark mode.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Light,
    Dark,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "dark" => Self::Dark,
            _ => Self::Light,
        }
    }

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Light => "Light",
            Self::Dark => "Dark",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Light => "Bright and clear",
            Self::Dark => "Easy on the eyes",
        }
    }
}

// ---------------------------------------------------------------------------
// localStorage keys
// ---------------------------------------------------------------------------

const THEME_KEY: &str = "videre-theme";
const MODE_KEY: &str = "videre-mode";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn local_storage() -> Option<web_sys::Storage> {
    web_sys::window()?.local_storage().ok()?
}

fn document_element() -> Option<web_sys::Element> {
    web_sys::window()?.document()?.document_element()
}

// ---------------------------------------------------------------------------
// Read / write
// ---------------------------------------------------------------------------

/// Read the stored theme from localStorage (defaults to AmethystHaze).
pub fn get_stored_theme() -> ThemeName {
    local_storage()
        .and_then(|s| s.get_item(THEME_KEY).ok().flatten())
        .map(|v| ThemeName::from_str(&v))
        .unwrap_or(ThemeName::AmethystHaze)
}

/// Read the stored mode from localStorage (defaults to Light).
pub fn get_stored_mode() -> Mode {
    local_storage()
        .and_then(|s| s.get_item(MODE_KEY).ok().flatten())
        .map(|v| Mode::from_str(&v))
        .unwrap_or(Mode::Light)
}

/// Write theme to localStorage.
fn set_stored_theme(theme: ThemeName) {
    if let Some(s) = local_storage() {
        let _ = s.set_item(THEME_KEY, theme.as_str());
    }
}

/// Write mode to localStorage.
fn set_stored_mode(mode: Mode) {
    if let Some(s) = local_storage() {
        let _ = s.set_item(MODE_KEY, mode.as_str());
    }
}

// ---------------------------------------------------------------------------
// Apply
// ---------------------------------------------------------------------------

/// Apply theme + mode by toggling CSS classes on `<html>`.
pub fn apply_theme(theme: ThemeName, mode: Mode) {
    let Some(el) = document_element() else {
        return;
    };
    let cl = el.class_list();

    // Remove all theme classes
    for t in ThemeName::ALL {
        let _ = cl.remove_1(t.as_str());
    }
    // Remove dark class
    let _ = cl.remove_1("dark");

    // Apply theme
    let _ = cl.add_1(theme.as_str());

    // Apply dark mode
    if mode == Mode::Dark {
        let _ = cl.add_1("dark");
    }
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Call once at startup (before mount) to apply stored preferences.
pub fn initialize_theme() {
    let theme = get_stored_theme();
    let mode = get_stored_mode();
    apply_theme(theme, mode);
}

/// Change and persist the theme (keeps current mode).
pub fn set_theme(theme: ThemeName) {
    let mode = get_stored_mode();
    set_stored_theme(theme);
    apply_theme(theme, mode);
}

/// Change and persist the mode (keeps current theme).
pub fn set_mode(mode: Mode) {
    let theme = get_stored_theme();
    set_stored_mode(mode);
    apply_theme(theme, mode);
}
