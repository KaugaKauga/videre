mod connection;
mod connection_store;
mod data_table;
mod db_store;
mod indexes_page;
mod empty;
mod shell;
mod sidebar;
mod tab_bar;
mod tab_store;
mod table_page;
mod tauri;
mod types;

use connection::ConnectionPage;
use connection_store::ConnectionStore;
use db_store::DbStore;
use leptos::prelude::*;
use shell::Shell;

fn main() {
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
