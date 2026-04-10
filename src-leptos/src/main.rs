use leptos::prelude::*;

fn main() {
    leptos::mount::mount_to_body(App);
}

#[component]
fn App() -> impl IntoView {
    view! {
        <div style="display: flex; align-items: center; justify-content: center; height: 100vh; font-family: sans-serif;">
            <div style="text-align: center;">
                <h1 style="font-size: 2rem; margin-bottom: 0.5rem;">"Videre"</h1>
                <p style="color: #888;">"Leptos edition — it works!"</p>
            </div>
        </div>
    }
}
