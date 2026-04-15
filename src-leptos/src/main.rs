mod components;
mod pages;
mod stores;
mod tauri;
mod theme;
mod types;

use pages::connection::ConnectionPage;
use stores::connection_store::ConnectionStore;
use stores::db_store::DbStore;
use leptos::prelude::*;
use components::shell::Shell;

fn main() {
    theme::initialize_theme();
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let conn_store = ConnectionStore::init();
    let db_store = DbStore::init();
    provide_context(conn_store);
    provide_context(db_store);

    view! {
        {move || {
            if db_store.is_connected.get() {
                view! { <Shell /> }.into_any()
            } else {
                view! { <ConnectionPage /> }.into_any()
            }
        }}
    }
}
