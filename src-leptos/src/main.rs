mod components;
mod pages;
mod stores;
mod tauri;
mod theme;
mod types;
mod window;

use components::shell::Shell;
use components::title_bar::TitleBar;
use leptos::prelude::*;
use pages::connection::ConnectionPage;
use stores::connection_store::ConnectionStore;
use stores::db_store::DbStore;

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
        <TitleBar />
        {move || {
            if db_store.is_connected.get() {
                view! { <Shell /> }.into_any()
            } else {
                view! { <ConnectionPage /> }.into_any()
            }
        }}
    }
}
