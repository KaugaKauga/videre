//! Settings page — theme picker, light/dark mode toggle, and about section.

use leptos::prelude::*;

use crate::theme::{self, FontSize, Mode, ThemeName};

// ---------------------------------------------------------------------------
// Settings page component
// ---------------------------------------------------------------------------

#[component]
pub fn SettingsPage() -> impl IntoView {
    let (current_theme, set_current_theme) = signal(theme::get_stored_theme());
    let (current_mode, set_current_mode) = signal(theme::get_stored_mode());
    let (current_font_size, set_current_font_size) = signal(theme::get_stored_font_size());

    let handle_theme_change = move |t: ThemeName| {
        theme::set_theme(t);
        set_current_theme.set(t);
    };

    let handle_mode_change = move |m: Mode| {
        theme::set_mode(m);
        set_current_mode.set(m);
    };

    let handle_font_size_change = move |s: FontSize| {
        theme::set_font_size(s);
        set_current_font_size.set(s);
    };

    view! {
        <div class="settings-page">
            <div class="settings-container">
                // ── Header ──────────────────────────────────────────
                <div class="settings-header">
                    <h2>"Settings"</h2>
                    <p class="text-sm text-muted">"Customize your Videre experience"</p>
                </div>

                // ── Appearance card ─────────────────────────────────
                <div class="settings-card">
                    <div class="settings-card-title">
                        {palette_icon()}
                        <h3>"Appearance"</h3>
                    </div>

                    // Mode selection
                    <div class="settings-section">
                        <p class="text-sm text-muted">"Choose between light and dark mode"</p>
                        <div class="mode-grid">
                            {Mode::Light.render_button(current_mode, handle_mode_change)}
                            {Mode::Dark.render_button(current_mode, handle_mode_change)}
                        </div>
                    </div>

                    // Font size selection
                    <div class="settings-section">
                        <p class="text-sm text-muted">"Adjust the font size for sidebar items, tabs, and table rows"</p>
                        <div class="mode-grid">
                            {FontSize::ALL.map(|s| s.render_button(current_font_size, handle_font_size_change)).collect_view()}
                        </div>
                    </div>

                    // Theme selection
                    <div class="settings-section">
                        <p class="text-sm text-muted">"Choose a color theme"</p>
                        <div class="theme-grid">
                            {ThemeName::ALL.map(|t| t.render_card(current_theme, handle_theme_change)).collect_view()}
                        </div>
                    </div>
                </div>

                // ── About card ──────────────────────────────────────
                <div class="settings-card">
                    <h3>"About"</h3>
                    <div class="about-rows">
                        <div class="about-row">
                            <span class="text-muted">"Version"</span>
                            <span class="about-value">"0.1.0"</span>
                        </div>
                        <div class="about-row">
                            <span class="text-muted">"Database Inspector"</span>
                            <span class="about-value">"Videre"</span>
                        </div>
                    </div>
                </div>
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Font size button
// ---------------------------------------------------------------------------

impl FontSize {
    fn render_button(
        self,
        current: ReadSignal<FontSize>,
        on_click: impl Fn(FontSize) + 'static + Copy,
    ) -> impl IntoView {
        let is_active = move || current.get() == self;
        let preview_size = match self {
            FontSize::Small => "0.75rem",
            FontSize::Normal => "0.8125rem",
            FontSize::Large => "0.9375rem",
            FontSize::XLarge => "1.0625rem",
        };

        view! {
            <button
                class="mode-btn"
                class:active=is_active
                on:click=move |_| on_click(self)
            >
                <div class="mode-btn-inner">
                    <div class="mode-icon" class:active=is_active style=format!("font-size:{preview_size};font-weight:600;width:20px;height:20px;display:flex;align-items:center;justify-content:center")>
                        "Aa"
                    </div>
                    <div>
                        <h4>{self.display_name()}</h4>
                        <p class="text-xs text-muted">{self.description()}</p>
                    </div>
                </div>
            </button>
        }
    }
}

// ---------------------------------------------------------------------------
// Mode button
// ---------------------------------------------------------------------------

impl Mode {
    fn render_button(
        self,
        current: ReadSignal<Mode>,
        on_click: impl Fn(Mode) + 'static + Copy,
    ) -> impl IntoView {
        let is_active = move || current.get() == self;

        view! {
            <button
                class="mode-btn"
                class:active=is_active
                on:click=move |_| on_click(self)
            >
                <div class="mode-btn-inner">
                    <div class="mode-icon" class:active=is_active>
                        {match self {
                            Mode::Light => sun_icon().into_any(),
                            Mode::Dark => moon_icon().into_any(),
                        }}
                    </div>
                    <div>
                        <h4>{self.display_name()}</h4>
                        <p class="text-xs text-muted">{self.description()}</p>
                    </div>
                </div>
            </button>
        }
    }
}

// ---------------------------------------------------------------------------
// Theme card
// ---------------------------------------------------------------------------

impl ThemeName {
    fn render_card(
        self,
        current: ReadSignal<ThemeName>,
        on_click: impl Fn(ThemeName) + 'static + Copy,
    ) -> impl IntoView {
        let is_active = move || current.get() == self;
        let (c1, c2, c3) = self.preview_colors();

        view! {
            <button
                class="theme-card"
                class:active=is_active
                on:click=move |_| on_click(self)
            >
                <div class="theme-card-top">
                    <div>
                        <h4>{self.display_name()}</h4>
                        <p class="text-xs text-muted">{self.description()}</p>
                    </div>
                    {move || {
                        if is_active() {
                            Some(view! {
                                <div class="theme-check">
                                    <div class="theme-check-dot"></div>
                                </div>
                            })
                        } else {
                            None
                        }
                    }}
                </div>
                <div class="theme-swatches">
                    <div class="theme-swatch" style=format!("background:{c1}")></div>
                    <div class="theme-swatch" style=format!("background:{c2}")></div>
                    <div class="theme-swatch" style=format!("background:{c3}")></div>
                </div>
            </button>
        }
    }

    /// Returns (primary, accent, muted) CSS color strings for the preview swatches.
    fn preview_colors(self) -> (&'static str, &'static str, &'static str) {
        match self {
            Self::AmethystHaze => (
                "oklch(0.55 0.15 295)",
                "oklch(0.70 0.14 350)",
                "oklch(0.90 0.03 300)",
            ),
            Self::SolarDusk => (
                "oklch(0.60 0.16 55)",
                "oklch(0.80 0.14 90)",
                "oklch(0.92 0.04 70)",
            ),
            Self::Nature => (
                "oklch(0.55 0.14 155)",
                "oklch(0.72 0.12 160)",
                "oklch(0.92 0.04 155)",
            ),
            Self::Swiss => (
                "oklch(0.528 0.216 27.33)",
                "oklch(0.15 0 0)",
                "oklch(0.948 0 0)",
            ),
            Self::CassetteFuturism => (
                "oklch(0.82 0.15 75)",
                "oklch(0.145 0.008 80)",
                "oklch(0.68 0.22 25)",
            ),
        }
    }
}

// ---------------------------------------------------------------------------
// Inline SVG icons
// ---------------------------------------------------------------------------

fn sun_icon() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <circle cx="12" cy="12" r="4"></circle>
            <path d="M12 2v2"></path>
            <path d="M12 20v2"></path>
            <path d="m4.93 4.93 1.41 1.41"></path>
            <path d="m17.66 17.66 1.41 1.41"></path>
            <path d="M2 12h2"></path>
            <path d="M20 12h2"></path>
            <path d="m6.34 17.66-1.41 1.41"></path>
            <path d="m19.07 4.93-1.41 1.41"></path>
        </svg>
    }
}

fn moon_icon() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <path d="M12 3a6 6 0 0 0 9 9 9 9 0 1 1-9-9Z"></path>
        </svg>
    }
}

fn palette_icon() -> impl IntoView {
    view! {
        <svg
            xmlns="http://www.w3.org/2000/svg"
            width="20"
            height="20"
            viewBox="0 0 24 24"
            fill="none"
            stroke="currentColor"
            stroke-width="2"
            stroke-linecap="round"
            stroke-linejoin="round"
        >
            <circle cx="13.5" cy="6.5" r="0.5" fill="currentColor"></circle>
            <circle cx="17.5" cy="10.5" r="0.5" fill="currentColor"></circle>
            <circle cx="8.5" cy="7.5" r="0.5" fill="currentColor"></circle>
            <circle cx="6.5" cy="12.5" r="0.5" fill="currentColor"></circle>
            <path d="M12 2C6.5 2 2 6.5 2 12s4.5 10 10 10c0.926 0 1.648-.746 1.648-1.688 0-.437-.18-.835-.437-1.125-.29-.289-.438-.652-.438-1.125a1.64 1.64 0 0 1 1.668-1.668h1.996c3.051 0 5.555-2.503 5.555-5.554C21.965 6.012 17.461 2 12 2z"></path>
        </svg>
    }
}
