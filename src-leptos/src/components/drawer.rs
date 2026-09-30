use leptos::{
    ev, html,
    leptos_dom::helpers::{request_animation_frame, window_event_listener},
    prelude::*,
};

/// Reusable slide-out side panel with a backdrop.
///
/// - Clicking the backdrop or the ✕ button closes the panel.
/// - Opening moves focus to the ✕ button; `on_close` decides where it returns.
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
    /// Optional reactive context line below the title and static subtitle.
    #[prop(optional)]
    context: Option<RwSignal<String>>,
    /// Uses the wider drawer surface for information-dense content.
    #[prop(optional)]
    wide: bool,
    /// Registers this drawer's Escape handler. Parent surfaces with mutually
    /// exclusive drawers can provide one shared handler instead.
    #[prop(optional = true)]
    close_on_escape: bool,
    /// Invoked for every user-initiated close, after the panel has been hidden.
    #[prop(optional)]
    on_close: Option<Callback<()>>,
    /// Body content — typically a reactive closure.
    children: Children,
) -> impl IntoView {
    let close = Callback::new(move |_| {
        open.set(false);
        if let Some(on_close) = on_close {
            on_close.run(());
        }
    });

    let close_button = NodeRef::<html::Button>::new();
    Effect::new(move |_| {
        if open.get() {
            // Wait a frame so the panel is no longer `inert` when focused.
            request_animation_frame(move || {
                if open.try_get_untracked().unwrap_or(false) {
                    if let Some(button) = close_button.try_get_untracked().flatten() {
                        // The panel is still sliding in from off-canvas. A scrolling
                        // focus would shift the overflow-hidden page sideways and
                        // expose sibling drawers parked off-screen.
                        let options = web_sys::FocusOptions::new();
                        options.set_prevent_scroll(true);
                        let _ = button.focus_with_options(&options);
                    }
                }
            });
        }
    });

    if close_on_escape {
        let escape_close = close;
        let keydown = window_event_listener(ev::keydown, move |event| {
            if event.key() == "Escape" && open.get_untracked() {
                event.prevent_default();
                escape_close.run(());
            }
        });
        on_cleanup(move || keydown.remove());
    }

    view! {
        <div
            class=move || if open.get() { "row-detail-backdrop open" } else { "row-detail-backdrop" }
            on:click=move |_| close.run(())
        />
        <div
            class=move || match (open.get(), wide) {
                (true, true) => "row-detail-panel wide open",
                (true, false) => "row-detail-panel open",
                (false, true) => "row-detail-panel wide",
                (false, false) => "row-detail-panel",
            }
            inert=move || !open.get()
        >
            <div class="row-detail-header">
                <h3>{move || title.get()}</h3>
                <button class="btn btn-ghost btn-sm" type="button" aria-label="Close drawer" node_ref=close_button on:click=move |_| close.run(())>
                    "\u{2715}"
                </button>
            </div>
            {if !subtitle.is_empty() {
                Some(view! { <p class="text-muted text-sm row-detail-subtitle">{subtitle}</p> })
            } else {
                None
            }}
            {context.map(|context| view! {
                <p class="text-muted text-sm row-detail-subtitle row-detail-context">{move || context.get()}</p>
            })}
            <div class="row-detail-body">
                {children()}
            </div>
        </div>
    }
}
