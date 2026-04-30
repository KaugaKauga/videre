//! Custom title bar component.
//!
//! Replaces native window chrome with a themed, draggable bar.
//! On macOS the native traffic lights are preserved via `titleBarStyle: overlay`.
//! On Windows/Linux custom control buttons are rendered.

use leptos::prelude::*;
use web_sys::MouseEvent;

use crate::window;

/// Detects if running on macOS (determined once at startup).
fn is_macos() -> bool {
    web_sys::window()
        .and_then(|w| w.navigator().platform().ok())
        .map(|p| p.contains("Mac"))
        .unwrap_or(false)
}

#[component]
pub fn TitleBar() -> impl IntoView {
    let macos = is_macos();

    let on_drag = move |ev: MouseEvent| {
        // Only left-click initiates a drag.
        if ev.button() != 0 {
            return;
        }
        leptos::task::spawn_local(async {
            window::start_dragging().await;
        });
    };

    let on_dblclick = move |_: MouseEvent| {
        leptos::task::spawn_local(async {
            window::toggle_maximize().await;
        });
    };

    view! {
        <div class="title-bar" on:mousedown=on_drag on:dblclick=on_dblclick>
            // Left spacer — keeps content clear of macOS traffic lights
            {if macos {
                Some(view! { <div class="title-bar-traffic-light-spacer"></div> })
            } else {
                None
            }}

            // Flexible center region. Future home of tabs / search / breadcrumbs.
            <div class="title-bar-center"></div>

            // Window controls (Windows/Linux only)
            {if !macos {
                Some(view! { <WindowControls /> })
            } else {
                None
            }}
        </div>
    }
}

/// Minimize / maximize / close buttons for platforms without native traffic lights.
#[component]
fn WindowControls() -> impl IntoView {
    // Stop propagation on mousedown so button clicks don't trigger a drag.
    let stop = |ev: MouseEvent| ev.stop_propagation();

    view! {
        <div class="title-bar-controls">
            <button
                class="title-bar-btn title-bar-minimize"
                on:mousedown=stop
                on:click=move |_| {
                    leptos::task::spawn_local(async { window::minimize().await });
                }
            >
                <svg width="10" height="10" viewBox="0 0 10 10">
                    <line x1="0" y1="5" x2="10" y2="5" stroke="currentColor" stroke-width="1.2"/>
                </svg>
            </button>
            <button
                class="title-bar-btn title-bar-maximize"
                on:mousedown=stop
                on:click=move |_| {
                    leptos::task::spawn_local(async { window::toggle_maximize().await });
                }
            >
                <svg width="10" height="10" viewBox="0 0 10 10">
                    <rect x="1" y="1" width="8" height="8" fill="none" stroke="currentColor" stroke-width="1.2"/>
                </svg>
            </button>
            <button
                class="title-bar-btn title-bar-close"
                on:mousedown=stop
                on:click=move |_| {
                    leptos::task::spawn_local(async { window::close().await });
                }
            >
                <svg width="10" height="10" viewBox="0 0 10 10">
                    <line x1="1" y1="1" x2="9" y2="9" stroke="currentColor" stroke-width="1.2"/>
                    <line x1="9" y1="1" x2="1" y2="9" stroke="currentColor" stroke-width="1.2"/>
                </svg>
            </button>
        </div>
    }
}
