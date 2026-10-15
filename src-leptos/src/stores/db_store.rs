use std::collections::HashMap;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::tauri;
use crate::types::{
    ForeignKeyInfo, IndexInfo, RelationStructure, RelationStructureKey, RoleInfo, TableInfo,
    TablePrivilege,
};

type ForeignKeyMap = HashMap<String, Vec<ForeignKeyInfo>>;
type IndexMap = HashMap<String, Vec<IndexInfo>>;

/// Key of a table in the foreign-key and index maps.
pub fn table_key(schema: &str, table: &str) -> String {
    format!("{schema}.{table}")
}

/// A failed lazy fetch is not retried on every keystroke, but a transient error
/// must not disable a relation until reconnecting either.
const FAILED_REQUEST_RETRY_MS: f64 = 15_000.0;

#[derive(Default)]
struct RelationStructureRequests {
    generation: u64,
    next_request: u64,
    pending: HashMap<RelationStructureKey, u64>,
    failures: HashMap<RelationStructureKey, RelationStructureFailure>,
}

struct RelationStructureFailure {
    message: String,
    failed_at_ms: f64,
}

/// What a drawer or editor can currently show for one relation.
#[derive(Clone, Debug, PartialEq)]
pub enum RelationStructureStatus {
    Ready(RelationStructure),
    Loading,
    Failed(String),
    /// Not cached and not requested in the current connection generation.
    Missing,
}

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
    pub relation_structures: RwSignal<HashMap<RelationStructureKey, RelationStructure>>,
    /// Reactive so open drawers follow loading, failure and reconnect resets.
    relation_structure_requests: RwSignal<RelationStructureRequests>,
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
            relation_structures: RwSignal::new(HashMap::new()),
            relation_structure_requests: RwSignal::new(RelationStructureRequests::default()),
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
        let Some(generation) = self.reset_relation_structures() else {
            return;
        };
        self.is_loading.set(true);
        self.error.set(None);

        spawn_local(async move {
            let result = Self::do_fetch().await;
            if !self.is_current_metadata_generation(generation) {
                return;
            }
            match result {
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
            let key = table_key(&table.schema, &table.name);

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
    #[allow(dead_code)]
    pub fn disconnect(self) {
        let Some(generation) = self
            .relation_structure_requests
            .try_with_untracked(|requests| requests.generation)
        else {
            return;
        };
        spawn_local(async move {
            let result = tauri::invoke_void("disconnect_db", &serde_json::json!({})).await;
            if !self.is_current_metadata_generation(generation) {
                return;
            }
            if let Err(e) = result {
                web_sys::console::error_1(&format!("Failed to disconnect: {e}").into());
                self.error.set(Some(format!("Failed to disconnect: {e}")));
                return;
            }

            // Supersedes any metadata fetch still in flight, which therefore
            // can no longer clear the loading state itself.
            self.reset_relation_structures();
            self.is_loading.set(false);
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
    // These use `.get_untracked()` when called while a page mounts to build
    // static metadata maps. Relation structures are an exception: an open
    // drawer or editor tracks the cache so lazy fetches and resets refresh it.

    /// Get foreign keys for a specific table.
    pub fn get_foreign_keys_for_table(
        &self,
        table_name: &str,
        schema: &str,
    ) -> Vec<ForeignKeyInfo> {
        let key = table_key(schema, table_name);
        self.foreign_keys
            .get_untracked()
            .get(&key)
            .cloned()
            .unwrap_or_default()
    }

    /// Tracked cache and request state for one relation.
    pub fn relation_structure_status(
        &self,
        schema: &str,
        relation: &str,
    ) -> RelationStructureStatus {
        let key = RelationStructureKey::new(schema, relation);
        if let Some(structure) = self
            .relation_structures
            .with(|structures| structures.get(&key).cloned())
        {
            return RelationStructureStatus::Ready(structure);
        }
        self.relation_structure_requests.with(|requests| {
            if requests.pending.contains_key(&key) {
                RelationStructureStatus::Loading
            } else if let Some(failure) = requests.failures.get(&key) {
                RelationStructureStatus::Failed(failure.message.clone())
            } else {
                RelationStructureStatus::Missing
            }
        })
    }

    /// Lazily request missing metadata without subscribing to the cache.
    /// Cached and pending requests are skipped; failures are retried after a cooldown.
    pub fn request_relation_structure(&self, schema: &str, relation: &str) {
        self.start_relation_structure_request(
            RelationStructureKey::new(schema, relation),
            js_sys::Date::now(),
        );
    }

    /// Explicit user retry: ignores the failure cooldown.
    pub fn retry_relation_structure(&self, schema: &str, relation: &str) {
        self.start_relation_structure_request(
            RelationStructureKey::new(schema, relation),
            f64::INFINITY,
        );
    }

    fn start_relation_structure_request(&self, key: RelationStructureKey, now_ms: f64) {
        let Some(request) = self.begin_relation_structure_request(&key, now_ms) else {
            return;
        };
        let store = *self;
        spawn_local(async move {
            let result = tauri::get_relation_structure(&key.relation, &key.schema).await;
            store.finish_relation_structure_request(key, request, result, js_sys::Date::now());
        });
    }

    fn begin_relation_structure_request(
        &self,
        key: &RelationStructureKey,
        now_ms: f64,
    ) -> Option<(u64, u64)> {
        if self
            .relation_structures
            .try_with_untracked(|structures| structures.contains_key(key))?
        {
            return None;
        }
        // Check before writing so repeated keystrokes do not notify drawers.
        let blocked = self
            .relation_structure_requests
            .try_with_untracked(|requests| {
                requests.pending.contains_key(key)
                    || requests.failures.get(key).is_some_and(|failure| {
                        now_ms - failure.failed_at_ms < FAILED_REQUEST_RETRY_MS
                    })
            })?;
        if blocked {
            return None;
        }
        self.relation_structure_requests.try_update(|requests| {
            requests.failures.remove(key);
            requests.next_request = requests.next_request.wrapping_add(1);
            requests.pending.insert(key.clone(), requests.next_request);
            (requests.generation, requests.next_request)
        })
    }

    fn finish_relation_structure_request(
        &self,
        key: RelationStructureKey,
        (generation, request): (u64, u64),
        result: Result<RelationStructure, String>,
        now_ms: f64,
    ) {
        let current = self
            .relation_structure_requests
            .try_with_untracked(|requests| {
                requests.generation == generation && requests.pending.get(&key) == Some(&request)
            })
            .unwrap_or(false);
        if !current {
            return;
        }
        self.relation_structure_requests.update(|requests| {
            requests.pending.remove(&key);
            if let Err(message) = &result {
                requests.failures.insert(
                    key.clone(),
                    RelationStructureFailure {
                        message: message.clone(),
                        failed_at_ms: now_ms,
                    },
                );
            }
        });
        if let Ok(structure) = result {
            self.relation_structures.update(|structures| {
                structures.insert(key, structure);
            });
        }
    }

    fn reset_relation_structures(&self) -> Option<u64> {
        let generation = self.relation_structure_requests.try_update(|requests| {
            requests.generation = requests.generation.wrapping_add(1);
            requests.pending.clear();
            requests.failures.clear();
            requests.generation
        })?;
        self.relation_structures
            .update(|structures| structures.clear());
        Some(generation)
    }

    fn is_current_metadata_generation(&self, generation: u64) -> bool {
        self.relation_structure_requests
            .try_with_untracked(|requests| requests.generation == generation)
            .unwrap_or(false)
    }

    /// Get indexes for a specific table.
    #[allow(dead_code)]
    pub fn get_indexes_for_table(&self, table_name: &str, schema: &str) -> Vec<IndexInfo> {
        let key = table_key(schema, table_name);
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

#[cfg(test)]
mod tests {
    use super::*;

    fn structure(schema: &str, relation: &str) -> RelationStructure {
        RelationStructure {
            schema: schema.into(),
            relation: relation.into(),
            relation_kind: "table".into(),
            columns: Vec::new(),
        }
    }

    #[test]
    fn requests_are_deduplicated_and_schema_scoped() {
        Owner::new().with(|| {
            untrack(|| {
                let db = DbStore::init();
                let key = RelationStructureKey::new("public", "orders");
                assert_eq!(
                    db.relation_structure_status("public", "orders"),
                    RelationStructureStatus::Missing
                );
                let request = db.begin_relation_structure_request(&key, 0.0).unwrap();
                assert_eq!(
                    db.relation_structure_status("public", "orders"),
                    RelationStructureStatus::Loading
                );
                assert!(db.begin_relation_structure_request(&key, 0.0).is_none());
                let archive = RelationStructureKey::new("archive", "orders");
                assert!(db.begin_relation_structure_request(&archive, 0.0).is_some());

                db.finish_relation_structure_request(
                    key.clone(),
                    request,
                    Ok(structure("public", "orders")),
                    0.0,
                );
                assert_eq!(
                    db.relation_structure_status("public", "orders"),
                    RelationStructureStatus::Ready(structure("public", "orders"))
                );
                assert_eq!(
                    db.relation_structure_status("archive", "orders"),
                    RelationStructureStatus::Loading
                );
                assert!(db.begin_relation_structure_request(&key, 0.0).is_none());
            })
        });
    }

    #[test]
    fn failures_are_retried_after_cooldown_or_explicit_retry() {
        Owner::new().with(|| {
            untrack(|| {
                let db = DbStore::init();
                let key = RelationStructureKey::new("public", "orders");
                let request = db.begin_relation_structure_request(&key, 0.0).unwrap();
                db.finish_relation_structure_request(
                    key.clone(),
                    request,
                    Err("unavailable".into()),
                    1_000.0,
                );
                assert_eq!(
                    db.relation_structure_status("public", "orders"),
                    RelationStructureStatus::Failed("unavailable".into())
                );
                let cooling = 1_000.0 + FAILED_REQUEST_RETRY_MS - 1.0;
                assert!(db.begin_relation_structure_request(&key, cooling).is_none());

                let retry = db
                    .begin_relation_structure_request(&key, 1_000.0 + FAILED_REQUEST_RETRY_MS)
                    .unwrap();
                assert_eq!(
                    db.relation_structure_status("public", "orders"),
                    RelationStructureStatus::Loading
                );
                db.finish_relation_structure_request(
                    key.clone(),
                    retry,
                    Err("still unavailable".into()),
                    20_000.0,
                );
                // An explicit retry ignores the cooldown.
                assert!(db
                    .begin_relation_structure_request(&key, f64::INFINITY)
                    .is_some());
            })
        });
    }

    #[test]
    fn connection_reset_clears_state_and_rejects_stale_generations() {
        Owner::new().with(|| {
            untrack(|| {
                let db = DbStore::init();
                let cached_key = RelationStructureKey::new("public", "cached");
                let cached = db
                    .begin_relation_structure_request(&cached_key, 0.0)
                    .unwrap();
                db.finish_relation_structure_request(
                    cached_key,
                    cached,
                    Ok(structure("public", "cached")),
                    0.0,
                );
                let failed_key = RelationStructureKey::new("public", "failed");
                let failed = db
                    .begin_relation_structure_request(&failed_key, 0.0)
                    .unwrap();
                db.finish_relation_structure_request(
                    failed_key.clone(),
                    failed,
                    Err("unavailable".into()),
                    0.0,
                );
                let key = RelationStructureKey::new("public", "orders");
                let old = db.begin_relation_structure_request(&key, 0.0).unwrap();

                let generation = db.reset_relation_structures().unwrap();
                assert_ne!(generation, old.0);
                assert!(!db.is_current_metadata_generation(old.0));
                assert!(db.is_current_metadata_generation(generation));
                assert!(db.relation_structures.with_untracked(HashMap::is_empty));
                db.relation_structure_requests.with_untracked(|requests| {
                    assert!(requests.pending.is_empty());
                    assert!(requests.failures.is_empty());
                });
                assert!(db
                    .begin_relation_structure_request(&failed_key, 0.0)
                    .is_some());
                let current = db.begin_relation_structure_request(&key, 0.0).unwrap();
                db.finish_relation_structure_request(
                    key.clone(),
                    old,
                    Ok(structure("public", "orders")),
                    0.0,
                );
                db.finish_relation_structure_request(
                    key.clone(),
                    old,
                    Err("stale error".into()),
                    0.0,
                );
                assert_eq!(
                    db.relation_structure_status("public", "orders"),
                    RelationStructureStatus::Loading
                );
                db.finish_relation_structure_request(
                    key,
                    current,
                    Ok(structure("public", "orders")),
                    0.0,
                );
                assert_eq!(
                    db.relation_structure_status("public", "orders"),
                    RelationStructureStatus::Ready(structure("public", "orders"))
                );
            })
        });
    }

    #[test]
    fn disposed_store_ignores_requests_and_late_completions() {
        let owner = Owner::new();
        let db = owner.with(DbStore::init);
        let key = RelationStructureKey::new("public", "orders");
        let request = db.begin_relation_structure_request(&key, 0.0).unwrap();
        owner.cleanup();

        assert!(!db.is_current_metadata_generation(request.0));
        assert!(db.begin_relation_structure_request(&key, 0.0).is_none());
        assert!(db.reset_relation_structures().is_none());
        db.finish_relation_structure_request(
            key.clone(),
            request,
            Ok(structure("public", "orders")),
            0.0,
        );
        db.finish_relation_structure_request(key, request, Err("late error".into()), 0.0);
    }
}
