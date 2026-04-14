use leptos::prelude::*;

use crate::db_store::DbStore;
use crate::icons;
use crate::tab_store::{TabStore, TabType};

#[component]
pub fn Sidebar() -> impl IntoView {
    let db = use_context::<DbStore>().expect("DbStore not provided");
    let tab_store = use_context::<TabStore>().expect("TabStore not provided");

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

    view! {
        <aside class="sidebar">
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
                                let items: Vec<_> = tables.iter().map(|table| {
                                    let name = table.name.clone();
                                    let schema = table.schema.clone();
                                    let display = table.name.clone();
                                    view! {
                                        <button class="sidebar-menu-button"
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
                        <button class="sidebar-menu-button" on:click=on_indexes_click>
                            {icons::icon_list(16)}
                            <span>"Indexes"</span>
                        </button>
                        <button class="sidebar-menu-button" on:click=on_roles_click>
                            {icons::icon_users(16)}
                            <span>"Roles"</span>
                        </button>
                    </nav>
                </div>
            </div>

            // Footer: Connection + Settings
            <div class="sidebar-footer">
                <nav class="sidebar-menu">
                    <button class="sidebar-menu-button" on:click=on_connection_click>
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
                    <button class="sidebar-menu-button" on:click=on_settings_click>
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
        </aside>
    }
}
