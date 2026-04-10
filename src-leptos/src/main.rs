mod connection;
mod tauri;
mod types;

use connection::ConnectionPage;
use leptos::prelude::*;

fn main() {
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! {
        <ConnectionPage />
    }
}
