use std::collections::HashMap;

use leptos::prelude::*;

use crate::stores::QuerySession;

// ---------------------------------------------------------------------------
// Tab types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum TabType {
    Table { name: String, schema: String },
    Empty,
    Query,
    Settings,
    Connection,
    Indexes,
    Roles,
}

#[derive(Clone, Debug)]
pub struct Tab {
    pub id: String,
    pub label: String,
    pub tab_type: TabType,
}

// ---------------------------------------------------------------------------
// ID generation (same approach as connection_store.rs)
// ---------------------------------------------------------------------------

fn generate_id() -> String {
    let now = js_sys::Date::now() as u64;
    let rand = (js_sys::Math::random() * 999_999_999.0) as u64;
    format!("{:x}-{:x}", now, rand)
}

// ---------------------------------------------------------------------------
// TabStore — provided via Leptos context
// ---------------------------------------------------------------------------

/// Reactive tab state.
///
/// Provided via `provide_context` in the `Shell` component.
/// Any child can grab it with `use_context::<TabStore>().unwrap()`.
#[derive(Clone, Copy)]
pub struct TabStore {
    pub tabs: RwSignal<Vec<Tab>>,
    pub active_tab_id: RwSignal<Option<String>>,
    queries: RwSignal<HashMap<String, QuerySession>>,
}

impl TabStore {
    pub fn init() -> Self {
        Self {
            tabs: RwSignal::new(Vec::new()),
            active_tab_id: RwSignal::new(None),
            queries: RwSignal::new(HashMap::new()),
        }
    }

    // -- Derived readers ---------------------------------------------------

    /// Return a clone of the currently active tab (if any).
    pub fn active_tab(&self) -> Option<Tab> {
        let id = self.active_tab_id.get()?;
        self.tabs.get().into_iter().find(|t| t.id == id)
    }

    // -- Tab openers -------------------------------------------------------

    /// Open a table tab.
    ///
    /// 1. If a tab for this table already exists → focus it.
    /// 2. Else if the active tab is empty → reuse it.
    /// 3. Else → create a new tab.
    pub fn open_table_tab(&self, name: String, schema: String) {
        let mut tabs = self.tabs.get_untracked();
        let active_id = self.active_tab_id.get_untracked();

        // 1. Already open?
        if let Some(existing) = tabs.iter().find(|t| {
            matches!(
                &t.tab_type,
                TabType::Table { name: n, schema: s } if *n == name && *s == schema
            )
        }) {
            self.active_tab_id.set(Some(existing.id.clone()));
            return;
        }

        // 2. Reuse active empty tab?
        if let Some(ref aid) = active_id {
            if let Some(tab) = tabs.iter_mut().find(|t| &t.id == aid) {
                if tab.tab_type == TabType::Empty {
                    tab.label = name.clone();
                    tab.tab_type = TabType::Table { name, schema };
                    self.tabs.set(tabs);
                    return;
                }
            }
        }

        // 3. New tab
        let id = generate_id();
        tabs.push(Tab {
            id: id.clone(),
            label: name.clone(),
            tab_type: TabType::Table { name, schema },
        });
        self.tabs.set(tabs);
        self.active_tab_id.set(Some(id));
    }

    /// Open a singleton tab (settings, connection, indexes, roles).
    ///
    /// Only one of each type may exist at a time.
    /// Follows the same reuse-empty-tab logic as `open_table_tab`.
    pub fn open_singleton_tab(&self, tab_type: TabType, label: &str) {
        let mut tabs = self.tabs.get_untracked();
        let active_id = self.active_tab_id.get_untracked();

        // Already open?
        if let Some(existing) = tabs
            .iter()
            .find(|t| std::mem::discriminant(&t.tab_type) == std::mem::discriminant(&tab_type))
        {
            self.active_tab_id.set(Some(existing.id.clone()));
            return;
        }

        // Reuse active empty tab?
        if let Some(ref aid) = active_id {
            if let Some(tab) = tabs.iter_mut().find(|t| &t.id == aid) {
                if tab.tab_type == TabType::Empty {
                    tab.label = label.to_string();
                    tab.tab_type = tab_type;
                    self.tabs.set(tabs);
                    return;
                }
            }
        }

        // New tab
        let id = generate_id();
        tabs.push(Tab {
            id: id.clone(),
            label: label.to_string(),
            tab_type,
        });
        self.tabs.set(tabs);
        self.active_tab_id.set(Some(id));
    }

    /// Open a new empty tab named "Untitled N" (lowest available number).
    pub fn open_empty_tab(&self) {
        let mut tabs = self.tabs.get_untracked();

        let used: Vec<u32> = tabs
            .iter()
            .filter_map(|t| {
                if t.tab_type == TabType::Empty {
                    t.label
                        .strip_prefix("Untitled ")
                        .and_then(|n| n.parse().ok())
                } else {
                    None
                }
            })
            .collect();

        let mut n = 1u32;
        while used.contains(&n) {
            n += 1;
        }

        let id = generate_id();
        tabs.push(Tab {
            id: id.clone(),
            label: format!("Untitled {n}"),
            tab_type: TabType::Empty,
        });
        self.tabs.set(tabs);
        self.active_tab_id.set(Some(id));
    }

    /// Open an independent SQL draft, reusing an active empty tab when possible.
    pub fn open_query_tab(&self) {
        let mut tabs = self.tabs.get_untracked();
        let active_id = self.active_tab_id.get_untracked();
        let mut number = 1;
        while tabs
            .iter()
            .any(|tab| tab.tab_type == TabType::Query && tab.label == format!("Query {number}"))
        {
            number += 1;
        }
        let label = format!("Query {number}");
        if let Some(tab) = tabs
            .iter_mut()
            .find(|tab| Some(&tab.id) == active_id.as_ref() && tab.tab_type == TabType::Empty)
        {
            tab.label = label;
            tab.tab_type = TabType::Query;
            let id = tab.id.clone();
            self.queries.update(|queries| {
                queries.insert(id, QuerySession::new());
            });
            self.tabs.set(tabs);
            return;
        }
        let id = generate_id();
        self.queries.update(|queries| {
            queries.insert(id.clone(), QuerySession::new());
        });
        tabs.push(Tab {
            id: id.clone(),
            label,
            tab_type: TabType::Query,
        });
        self.tabs.set(tabs);
        self.active_tab_id.set(Some(id));
    }

    pub fn query_session(&self, tab_id: &str) -> Option<QuerySession> {
        self.queries
            .with_untracked(|queries| queries.get(tab_id).cloned())
    }

    pub fn close_queries(&self) {
        self.queries.with_untracked(|queries| {
            for session in queries.values() {
                session.close();
            }
        });
    }

    // -- Tab actions -------------------------------------------------------

    /// Close a tab by ID.
    ///
    /// If it was the active tab the last remaining tab becomes active
    /// (matches the React behaviour).
    pub fn close_tab(&self, tab_id: &str) {
        let mut tabs = self.tabs.get_untracked();
        let active_id = self.active_tab_id.get_untracked();

        if let Some(session) = self.query_session(tab_id) {
            session.close();
            self.queries.update(|queries| {
                queries.remove(tab_id);
            });
        }
        tabs.retain(|t| t.id != tab_id);

        if active_id.as_deref() == Some(tab_id) {
            let new_active = tabs.last().map(|t| t.id.clone());
            self.active_tab_id.set(new_active);
        }

        self.tabs.set(tabs);
    }

    /// Close the currently active tab (keyboard shortcut helper).
    pub fn close_active_tab(&self) {
        if let Some(id) = self.active_tab_id.get_untracked() {
            self.close_tab(&id);
        }
    }

    /// Activate a tab by click.
    pub fn set_active(&self, tab_id: &str) {
        self.active_tab_id.set(Some(tab_id.to_string()));
    }

    /// Activate the next tab in visible order, wrapping to the first.
    pub fn next_tab(&self) {
        self.cycle_tab(false);
    }

    /// Activate the previous tab in visible order, wrapping to the last.
    pub fn previous_tab(&self) {
        self.cycle_tab(true);
    }

    fn cycle_tab(&self, backwards: bool) {
        let tabs = self.tabs.get_untracked();
        let active_id = self.active_tab_id.get_untracked();
        if let Some(index) = adjacent_tab_index(&tabs, active_id.as_deref(), backwards) {
            self.active_tab_id.set(Some(tabs[index].id.clone()));
        }
    }

    /// Switch to a tab by 0-based index (for Cmd/Ctrl + 1-9 shortcuts).
    pub fn switch_to_tab(&self, index: usize) {
        let tabs = self.tabs.get_untracked();
        if let Some(tab) = tabs.get(index) {
            self.active_tab_id.set(Some(tab.id.clone()));
        }
    }
}

fn adjacent_tab_index(tabs: &[Tab], active_id: Option<&str>, backwards: bool) -> Option<usize> {
    if tabs.is_empty() {
        return None;
    }
    let current = active_id.and_then(|id| tabs.iter().position(|tab| tab.id == id));
    Some(match (current, backwards) {
        (Some(0) | None, true) => tabs.len() - 1,
        (Some(index), true) => index - 1,
        (Some(index), false) => (index + 1) % tabs.len(),
        (None, false) => 0,
    })
}

#[cfg(test)]
mod tests {
    use super::{adjacent_tab_index, Tab, TabStore, TabType};
    use leptos::prelude::*;

    fn tabs() -> Vec<Tab> {
        ["first", "second", "third"]
            .into_iter()
            .map(|id| Tab {
                id: id.into(),
                label: id.into(),
                tab_type: TabType::Empty,
            })
            .collect()
    }

    #[test]
    fn query_drafts_survive_switching_and_closing_releases_only_that_session() {
        Owner::new().with(|| {
            let store = TabStore::init();
            store.tabs.set(tabs());
            let first = super::QuerySession::new();
            let second = super::QuerySession::new();
            first.sql.set("SELECT 1".into());
            second.sql.set("VALUES (2)".into());
            store.queries.update(|queries| {
                queries.insert("first".into(), first.clone());
                queries.insert("second".into(), second.clone());
            });
            store.set_active("first");
            store.set_active("second");
            assert_eq!(
                store.query_session("first").unwrap().sql.get_untracked(),
                "SELECT 1"
            );
            store.close_tab("first");
            assert!(first.closed.get_untracked());
            assert!(store.query_session("first").is_none());
            assert!(!second.closed.get_untracked());
            store.close_queries();
            assert!(second.closed.get_untracked());
        });
    }

    #[test]
    fn cycles_in_visible_order_and_wraps_in_both_directions() {
        let tabs = tabs();
        for (index, tab) in tabs.iter().enumerate() {
            assert_eq!(
                adjacent_tab_index(&tabs, Some(&tab.id), false),
                Some((index + 1) % tabs.len()),
            );
            assert_eq!(
                adjacent_tab_index(&tabs, Some(&tab.id), true),
                Some((index + tabs.len() - 1) % tabs.len()),
            );
        }
    }

    #[test]
    fn no_tabs_is_a_noop_and_one_tab_stays_selected() {
        for backwards in [false, true] {
            assert_eq!(adjacent_tab_index(&[], None, backwards), None);
            assert_eq!(
                adjacent_tab_index(&tabs()[..1], Some("first"), backwards),
                Some(0),
            );
        }
    }

    #[test]
    fn missing_or_stale_active_tab_selects_the_directional_boundary() {
        for active in [None, Some("closed")] {
            assert_eq!(adjacent_tab_index(&tabs(), active, false), Some(0));
            assert_eq!(adjacent_tab_index(&tabs(), active, true), Some(2));
        }
    }

    #[test]
    fn cycling_updates_the_store_and_skips_closed_tabs() {
        Owner::new().with(|| {
            let store = TabStore::init();
            store.tabs.set(tabs());
            store.set_active("first");
            store.next_tab();
            assert_eq!(
                store.active_tab_id.get_untracked().as_deref(),
                Some("second")
            );
            store.close_tab("second");
            store.previous_tab();
            assert_eq!(
                store.active_tab_id.get_untracked().as_deref(),
                Some("first")
            );
            store.previous_tab();
            assert_eq!(
                store.active_tab_id.get_untracked().as_deref(),
                Some("third")
            );
        });
    }
}
