//! Theme management for Videre.
//!
//! Reads/writes localStorage and toggles CSS classes on `<html>` to switch
//! between themes.

// ---------------------------------------------------------------------------
// Types
// ---------------------------------------------------------------------------

/// Available color themes.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ThemeName {
    Swiss,
    CassetteFuturism,
}

impl ThemeName {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Swiss => "swiss",
            Self::CassetteFuturism => "cassette-futurism",
        }
    }

    pub fn from_str(s: &str) -> Self {
        match s {
            "cassette-futurism" => Self::CassetteFuturism,
            _ => Self::Swiss,
        }
    }

    pub const ALL: [ThemeName; 2] = [Self::Swiss, Self::CassetteFuturism];

    pub fn display_name(self) -> &'static str {
        match self {
            Self::Swiss => "Swiss",
            Self::CassetteFuturism => "Cassette Futurism",
        }
    }

    pub fn description(self) -> &'static str {
        match self {
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

// ---------------------------------------------------------------------------
// localStorage keys
// ---------------------------------------------------------------------------

const THEME_KEY: &str = "videre-theme";
const FONT_SIZE_KEY: &str = "videre-font-size";
const SIDEBAR_WIDTH_KEY: &str = "videre-sidebar-width";

/// Minimum sidebar width in pixels; prevents the user from collapsing it to nothing.
pub const SIDEBAR_MIN_PX: f64 = 160.0;
/// Maximum sidebar width in pixels; keeps it from swallowing the content area.
pub const SIDEBAR_MAX_PX: f64 = 480.0;

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

/// Read the stored theme from localStorage (defaults to Swiss).
pub fn get_stored_theme() -> ThemeName {
    local_storage()
        .and_then(|s| s.get_item(THEME_KEY).ok().flatten())
        .map(|v| ThemeName::from_str(&v))
        .unwrap_or(ThemeName::Swiss)
}

/// Write theme to localStorage.
fn set_stored_theme(theme: ThemeName) {
    if let Some(s) = local_storage() {
        let _ = s.set_item(THEME_KEY, theme.as_str());
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

/// Read the stored sidebar width (pixels).  None = use CSS default.
pub fn get_stored_sidebar_width() -> Option<f64> {
    local_storage()
        .and_then(|s| s.get_item(SIDEBAR_WIDTH_KEY).ok().flatten())
        .and_then(|v| v.parse::<f64>().ok())
        .map(|w| w.clamp(SIDEBAR_MIN_PX, SIDEBAR_MAX_PX))
}

/// Write sidebar width (pixels) to localStorage.
pub fn set_stored_sidebar_width(px: f64) {
    if let Some(s) = local_storage() {
        let _ = s.set_item(SIDEBAR_WIDTH_KEY, &px.to_string());
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

/// Apply theme by toggling CSS classes on `<html>`.
pub fn apply_theme(theme: ThemeName) {
    let Some(el) = document_element() else {
        return;
    };
    let cl = el.class_list();
    for t in ThemeName::ALL {
        let _ = cl.remove_1(t.as_str());
    }
    let _ = cl.add_1(theme.as_str());
}

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

/// Call once at startup (before mount) to apply stored preferences.
pub fn initialize_theme() {
    apply_theme(get_stored_theme());
    apply_font_size(get_stored_font_size());
}

/// Change and persist the theme.
pub fn set_theme(theme: ThemeName) {
    set_stored_theme(theme);
    apply_theme(theme);
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
        assert_eq!(ThemeName::Swiss.as_str(), "swiss");
        assert_eq!(ThemeName::CassetteFuturism.as_str(), "cassette-futurism");
    }

    #[test]
    fn theme_from_str_known_values() {
        assert_eq!(ThemeName::from_str("swiss"), ThemeName::Swiss);
        assert_eq!(
            ThemeName::from_str("cassette-futurism"),
            ThemeName::CassetteFuturism
        );
    }

    #[test]
    fn theme_from_str_unknown_defaults_to_swiss() {
        assert_eq!(ThemeName::from_str(""), ThemeName::Swiss);
        assert_eq!(ThemeName::from_str("neon-glow"), ThemeName::Swiss);
        assert_eq!(ThemeName::from_str("NATURE"), ThemeName::Swiss);
    }

    #[test]
    fn theme_round_trip_through_str() {
        for theme in ThemeName::ALL {
            assert_eq!(ThemeName::from_str(theme.as_str()), theme);
        }
    }

    #[test]
    fn theme_all_has_two_variants() {
        assert_eq!(ThemeName::ALL.len(), 2);
    }

    #[test]
    fn theme_display_name_is_human_readable() {
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
}
