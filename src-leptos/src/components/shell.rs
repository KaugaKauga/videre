use leptos::prelude::*;
use wasm_bindgen::prelude::*;
use wasm_bindgen::JsCast;

use crate::components::empty::{EmptyState, EmptyTab};
use crate::components::sidebar::{focus_sidebar, Sidebar};
use crate::components::tab_bar::TabBar;
use crate::pages::connection::ConnectionPage;
use crate::pages::indexes::IndexesPage;
use crate::pages::roles::RolesPage;
use crate::pages::settings::SettingsPage;
use crate::pages::table::TablePage;
use crate::stores::tab_store::{TabStore, TabType};

#[component]
pub fn Shell() -> impl IntoView {
    let tab_store = TabStore::init();
    provide_context(tab_store);

    // ---- Keyboard shortcuts ------------------------------------------------
    setup_keyboard_shortcuts(tab_store);

    // ---- View --------------------------------------------------------------
    view! {
        <div class="shell">
            <Sidebar />
            <div class="shell-main">
                <TabBar />
                <div class="shell-content">
                    {move || {
                        let active = tab_store.active_tab();
                        match active {
                            None => view! { <EmptyState /> }.into_any(),
                            Some(tab) => match &tab.tab_type {
                                TabType::Empty => {
                                    view! { <EmptyTab /> }.into_any()
                                }
                                TabType::Connection => {
                                    view! { <ConnectionPage /> }.into_any()
                                }
                                TabType::Table { name, schema } => {
                                    let n = name.clone();
                                    let s = schema.clone();
                                    view! {
                                        <TablePage name=n schema=s />
                                    }.into_any()
                                }
                                TabType::Settings => {
                                    view! { <SettingsPage /> }.into_any()
                                }
                                TabType::Indexes => {
                                    view! { <IndexesPage /> }.into_any()
                                }
                                TabType::Roles => {
                                    view! { <RolesPage /> }.into_any()
                                }
                            },
                        }
                    }}
                </div>
            </div>
        </div>
    }
}

/// `on_cleanup`. The event listener itself is properly removed on cleanup.
fn setup_keyboard_shortcuts(tab_store: TabStore) {
    let window = web_sys::window().expect("no global window");
    let is_mac = window
        .navigator()
        .platform()
        .unwrap_or_default()
        .contains("Mac");

    let cb = Closure::<dyn Fn(web_sys::KeyboardEvent)>::new(move |ev: web_sys::KeyboardEvent| {
        if ev.is_composing() || ev.default_prevented() {
            return;
        }
        let modifiers = KeyboardModifiers {
            ctrl: ev.ctrl_key(),
            meta: ev.meta_key(),
            alt: ev.alt_key(),
            shift: ev.shift_key(),
        };
        let Some(shortcut) = shell_shortcut(&ev.key(), modifiers, is_mac) else {
            return;
        };
        ev.prevent_default();
        match shortcut {
            ShellShortcut::OpenTab => tab_store.open_empty_tab(),
            ShellShortcut::CloseTab => tab_store.close_active_tab(),
            ShellShortcut::SwitchToTab(index) => tab_store.switch_to_tab(index),
            ShellShortcut::NextTab => tab_store.next_tab(),
            ShellShortcut::PreviousTab => tab_store.previous_tab(),
            ShellShortcut::FocusSidebar => focus_sidebar(),
        }
    });

    let js_fn: js_sys::Function = cb.as_ref().unchecked_ref::<js_sys::Function>().clone();
    cb.forget(); // leak closure so js_fn stays valid; on_cleanup removes the listener

    window
        .add_event_listener_with_callback("keydown", &js_fn)
        .expect("failed to add keydown listener");

    on_cleanup(move || {
        if let Some(w) = web_sys::window() {
            let _ = w.remove_event_listener_with_callback("keydown", &js_fn);
        }
    });
}

#[derive(Debug, PartialEq)]
enum ShellShortcut {
    OpenTab,
    CloseTab,
    SwitchToTab(usize),
    NextTab,
    PreviousTab,
    FocusSidebar,
}

#[derive(Clone, Copy, Default)]
struct KeyboardModifiers {
    ctrl: bool,
    meta: bool,
    alt: bool,
    shift: bool,
}

fn shell_shortcut(key: &str, modifiers: KeyboardModifiers, is_mac: bool) -> Option<ShellShortcut> {
    let KeyboardModifiers {
        ctrl,
        meta,
        alt,
        shift,
    } = modifiers;
    // Control-Tab is shared by both platforms; it is not Command-Tab on macOS.
    if key == "Tab" && ctrl && !meta && !alt {
        return Some(if shift {
            ShellShortcut::PreviousTab
        } else {
            ShellShortcut::NextTab
        });
    }
    if is_mac && meta && alt && !ctrl && !shift {
        return match key {
            "ArrowRight" => Some(ShellShortcut::NextTab),
            "ArrowLeft" => Some(ShellShortcut::PreviousTab),
            _ => None,
        };
    }
    let command = if is_mac { meta && !ctrl } else { ctrl && !meta };
    if !command || alt {
        return None;
    }
    match (key, shift) {
        ("e" | "E", true) => Some(ShellShortcut::FocusSidebar),
        ("t", false) => Some(ShellShortcut::OpenTab),
        ("w", false) => Some(ShellShortcut::CloseTab),
        (key, false) => key
            .parse::<usize>()
            .ok()
            .filter(|index| (1..=9).contains(index))
            .map(|index| ShellShortcut::SwitchToTab(index - 1)),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{shell_shortcut, KeyboardModifiers, ShellShortcut};

    fn command_modifiers(is_mac: bool, shift: bool) -> KeyboardModifiers {
        KeyboardModifiers {
            ctrl: !is_mac,
            meta: is_mac,
            shift,
            ..Default::default()
        }
    }

    #[test]
    fn sidebar_shortcut_requires_shift_and_accepts_browser_key_casing() {
        for is_mac in [false, true] {
            assert_eq!(
                shell_shortcut("e", command_modifiers(is_mac, false), is_mac),
                None
            );
            for key in ["e", "E"] {
                assert_eq!(
                    shell_shortcut(key, command_modifiers(is_mac, true), is_mac),
                    Some(ShellShortcut::FocusSidebar),
                );
            }
        }
    }

    #[test]
    fn existing_tab_shortcuts_use_the_platform_command_modifier() {
        for is_mac in [false, true] {
            let modifiers = command_modifiers(is_mac, false);
            assert_eq!(
                shell_shortcut("t", modifiers, is_mac),
                Some(ShellShortcut::OpenTab)
            );
            assert_eq!(
                shell_shortcut("w", modifiers, is_mac),
                Some(ShellShortcut::CloseTab)
            );
            assert_eq!(
                shell_shortcut("t", command_modifiers(!is_mac, false), is_mac),
                None
            );
            for index in 1..=9 {
                assert_eq!(
                    shell_shortcut(&index.to_string(), modifiers, is_mac),
                    Some(ShellShortcut::SwitchToTab(index - 1)),
                );
            }
        }
    }

    #[test]
    fn other_keys_and_shifted_tab_shortcuts_are_not_intercepted() {
        for is_mac in [false, true] {
            for key in [
                "c",
                "j",
                "k",
                "ArrowDown",
                "ArrowUp",
                "ArrowLeft",
                "ArrowRight",
                "0",
                "Escape",
            ] {
                assert_eq!(
                    shell_shortcut(key, command_modifiers(is_mac, false), is_mac),
                    None
                );
            }
            for key in ["t", "w", "1"] {
                assert_eq!(
                    shell_shortcut(key, command_modifiers(is_mac, true), is_mac),
                    None
                );
            }
        }
    }

    #[test]
    fn control_tab_cycles_on_both_platforms_without_extra_modifiers() {
        let ctrl = KeyboardModifiers {
            ctrl: true,
            ..Default::default()
        };
        for is_mac in [false, true] {
            assert_eq!(
                shell_shortcut("Tab", ctrl, is_mac),
                Some(ShellShortcut::NextTab)
            );
            assert_eq!(
                shell_shortcut(
                    "Tab",
                    KeyboardModifiers {
                        shift: true,
                        ..ctrl
                    },
                    is_mac
                ),
                Some(ShellShortcut::PreviousTab),
            );
            for modifiers in [
                KeyboardModifiers::default(),
                KeyboardModifiers {
                    shift: true,
                    ..Default::default()
                },
                KeyboardModifiers {
                    meta: true,
                    ..Default::default()
                },
                KeyboardModifiers { meta: true, ..ctrl },
                KeyboardModifiers { alt: true, ..ctrl },
            ] {
                assert_eq!(shell_shortcut("Tab", modifiers, is_mac), None);
            }
        }
    }

    #[test]
    fn command_option_arrows_cycle_only_on_macos() {
        let cmd_option = KeyboardModifiers {
            meta: true,
            alt: true,
            ..Default::default()
        };
        assert_eq!(
            shell_shortcut("ArrowRight", cmd_option, true),
            Some(ShellShortcut::NextTab)
        );
        assert_eq!(
            shell_shortcut("ArrowLeft", cmd_option, true),
            Some(ShellShortcut::PreviousTab)
        );
        for key in ["ArrowLeft", "ArrowRight"] {
            assert_eq!(shell_shortcut(key, cmd_option, false), None);
            for modifiers in [
                KeyboardModifiers {
                    meta: true,
                    ..Default::default()
                },
                KeyboardModifiers {
                    shift: true,
                    ..cmd_option
                },
                KeyboardModifiers {
                    ctrl: true,
                    ..cmd_option
                },
            ] {
                assert_eq!(shell_shortcut(key, modifiers, true), None);
            }
        }
    }

    #[test]
    fn altgr_and_unmodified_typing_are_not_app_shortcuts() {
        for is_mac in [false, true] {
            for key in ["t", "w", "e", "1", "c"] {
                assert_eq!(
                    shell_shortcut(key, KeyboardModifiers::default(), is_mac),
                    None
                );
                assert_eq!(
                    shell_shortcut(
                        key,
                        KeyboardModifiers {
                            ctrl: true,
                            alt: true,
                            ..Default::default()
                        },
                        is_mac
                    ),
                    None,
                );
            }
        }
    }
}
