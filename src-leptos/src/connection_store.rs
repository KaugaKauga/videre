use leptos::prelude::*;
use leptos::task::spawn_local;
use serde::{Deserialize, Serialize};

use crate::tauri;

// --- Saved connection (matches React's camelCase JSON keys) ---

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SavedConnection {
    pub id: String,
    pub host: String,
    pub port: String,
    pub database: String,
    pub username: String,
    pub saved_at: f64,
}

// --- Tauri plugin-store helpers ---

/// Resource ID returned by `plugin:store|create_store`.
/// Cached per-session so we only create once.
use std::cell::Cell;

thread_local! {
    static STORE_RID: Cell<Option<u32>> = const { Cell::new(None) };
}

async fn get_rid() -> Result<u32, String> {
    if let Some(rid) = STORE_RID.with(|c| c.get()) {
        web_sys::console::log_1(&format!("[store] reusing cached rid={rid}").into());
        return Ok(rid);
    }
    web_sys::console::log_1(&"[store] calling plugin:store|load ...".into());
    let rid: u32 = tauri::invoke(
        "plugin:store|load",
        &serde_json::json!({ "path": "connections.json" }),
    )
    .await
    .map_err(|e| {
        web_sys::console::error_1(&format!("[store] load failed: {e}").into());
        e
    })?;
    web_sys::console::log_1(&format!("[store] got rid={rid}").into());
    STORE_RID.with(|c| c.set(Some(rid)));
    Ok(rid)
}

async fn store_get_connections() -> Result<Vec<SavedConnection>, String> {
    let rid = get_rid().await?;
    web_sys::console::log_1(
        &format!("[store] calling plugin:store|get rid={rid} key=connections").into(),
    );
    let result: (serde_json::Value, bool) = tauri::invoke(
        "plugin:store|get",
        &serde_json::json!({ "rid": rid, "key": "connections" }),
    )
    .await
    .map_err(|e| {
        web_sys::console::error_1(&format!("[store] get failed: {e}").into());
        e
    })?;
    web_sys::console::log_1(
        &format!(
            "[store] get returned exists={}, value={}",
            result.1, result.0
        )
        .into(),
    );
    if result.1 {
        let conns: Vec<SavedConnection> = serde_json::from_value(result.0).map_err(|e| {
            web_sys::console::error_1(
                &format!("[store] deserialize connections failed: {e}").into(),
            );
            e.to_string()
        })?;
        web_sys::console::log_1(&format!("[store] loaded {} connections", conns.len()).into());
        Ok(conns)
    } else {
        web_sys::console::log_1(&"[store] no connections key found, returning empty".into());
        Ok(Vec::new())
    }
}

async fn store_set_connections(connections: &[SavedConnection]) -> Result<(), String> {
    let rid = get_rid().await?;
    let value = serde_json::to_value(connections).map_err(|e| e.to_string())?;
    web_sys::console::log_1(
        &format!(
            "[store] calling plugin:store|set rid={rid} with {} connections",
            connections.len()
        )
        .into(),
    );
    tauri::invoke_void(
        "plugin:store|set",
        &serde_json::json!({ "rid": rid, "key": "connections", "value": value }),
    )
    .await
    .map_err(|e| {
        web_sys::console::error_1(&format!("[store] set failed: {e}").into());
        e
    })?;
    web_sys::console::log_1(&"[store] calling plugin:store|save ...".into());
    tauri::invoke_void("plugin:store|save", &serde_json::json!({ "rid": rid }))
        .await
        .map_err(|e| {
            web_sys::console::error_1(&format!("[store] save failed: {e}").into());
            e
        })?;
    web_sys::console::log_1(&"[store] save OK".into());
    Ok(())
}

// --- ID generation ---

fn generate_id() -> String {
    let now = js_sys::Date::now() as u64;
    let rand = (js_sys::Math::random() * 999_999_999.0) as u64;
    format!("{:x}-{:x}", now, rand)
}

// --- ConnectionStore (Leptos signals, provided via context) ---

#[derive(Clone, Copy)]
pub struct ConnectionStore {
    pub connections: RwSignal<Vec<SavedConnection>>,
    pub is_loaded: RwSignal<bool>,
}

impl ConnectionStore {
    /// Create the store and kick off async load from disk.
    pub fn init() -> Self {
        let connections = RwSignal::new(Vec::<SavedConnection>::new());
        let is_loaded = RwSignal::new(false);

        web_sys::console::log_1(&"[store] init() starting async load ...".into());
        spawn_local(async move {
            match store_get_connections().await {
                Ok(conns) => {
                    web_sys::console::log_1(
                        &format!("[store] init loaded {} connections", conns.len()).into(),
                    );
                    connections.set(conns);
                }
                Err(e) => web_sys::console::error_1(
                    &format!("[store] init failed to load connections: {e}").into(),
                ),
            }
            is_loaded.set(true);
        });

        Self {
            connections,
            is_loaded,
        }
    }

    /// Save (or bump) a connection and persist to disk.
    pub fn save_connection(self, host: String, port: String, database: String, username: String) {
        web_sys::console::log_1(
            &format!("[store] save_connection called: {database}@{host}:{port}").into(),
        );
        let connections = self.connections;
        spawn_local(async move {
            let mut conns = connections.get_untracked();

            let existing_idx = conns.iter().position(|c| {
                c.host == host && c.port == port && c.database == database && c.username == username
            });

            if let Some(idx) = existing_idx {
                let mut existing = conns.remove(idx);
                existing.saved_at = js_sys::Date::now();
                conns.insert(0, existing);
            } else {
                conns.insert(
                    0,
                    SavedConnection {
                        id: generate_id(),
                        host,
                        port,
                        database,
                        username,
                        saved_at: js_sys::Date::now(),
                    },
                );
            }

            conns.truncate(10);

            if let Err(e) = store_set_connections(&conns).await {
                web_sys::console::error_1(&format!("Failed to save connections: {e}").into());
                return;
            }

            connections.set(conns);
        });
    }

    /// Remove a connection by ID and persist to disk.
    pub fn remove_connection(self, id: String) {
        let connections = self.connections;
        spawn_local(async move {
            let mut conns = connections.get_untracked();
            conns.retain(|c| c.id != id);

            if let Err(e) = store_set_connections(&conns).await {
                web_sys::console::error_1(&format!("Failed to save connections: {e}").into());
                return;
            }

            connections.set(conns);
        });
    }
}
