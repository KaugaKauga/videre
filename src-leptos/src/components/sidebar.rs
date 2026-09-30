use leptos::prelude::*;
use wasm_bindgen::JsCast;
use web_sys::{HtmlButtonElement, KeyboardEvent, PointerEvent};

use crate::components::icons;
use crate::stores::db_store::DbStore;
use crate::stores::tab_store::{TabStore, TabType};
use crate::theme;

#[component]
pub fn Sidebar() -> impl IntoView {
    let db = use_context::<DbStore>().expect("DbStore not provided");
    let tab_store = use_context::<TabStore>().expect("TabStore not provided");
    // Roving tabindex: only the last focused item is a Tab stop.
    let focused_index = RwSignal::new(0usize);
    let table_count = move || {
        if db.is_loading.get() {
            0
        } else {
            db.tables.with(Vec::len)
        }
    };
    // Clamp so a metadata refresh that shortens the table list keeps one Tab stop.
    let tab_stop = StoredValue::new(Selector::new(move || {
        focused_index
            .get()
            .min(table_count() + FIXED_SIDEBAR_ITEMS - 1)
    }));
    let item_tabindex = move |index| {
        if tab_stop.with_value(|stop| stop.selected(index)) {
            0
        } else {
            -1
        }
    };

    let on_indexes_click = move |_| {
        tab_store.open_singleton_tab(TabType::Indexes, "Indexes");
    };

    let on_roles_click = move |_| {
        tab_store.open_singleton_tab(TabType::Roles, "Roles");
    };

    let on_connection_click = move |_| {
        tab_store.open_singleton_tab(TabType::Connection, "Connection");
    };

    let on_settings_click = move |_| {
        tab_store.open_singleton_tab(TabType::Settings, "Settings");
    };

    // ---- Resize state ------------------------------------------------------
    // `width_px`: Some(n) = user-set, None = fall back to CSS default (15rem,
    // scales with root font-size).  `drag_start`: (pointer_x, width_at_drag_start).
    let width_px = RwSignal::new(theme::get_stored_sidebar_width());
    let drag_start = RwSignal::new(None::<(f64, f64)>);

    let on_handle_down = move |e: PointerEvent| {
        // Read current rendered width so we can drag relative to it, even if
        // the user hasn't set one yet (CSS default).
        let current_w = width_px.get_untracked().unwrap_or_else(|| {
            web_sys::window()
                .and_then(|w| w.document())
                .and_then(|d| d.query_selector(".sidebar").ok().flatten())
                .map(|el| el.get_bounding_client_rect().width())
                .unwrap_or(240.0)
        });
        drag_start.set(Some((e.client_x() as f64, current_w)));
        if let Some(target) = e
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
        {
            let _ = target.set_pointer_capture(e.pointer_id());
        }
        e.prevent_default();
    };

    let on_handle_move = move |e: PointerEvent| {
        if let Some((start_x, start_w)) = drag_start.get_untracked() {
            let new_w = (start_w + (e.client_x() as f64 - start_x))
                .clamp(theme::SIDEBAR_MIN_PX, theme::SIDEBAR_MAX_PX);
            width_px.set(Some(new_w));
        }
    };

    let on_handle_up = move |_: PointerEvent| {
        if drag_start.get_untracked().is_some() {
            if let Some(w) = width_px.get_untracked() {
                theme::set_stored_sidebar_width(w);
            }
        }
        drag_start.set(None);
    };

    let is_dragging = move || drag_start.get().is_some();

    let sidebar_style = move || match width_px.get() {
        Some(w) => format!("width: {w}px; min-width: {w}px;"),
        None => String::new(),
    };

    view! {
        <aside class="sidebar" style=sidebar_style
            aria-label="Database navigation"
            on:keydown=on_sidebar_keydown
        >
            // Header
            <div class="sidebar-header">
                {icons::icon_database(20)}
                <h2 class="sidebar-title">"Videre"</h2>
            </div>

            // Scrollable content area
            <div class="sidebar-content">
                // Tables group
                <div class="sidebar-group sidebar-group-tables">
                    <div class="sidebar-group-label">"Tables"</div>
                    <div class="sidebar-group-content">
                        {move || {
                            let loading = db.is_loading.get();
                            let tables = db.tables.get();

                            if loading {
                                view! {
                                    <div class="sidebar-loading">
                                        {icons::icon_spinner(16)}
                                    </div>
                                }.into_any()
                            } else if tables.is_empty() {
                                view! {
                                    <div class="sidebar-empty">"No tables found"</div>
                                }.into_any()
                            } else {
                                let items: Vec<_> = tables.iter().enumerate().map(|(index, table)| {
                                    let name = table.name.clone();
                                    let schema = table.schema.clone();
                                    let display = table.name.clone();
                                    let tooltip = format!("{}.{}", table.schema, table.name);
                                    view! {
                                        <button class="sidebar-menu-button"
                                            tabindex=move || item_tabindex(index)
                                            on:focus=move |_| focused_index.set(index)
                                            title=tooltip
                                            on:click=move |_| {
                                                tab_store.open_table_tab(name.clone(), schema.clone());
                                            }
                                        >
                                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"
                                                 viewBox="0 0 24 24" fill="none" stroke="currentColor"
                                                 stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                                                <path d="M3 10h18"/>
                                                <path d="M10 3v18"/>
                                                <rect x="3" y="3" width="18" height="18" rx="1"/>
                                            </svg>
                                            <span>{display}</span>
                                        </button>
                                    }
                                }).collect();
                                view! { <nav class="sidebar-menu">{items}</nav> }.into_any()
                            }
                        }}
                    </div>
                </div>

                // Bottom section: Indexes + Roles
                <div class="sidebar-bottom-section">
                    <nav class="sidebar-menu">
                        <button class="sidebar-menu-button"
                            tabindex=move || item_tabindex(table_count())
                            on:focus=move |_| focused_index.set(table_count())
                            on:click=on_indexes_click
                        >
                            {icons::icon_list(16)}
                            <span>"Indexes"</span>
                        </button>
                        <button class="sidebar-menu-button"
                            tabindex=move || item_tabindex(table_count() + 1)
                            on:focus=move |_| focused_index.set(table_count() + 1)
                            on:click=on_roles_click
                        >
                            {icons::icon_users(16)}
                            <span>"Roles"</span>
                        </button>
                    </nav>
                </div>
            </div>

            // Footer: Connection + Settings
            <div class="sidebar-footer">
                <nav class="sidebar-menu">
                    <button class="sidebar-menu-button"
                        tabindex=move || item_tabindex(table_count() + 2)
                        on:focus=move |_| focused_index.set(table_count() + 2)
                        on:click=on_connection_click
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"
                             viewBox="0 0 24 24" fill="none" stroke="currentColor"
                             stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                            <path d="M7 12l5 5l5 -5"/>
                            <path d="M12 3v14"/>
                            <path d="M5 20h14"/>
                        </svg>
                        <span>"Connection"</span>
                    </button>
                    <button class="sidebar-menu-button"
                        tabindex=move || item_tabindex(table_count() + 3)
                        on:focus=move |_| focused_index.set(table_count() + 3)
                        on:click=on_settings_click
                    >
                        <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16"
                             viewBox="0 0 24 24" fill="none" stroke="currentColor"
                             stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                            <path d="M10.325 4.317c.426 -1.756 2.924 -1.756 3.35 0a1.724 1.724 0 0 0 2.573 1.066c1.543 -.94 3.31 .826 2.37 2.37a1.724 1.724 0 0 0 1.066 2.573c1.756 .426 1.756 2.924 0 3.35a1.724 1.724 0 0 0 -1.066 2.573c.94 1.543 -.826 3.31 -2.37 2.37a1.724 1.724 0 0 0 -2.573 1.066c-.426 1.756 -2.924 1.756 -3.35 0a1.724 1.724 0 0 0 -2.573 -1.066c-1.543 .94 -3.31 -.826 -2.37 -2.37a1.724 1.724 0 0 0 -1.066 -2.573c-1.756 -.426 -1.756 -2.924 0 -3.35a1.724 1.724 0 0 0 1.066 -2.573c-.94 -1.543 .826 -3.31 2.37 -2.37c1.05 .64 2.4 .09 2.573 -1.066z"/>
                            <circle cx="12" cy="12" r="3"/>
                        </svg>
                        <span>"Settings"</span>
                    </button>
                </nav>
            </div>

            // Drag handle — straddles the right border.
            <div
                class="sidebar-resize-handle"
                class:dragging=is_dragging
                title="Drag to resize"
                on:pointerdown=on_handle_down
                on:pointermove=on_handle_move
                on:pointerup=on_handle_up
                on:pointercancel=on_handle_up
            />
        </aside>
    }
}

/// Indexes, Roles, Connection and Settings, which follow the tables.
const FIXED_SIDEBAR_ITEMS: usize = 4;

fn sidebar_buttons(sidebar: &web_sys::Element) -> Vec<HtmlButtonElement> {
    let Ok(nodes) = sidebar.query_selector_all(".sidebar-menu-button") else {
        return Vec::new();
    };
    (0..nodes.length())
        .filter_map(|index| nodes.item(index)?.dyn_into::<HtmlButtonElement>().ok())
        .collect()
}

pub(super) fn focus_sidebar() {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Ok(Some(sidebar)) = document.query_selector(".sidebar") else {
        return;
    };
    let buttons = sidebar_buttons(&sidebar);
    if let Some(button) = buttons
        .iter()
        .find(|button| button.tab_index() == 0)
        .or_else(|| buttons.first())
    {
        let _ = button.focus();
    }
}

fn on_sidebar_keydown(event: KeyboardEvent) {
    if event.default_prevented()
        || event.is_composing()
        || event.meta_key()
        || event.ctrl_key()
        || event.alt_key()
        || event.shift_key()
    {
        return;
    }
    let Some(sidebar) = event
        .current_target()
        .and_then(|target| target.dyn_into::<web_sys::Element>().ok())
    else {
        return;
    };
    let Some(target) = event
        .target()
        .and_then(|target| target.dyn_into::<HtmlButtonElement>().ok())
    else {
        return;
    };
    let buttons = sidebar_buttons(&sidebar);
    let Some(current) = buttons
        .iter()
        .position(|button| button.is_same_node(Some(&target)))
    else {
        return;
    };
    if let Some(index) = sidebar_navigation_index(&event.key(), current, buttons.len()) {
        event.prevent_default();
        let _ = buttons[index].focus();
    }
}

fn sidebar_navigation_index(key: &str, current: usize, count: usize) -> Option<usize> {
    match key {
        "ArrowDown" | "j" => Some((current + 1).min(count - 1)),
        "ArrowUp" | "k" => Some(current.saturating_sub(1)),
        "Home" => Some(0),
        "End" => Some(count - 1),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::sidebar_navigation_index;

    #[test]
    fn arrows_and_vim_keys_move_between_sidebar_items() {
        for key in ["ArrowDown", "j"] {
            assert_eq!(sidebar_navigation_index(key, 1, 6), Some(2));
        }
        for key in ["ArrowUp", "k"] {
            assert_eq!(sidebar_navigation_index(key, 2, 6), Some(1));
        }
    }

    #[test]
    fn movement_stops_at_the_first_and_last_items() {
        for key in ["ArrowUp", "k"] {
            assert_eq!(sidebar_navigation_index(key, 0, 6), Some(0));
        }
        for key in ["ArrowDown", "j"] {
            assert_eq!(sidebar_navigation_index(key, 5, 6), Some(5));
        }
    }

    #[test]
    fn home_and_end_jump_to_sidebar_boundaries() {
        assert_eq!(sidebar_navigation_index("Home", 2, 6), Some(0));
        assert_eq!(sidebar_navigation_index("End", 2, 6), Some(5));
    }

    #[test]
    fn a_single_item_keeps_focus() {
        for key in ["j", "k", "Home", "End"] {
            assert_eq!(sidebar_navigation_index(key, 0, 1), Some(0));
        }
    }

    #[test]
    fn activation_and_other_keys_keep_their_native_behavior() {
        for key in ["Enter", " ", "Tab", "Escape", "c", "J", "K"] {
            assert_eq!(sidebar_navigation_index(key, 1, 6), None);
        }
    }
}
