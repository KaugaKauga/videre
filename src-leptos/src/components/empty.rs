use leptos::prelude::*;

/// Shown when no tabs are open at all.
#[component]
pub fn EmptyState() -> impl IntoView {
    view! {
        <div class="empty-state">
            <div class="empty-state-inner">
                <svg class="empty-state-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"
                     fill="none" stroke="currentColor" stroke-width="1.5"
                     stroke-linecap="round" stroke-linejoin="round">
                    <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                    <ellipse cx="12" cy="6" rx="8" ry="3"/>
                    <path d="M4 6v6a8 3 0 0 0 16 0v-6"/>
                    <path d="M4 12v6a8 3 0 0 0 16 0v-6"/>
                </svg>
                <h3>"No table selected"</h3>
                <p>"Select a table from the sidebar to view its contents"</p>
            </div>
        </div>
    }
}

/// Shown inside an empty/blank tab.
#[component]
pub fn EmptyTab() -> impl IntoView {
    view! {
        <div class="empty-state">
            <div class="empty-state-inner">
                <svg class="empty-state-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"
                     fill="none" stroke="currentColor" stroke-width="1.5"
                     stroke-linecap="round" stroke-linejoin="round">
                    <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                    <path d="M14 3v4a1 1 0 0 0 1 1h4"/>
                    <path d="M17 21h-10a2 2 0 0 1 -2 -2v-14a2 2 0 0 1 2 -2h7l5 5v11a2 2 0 0 1 -2 2z"/>
                    <path d="M9 17h6"/>
                    <path d="M9 13h6"/>
                </svg>
                <h3>"Empty Tab"</h3>
                <p class="empty-tab-desc">
                    "This is an empty tab. Select a table from the sidebar to view its contents."
                </p>
                <div class="shortcuts-box">
                    <p class="shortcuts-title">"Keyboard Shortcuts:"</p>
                    <div class="shortcut-row">
                        <span>"New tab"</span>
                        <kbd>{"\u{2318}/Ctrl + T"}</kbd>
                    </div>
                    <div class="shortcut-row">
                        <span>"Close tab"</span>
                        <kbd>{"\u{2318}/Ctrl + W"}</kbd>
                    </div>
                    <div class="shortcut-row">
                        <span>"Switch to tab 1-9"</span>
                        <kbd>{"\u{2318}/Ctrl + [1-9]"}</kbd>
                    </div>
                </div>
            </div>
        </div>
    }
}
