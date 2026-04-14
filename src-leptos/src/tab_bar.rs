use leptos::prelude::*;

use crate::icons;
use crate::tab_store::TabStore;

/// Horizontal tab strip rendered above the content area.
///
/// Each tab shows its label + a close button (visible on hover).
/// The active tab gets a bottom accent bar and a distinct background.
#[component]
pub fn TabBar() -> impl IntoView {
    let tab_store = use_context::<TabStore>().expect("TabStore not provided");

    move || {
        let tabs = tab_store.tabs.get();
        let active_id = tab_store.active_tab_id.get();

        if tabs.is_empty() {
            return view! { <div class="tab-bar tab-bar-empty"/> }.into_any();
        }

        let items: Vec<_> = tabs
            .iter()
            .map(|tab| {
                let id = tab.id.clone();
                let id_click = id.clone();
                let id_close = id.clone();
                let label = tab.label.clone();
                let is_active = active_id.as_deref() == Some(&id);
                let class = if is_active { "tab active" } else { "tab" };

                view! {
                    <div
                        class=class
                        on:click=move |_| tab_store.set_active(&id_click)
                    >
                        <span class="tab-label">{label}</span>
                        <button
                            class="tab-close"
                            title="Close"
                            on:click=move |ev| {
                                ev.stop_propagation();
                                tab_store.close_tab(&id_close);
                            }
                        >
                            {icons::icon_x(12)}
                        </button>
                        {if is_active {
                            Some(view! { <div class="tab-indicator"/> })
                        } else {
                            None
                        }}
                    </div>
                }
            })
            .collect();

        view! { <div class="tab-bar">{items}</div> }.into_any()
    }
}
