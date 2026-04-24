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
    Swiss,
    CassetteFuturism,
}

impl ThemeName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::AmethystHaze => "amethyst-haze",
            Self::SolarDusk => "solar-dusk",
            Self::Nature => "nature",
            Self::Swiss => "swiss",
            Self::CassetteFuturism => "cassette-futurism",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "solar-dusk" => Self::SolarDusk,
            "nature" => Self::Nature,
            "swiss" => Self::Swiss,
            "cassette-futurism" => Self::CassetteFuturism,
            _ => Self::AmethystHaze,
        }
    }

    pub const ALL: [ThemeName; 5] = [
        Self::AmethystHaze,
        Self::SolarDusk,
        Self::Nature,
        Self::Swiss,
        Self::CassetteFuturism,
    ];

    pub fn display_name(self) -> &'static str {
        match self {
            Self::AmethystHaze => "Amethyst Haze",
            Self::SolarDusk => "Solar Dusk",
            Self::Nature => "Nature",
            Self::Swiss => "Swiss",
            Self::CassetteFuturism => "Cassette Futurism",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::AmethystHaze => "Purple-tinted elegance",
            Self::SolarDusk => "Warm sunset tones",
            Self::Nature => "Fresh green palette",
            Self::Swiss => "Precision engineering",
            Self::CassetteFuturism => "Phosphor, scanlines, ALL CAPS",
        }
    }
}

/// UI font size scale.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FontSize {
    Small,
    Normal,
    Large,
    XLarge,
}

impl FontSize {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Small => "fs-small",
            Self::Normal => "fs-normal",
            Self::Large => "fs-large",
            Self::XLarge => "fs-xl",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "fs-small" => Self::Small,
            "fs-large" => Self::Large,
            "fs-xl" => Self::XLarge,
            _ => Self::Normal,
        }
    }

    pub const ALL: [FontSize; 4] = [Self::Small, Self::Normal, Self::Large, Self::XLarge];

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Small => "Small",
            Self::Normal => "Normal",
            Self::Large => "Large",
            Self::XLarge => "X-Large",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
            Self::Small => "Compact",
            Self::Normal => "Default",
            Self::Large => "Comfortable",
            Self::XLarge => "Accessible",
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
const FONT_SIZE_KEY: &str = "videre-font-size";

const FONT_SIZE_CLASSES: [&str; 3] = ["fs-small", "fs-large", "fs-xl"];

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

/// Read the stored font size from localStorage (defaults to Normal).
pub fn get_stored_font_size() -> FontSize {
    local_storage()
        .and_then(|s| s.get_item(FONT_SIZE_KEY).ok().flatten())
        .map(|v| FontSize::from_str(&v))
        .unwrap_or(FontSize::Normal)
}

/// Write font size to localStorage.
fn set_stored_font_size(size: FontSize) {
    if let Some(s) = local_storage() {
        let _ = s.set_item(FONT_SIZE_KEY, size.as_str());
    }
}

// ---------------------------------------------------------------------------
// Apply
// ---------------------------------------------------------------------------

/// Apply font size by toggling a CSS class on `<html>`.
pub fn apply_font_size(size: FontSize) {
    let Some(el) = document_element() else {
        return;
    };
    let cl = el.class_list();
    for cls in FONT_SIZE_CLASSES {
        let _ = cl.remove_1(cls);
    }
    if size != FontSize::Normal {
        let _ = cl.add_1(size.as_str());
    }
}

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
    apply_font_size(get_stored_font_size());
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

/// Change and persist the font size.
pub fn set_font_size(size: FontSize) {
    set_stored_font_size(size);
    apply_font_size(size);
}

#[cfg(test)]
mod tests {
    use super::*;

    // -- ThemeName ----------------------------------------------------------

    #[test]
    fn theme_as_str_returns_css_class() {
        assert_eq!(ThemeName::AmethystHaze.as_str(), "amethyst-haze");
        assert_eq!(ThemeName::SolarDusk.as_str(), "solar-dusk");
        assert_eq!(ThemeName::Nature.as_str(), "nature");
        assert_eq!(ThemeName::Swiss.as_str(), "swiss");
        assert_eq!(
            ThemeName::CassetteFuturism.as_str(),
            "cassette-futurism"
        );
    }

    #[test]
    fn theme_from_str_known_values() {
        assert_eq!(
            ThemeName::from_str("amethyst-haze"),
            ThemeName::AmethystHaze
        );
        assert_eq!(ThemeName::from_str("solar-dusk"), ThemeName::SolarDusk);
        assert_eq!(ThemeName::from_str("nature"), ThemeName::Nature);
        assert_eq!(ThemeName::from_str("swiss"), ThemeName::Swiss);
        assert_eq!(
            ThemeName::from_str("cassette-futurism"),
            ThemeName::CassetteFuturism
        );
    }

    #[test]
    fn theme_from_str_unknown_defaults_to_amethyst_haze() {
        assert_eq!(ThemeName::from_str(""), ThemeName::AmethystHaze);
        assert_eq!(ThemeName::from_str("neon-glow"), ThemeName::AmethystHaze);
        assert_eq!(ThemeName::from_str("NATURE"), ThemeName::AmethystHaze);
    }

    #[test]
    fn theme_round_trip_through_str() {
        for theme in ThemeName::ALL {
            assert_eq!(ThemeName::from_str(theme.as_str()), theme);
        }
    }

    #[test]
    fn theme_all_has_five_variants() {
        assert_eq!(ThemeName::ALL.len(), 5);
    }

    #[test]
    fn theme_display_name_is_human_readable() {
        assert_eq!(ThemeName::AmethystHaze.display_name(), "Amethyst Haze");
        assert_eq!(ThemeName::SolarDusk.display_name(), "Solar Dusk");
        assert_eq!(ThemeName::Nature.display_name(), "Nature");
        assert_eq!(ThemeName::Swiss.display_name(), "Swiss");
        assert_eq!(
            ThemeName::CassetteFuturism.display_name(),
            "Cassette Futurism"
        );
    }

    #[test]
    fn theme_description_not_empty() {
        for theme in ThemeName::ALL {
            assert!(!theme.description().is_empty());
        }
    }

    // -- Mode ---------------------------------------------------------------

    #[test]
    fn mode_as_str() {
        assert_eq!(Mode::Light.as_str(), "light");
        assert_eq!(Mode::Dark.as_str(), "dark");
    }

    #[test]
    fn mode_from_str_known_values() {
        assert_eq!(Mode::from_str("light"), Mode::Light);
        assert_eq!(Mode::from_str("dark"), Mode::Dark);
    }

    #[test]
    fn mode_from_str_unknown_defaults_to_light() {
        assert_eq!(Mode::from_str(""), Mode::Light);
        assert_eq!(Mode::from_str("auto"), Mode::Light);
    }

    #[test]
    fn mode_round_trip_through_str() {
        assert_eq!(Mode::from_str(Mode::Light.as_str()), Mode::Light);
        assert_eq!(Mode::from_str(Mode::Dark.as_str()), Mode::Dark);
    }

    #[test]
    fn mode_display_name() {
        assert_eq!(Mode::Light.display_name(), "Light");
        assert_eq!(Mode::Dark.display_name(), "Dark");
    }

    #[test]
    fn mode_description_not_empty() {
        assert!(!Mode::Light.description().is_empty());
        assert!(!Mode::Dark.description().is_empty());
    }
}
