mod connection;
mod connection_store;
mod tauri;
mod types;

use connection::ConnectionPage;
use connection_store::ConnectionStore;
use leptos::prelude::*;

fn main() {
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    let conn_store = ConnectionStore::init();
    provide_context(conn_store);

    view! {
        <ConnectionPage />
    }
}
