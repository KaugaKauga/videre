use std::collections::HashMap;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::tauri;
use crate::types::{ForeignKeyInfo, IndexInfo, RoleInfo, TableInfo, TablePrivilege};

type ForeignKeyMap = HashMap<String, Vec<ForeignKeyInfo>>;
type IndexMap = HashMap<String, Vec<IndexInfo>>;

/// Global reactive store for database metadata.
///
/// Provided via Leptos context in `App`. Any component can access it with:
///   `let db = use_context::<DbStore>().unwrap();`
///
/// Signals are fine-grained — only the DOM that reads a specific signal
/// re-renders when that signal changes.
#[derive(Clone, Copy)]
pub struct DbStore {
    pub is_connected: RwSignal<bool>,
    pub tables: RwSignal<Vec<TableInfo>>,
    pub foreign_keys: RwSignal<ForeignKeyMap>,
    pub indexes: RwSignal<IndexMap>,
    pub roles: RwSignal<Vec<RoleInfo>>,
    pub table_privileges: RwSignal<Vec<TablePrivilege>>,
    pub is_loading: RwSignal<bool>,
    pub error: RwSignal<Option<String>>,
}

impl DbStore {
    /// Create the store with empty defaults. No async work happens here.
    pub fn init() -> Self {
        Self {
            is_connected: RwSignal::new(false),
            tables: RwSignal::new(Vec::new()),
            foreign_keys: RwSignal::new(HashMap::new()),
            indexes: RwSignal::new(HashMap::new()),
            roles: RwSignal::new(Vec::new()),
            table_privileges: RwSignal::new(Vec::new()),
            is_loading: RwSignal::new(false),
            error: RwSignal::new(None),
        }
    }

    /// Fetch all database metadata from the Tauri backend.
    ///
    /// Calls `get_tables`, then iterates each table to fetch its foreign keys
    /// and indexes, plus `get_roles` and `get_table_privileges`.
    ///
    /// All IPC calls are local (in-process via Tauri) so sequential awaits
    /// are effectively instant — no network round trips.
    pub fn fetch_database_metadata(self) {
        self.is_loading.set(true);
        self.error.set(None);

        spawn_local(async move {
            match Self::do_fetch().await {
                Ok((tables, fk_map, idx_map, roles, privileges)) => {
                    self.tables.set(tables);
                    self.foreign_keys.set(fk_map);
                    self.indexes.set(idx_map);
                    self.roles.set(roles);
                    self.table_privileges.set(privileges);
                    self.is_connected.set(true);
                    self.is_loading.set(false);
                }
                Err(e) => {
                    web_sys::console::error_1(
                        &format!("Failed to fetch database metadata: {e}").into(),
                    );
                    self.error
                        .set(Some(format!("Failed to fetch database metadata: {e}")));
                    self.is_connected.set(false);
                    self.is_loading.set(false);
                }
            }
        });
    }

    /// Inner async function that does the actual IPC calls.
    /// Returns all fetched data or the first error encountered.
    async fn do_fetch() -> Result<
        (
            Vec<TableInfo>,
            ForeignKeyMap,
            IndexMap,
            Vec<RoleInfo>,
            Vec<TablePrivilege>,
        ),
        String,
    > {
        // 1. Fetch the table list first — everything else depends on it.
        let tables: Vec<TableInfo> = tauri::invoke("get_tables", &serde_json::json!({})).await?;

        // 2. For each table, fetch foreign keys and indexes.
        let mut fk_map: ForeignKeyMap = HashMap::new();
        let mut idx_map: IndexMap = HashMap::new();

        for table in &tables {
            let key = format!("{}.{}", table.schema, table.name);

            let fks: Vec<ForeignKeyInfo> = tauri::invoke(
                "get_foreign_keys",
                &serde_json::json!({
                    "tableName": table.name,
                    "schema": table.schema,
                }),
            )
            .await?;
            fk_map.insert(key.clone(), fks);

            let idxs: Vec<IndexInfo> = tauri::invoke(
                "get_indexes",
                &serde_json::json!({
                    "tableName": table.name,
                    "schema": table.schema,
                }),
            )
            .await?;
            idx_map.insert(key, idxs);
        }

        // 3. Fetch roles and table privileges.
        let roles: Vec<RoleInfo> = tauri::invoke("get_roles", &serde_json::json!({})).await?;

        let privileges: Vec<TablePrivilege> =
            tauri::invoke("get_table_privileges", &serde_json::json!({})).await?;

        Ok((tables, fk_map, idx_map, roles, privileges))
    }

    /// Disconnect from the database and reset all state to defaults.
    pub fn disconnect(self) {
        spawn_local(async move {
            if let Err(e) = tauri::invoke_void("disconnect_db", &serde_json::json!({})).await {
                web_sys::console::error_1(&format!("Failed to disconnect: {e}").into());
                self.error.set(Some(format!("Failed to disconnect: {e}")));
                return;
            }

            self.is_connected.set(false);
            self.tables.set(Vec::new());
            self.foreign_keys.set(HashMap::new());
            self.indexes.set(HashMap::new());
            self.roles.set(Vec::new());
            self.table_privileges.set(Vec::new());
            self.error.set(None);
        });
    }

    // -- Lookup helpers --------------------------------------------------
    //
    // These use `.get_untracked()` because they are called from component
    // bodies as one-shot reads (e.g. building an FK map when a TablePage
    // mounts).  The metadata is static for the lifetime of a connection so
    // reactive tracking is not needed.

    /// Get foreign keys for a specific table.
    pub fn get_foreign_keys_for_table(
        &self,
        table_name: &str,
        schema: &str,
    ) -> Vec<ForeignKeyInfo> {
        let key = format!("{schema}.{table_name}");
        self.foreign_keys
            .get_untracked()
            .get(&key)
            .cloned()
            .unwrap_or_default()
    }

    /// Get indexes for a specific table.
    pub fn get_indexes_for_table(&self, table_name: &str, schema: &str) -> Vec<IndexInfo> {
        let key = format!("{schema}.{table_name}");
        self.indexes
            .get_untracked()
            .get(&key)
            .cloned()
            .unwrap_or_default()
    }

    /// Get table privileges for a specific role.
    pub fn get_privileges_for_role(&self, role_name: &str) -> Vec<TablePrivilege> {
        self.table_privileges
            .get_untracked()
            .into_iter()
            .filter(|p| p.grantee == role_name)
            .collect()
    }
}
