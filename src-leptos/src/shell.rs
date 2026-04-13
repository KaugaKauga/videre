use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::connection::ConnectionPage;
use crate::empty::{EmptyState, EmptyTab};
use crate::sidebar::Sidebar;
use crate::tab_bar::TabBar;
use crate::tab_store::{TabStore, TabType};
use crate::table_page::TablePage;

/// Main application shell shown after a successful DB connection.
///
/// Layout: sidebar | (tab-bar / content-area)
///
/// Also registers global keyboard shortcuts (Cmd/Ctrl + T/W/1-9).
#[component]
pub fn Shell() -> impl IntoView {
    let tab_store = TabStore::init();
    provide_context(tab_store);

    // ---- Keyboard shortcuts ------------------------------------------------
    setup_keyboard_shortcuts(tab_store);

    // ---- View --------------------------------------------------------------
    view! {
        <div class="shell">
            <Sidebar />
            <div class="shell-main">
                <TabBar />
                <div class="shell-content">
                    {move || {
                        let active = tab_store.active_tab();
                        match active {
                            None => view! { <EmptyState /> }.into_any(),
                            Some(tab) => match &tab.tab_type {
                                TabType::Empty => {
                                    view! { <EmptyTab /> }.into_any()
                                }
                                TabType::Connection => {
                                    view! { <ConnectionPage /> }.into_any()
                                }
                                TabType::Table { name, schema } => {
                                    let n = name.clone();
                                    let s = schema.clone();
                                    view! {
                                        <TablePage name=n schema=s />
                                    }.into_any()
                                }
                                TabType::Settings => {
                                    view! {
                                        <div class="placeholder-page">
                                            <span class="text-muted">"Settings (coming soon)"</span>
                                        </div>
                                    }.into_any()
                                }
                                TabType::Indexes => {
                                    view! {
                                        <div class="placeholder-page">
                                            <span class="text-muted">"Indexes (coming soon)"</span>
                                        </div>
                                    }.into_any()
                                }
                                TabType::Roles => {
                                    view! {
                                        <div class="placeholder-page">
                                            <span class="text-muted">"Roles (coming soon)"</span>
                                        </div>
                                    }.into_any()
                                }
                            },
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

/// Sets up global keydown listener for tab shortcuts.
///
/// The closure is forgotten (leaks ~200 bytes of WASM memory) so that only
/// the `js_sys::Function` reference — which IS Send+Sync — is captured by
/// `on_cleanup`. The event listener itself is properly removed on cleanup.
fn setup_keyboard_shortcuts(tab_store: TabStore) {
    let window = web_sys::window().expect("no global window");
    let is_mac = window
        .navigator()
        .platform()
        .unwrap_or_default()
        .contains("Mac");

    let cb = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |ev: web_sys::KeyboardEvent| {
        let modifier = if is_mac { ev.meta_key() } else { ev.ctrl_key() };
        if !modifier {
            return;
        }

        match ev.key().as_str() {
            "t" => {
                ev.prevent_default();
                tab_store.open_empty_tab();
            }
            "w" => {
                ev.prevent_default();
                tab_store.close_active_tab();
            }
            k => {
                if let Ok(n) = k.parse::<usize>() {
                    if (1..=9).contains(&n) {
                        ev.prevent_default();
                        tab_store.switch_to_tab(n - 1);
                    }
                }
            }
        }
    });

    let js_fn: js_sys::Function = cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
    cb.forget(); // leak closure so js_fn stays valid; on_cleanup removes the listener

    window
        .add_event_listener_with_callback("keydown", &js_fn)
        .expect("failed to add keydown listener");

    on_cleanup(move || {
        if let Some(w) = web_sys::window() {
            let _ = w.remove_event_listener_with_callback("keydown", &js_fn);
        }
    });
}
