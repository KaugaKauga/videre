mod connection;
mod connection_store;
mod db_store;
mod tauri;
mod types;

use connection::ConnectionPage;
use connection_store::ConnectionStore;
use db_store::DbStore;
use leptos::prelude::*;

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
        <ConnectionPage />
    }
}
