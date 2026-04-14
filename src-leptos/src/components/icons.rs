//! Shared SVG icon functions to reduce duplication across components.
//!
//! Only icons that appear in **multiple files** are extracted here.
//! Single-use icons remain inline in their respective components.

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
