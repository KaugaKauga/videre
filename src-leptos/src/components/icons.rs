//! SVG icons used by the UI components.

use leptos::prelude::*;

// ---------------------------------------------------------------------------
// Database (cylinder) — connection.rs, sidebar.rs, empty.rs
// ---------------------------------------------------------------------------

pub fn icon_database(size: u32) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg"
             width=size height=size viewBox="0 0 24 24"
             fill="none" stroke="currentColor" stroke-width="2"
             stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <ellipse cx="12" cy="6" rx="8" ry="3"/>
            <path d="M4 6v6a8 3 0 0 0 16 0v-6"/>
            <path d="M4 12v6a8 3 0 0 0 16 0v-6"/>
        </svg>
    }
}

// ---------------------------------------------------------------------------
// Spinner (loader) — connection.rs, sidebar.rs, table_page.rs
// ---------------------------------------------------------------------------

pub fn icon_spinner(size: u32) -> impl IntoView {
    view! {
        <svg class="animate-spin" xmlns="http://www.w3.org/2000/svg"
             width=size height=size viewBox="0 0 24 24"
             fill="none" stroke="currentColor" stroke-width="2"
             stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M12 6l0 -3"/>
            <path d="M16.25 7.75l2.15 -2.15"/>
            <path d="M18 12l3 0"/>
            <path d="M16.25 16.25l2.15 2.15"/>
            <path d="M12 18l0 3"/>
            <path d="M7.75 16.25l-2.15 2.15"/>
            <path d="M6 12l-3 0"/>
            <path d="M7.75 7.75l-2.15 -2.15"/>
        </svg>
    }
}

// ---------------------------------------------------------------------------
// Refresh (Lucide refresh-cw) — table page
// ---------------------------------------------------------------------------
// Source: https://github.com/lucide-icons/lucide/blob/main/icons/refresh-cw.svg
// ISC License — Copyright (c) 2026 Lucide Icons and Contributors.
// Permission to use, copy, modify, and/or distribute this software for any
// purpose with or without fee is hereby granted, provided that the above
// copyright notice and this permission notice appear in all copies.
// THE SOFTWARE IS PROVIDED "AS IS" AND THE AUTHOR DISCLAIMS ALL WARRANTIES
// WITH REGARD TO THIS SOFTWARE INCLUDING ALL IMPLIED WARRANTIES OF
// MERCHANTABILITY AND FITNESS. IN NO EVENT SHALL THE AUTHOR BE LIABLE FOR
// ANY SPECIAL, DIRECT, INDIRECT, OR CONSEQUENTIAL DAMAGES OR ANY DAMAGES
// WHATSOEVER RESULTING FROM LOSS OF USE, DATA OR PROFITS, WHETHER IN AN ACTION
// OF CONTRACT, NEGLIGENCE OR OTHER TORTIOUS ACTION, ARISING OUT OF OR IN
// CONNECTION WITH THE USE OR PERFORMANCE OF THIS SOFTWARE.

pub fn icon_refresh(size: u32, loading: RwSignal<bool>) -> impl IntoView {
    view! {
        <svg class=move || if loading.get() { "animate-spin" } else { "" }
             aria-hidden="true" xmlns="http://www.w3.org/2000/svg"
             width=size height=size viewBox="0 0 24 24"
             fill="none" stroke="currentColor" stroke-width="2"
             stroke-linecap="round" stroke-linejoin="round">
            <path d="M3 12a9 9 0 0 1 9-9 9.75 9.75 0 0 1 6.74 2.74L21 8"/>
            <path d="M21 3v5h-5"/>
            <path d="M21 12a9 9 0 0 1-9 9 9.75 9.75 0 0 1-6.74-2.74L3 16"/>
            <path d="M8 16H3v5"/>
        </svg>
    }
}

// ---------------------------------------------------------------------------
// List — sidebar.rs, indexes_page.rs
// ---------------------------------------------------------------------------

pub fn icon_list(size: u32) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg"
             width=size height=size viewBox="0 0 24 24"
             fill="none" stroke="currentColor" stroke-width="2"
             stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M9 6l11 0"/>
            <path d="M9 12l11 0"/>
            <path d="M9 18l11 0"/>
            <path d="M5 6l0 .01"/>
            <path d="M5 12l0 .01"/>
            <path d="M5 18l0 .01"/>
        </svg>
    }
}

// ---------------------------------------------------------------------------
// Users — sidebar.rs, roles_page.rs
// ---------------------------------------------------------------------------

pub fn icon_users(size: u32) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg"
             width=size height=size viewBox="0 0 24 24"
             fill="none" stroke="currentColor" stroke-width="2"
             stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <circle cx="9" cy="7" r="4"/>
            <path d="M3 21v-2a4 4 0 0 1 4 -4h4a4 4 0 0 1 4 4v2"/>
            <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
            <path d="M21 21v-2a4 4 0 0 0 -3 -3.85"/>
        </svg>
    }
}

// ---------------------------------------------------------------------------
// X / Close — tab_bar.rs
// ---------------------------------------------------------------------------

pub fn icon_x(size: u32) -> impl IntoView {
    view! {
        <svg xmlns="http://www.w3.org/2000/svg"
             width=size height=size viewBox="0 0 24 24"
             fill="none" stroke="currentColor" stroke-width="2"
             stroke-linecap="round" stroke-linejoin="round">
            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
            <path d="M18 6l-12 12"/>
            <path d="M6 6l12 12"/>
        </svg>
    }
}
