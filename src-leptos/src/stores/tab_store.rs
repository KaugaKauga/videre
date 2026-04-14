use leptos::prelude::*;

// ---------------------------------------------------------------------------
// Tab types
// ---------------------------------------------------------------------------

#[derive(Clone, Debug, PartialEq)]
pub enum TabType {
    Table { name: String, schema: String },
    Empty,
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
}

impl TabStore {
    pub fn init() -> Self {
        Self {
            tabs: RwSignal::new(Vec::new()),
            active_tab_id: RwSignal::new(None),
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

    // -- Tab actions -------------------------------------------------------

    /// Close a tab by ID.
    ///
    /// If it was the active tab the last remaining tab becomes active
    /// (matches the React behaviour).
    pub fn close_tab(&self, tab_id: &str) {
        let mut tabs = self.tabs.get_untracked();
        let active_id = self.active_tab_id.get_untracked();

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

    /// Switch to a tab by 0-based index (for Cmd/Ctrl + 1-9 shortcuts).
    pub fn switch_to_tab(&self, index: usize) {
        let tabs = self.tabs.get_untracked();
        if let Some(tab) = tabs.get(index) {
            self.active_tab_id.set(Some(tab.id.clone()));
        }
    }
}
