use leptos::prelude::*;

/// Reusable slide-out side panel with a backdrop.
///
/// - Clicking the backdrop or the ✕ button closes the panel.
/// - `title` is reactive so the parent can update it dynamically.
/// - Pass arbitrary body content as children.
///
/// ```ignore
/// <Drawer open=my_signal title=my_title subtitle="Some description">
///     {move || { /* reactive body content */ }}
/// </Drawer>
/// ```
#[component]
pub fn Drawer(
    /// Controls whether the panel is visible.
    open: RwSignal<bool>,
    /// Reactive panel title (displayed in the header).
    title: RwSignal<String>,
    /// Optional static subtitle below the title.
    #[prop(optional)]
    subtitle: &'static str,
    /// Body content — typically a reactive closure.
    children: Children,
) -> impl IntoView {
    view! {
        <div
            class=move || if open.get() { "row-detail-backdrop open" } else { "row-detail-backdrop" }
            on:click=move |_| open.set(false)
        />
        <div class=move || if open.get() { "row-detail-panel open" } else { "row-detail-panel" }>
            <div class="row-detail-header">
                <h3>{move || title.get()}</h3>
                <button class="btn btn-ghost btn-sm" on:click=move |_| open.set(false)>
                    "\u{2715}"
                </button>
            </div>
            {if !subtitle.is_empty() {
                Some(view! { <p class="text-muted text-sm row-detail-subtitle">{subtitle}</p> })
            } else {
                None
            }}
            <div class="row-detail-body">
                {children()}
            </div>
        </div>
    }
}
