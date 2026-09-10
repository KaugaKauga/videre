//! The IPC surface. Argument unwrapping, state lookup, and nothing else — no
//! SQL, no driver types. Each command delegates straight to the adapter.

use tauri::State;

use crate::pg;
use crate::state::DbState;
use crate::types::{
    ConnectionConfig, ConnectionResult, ForeignKeyInfo, IndexInfo, RoleInfo, RowData,
    SortDirection, TableData, TableInfo, TablePrivilege,
};

const DEFAULT_PAGE_SIZE: i64 = 100;

fn not_connected() -> String {
    "Not connected to database".to_string()
}

/// Connection failures come back in-band as `success: false` rather than as
/// `Err`, because the connection form renders the message inline.
fn outcome(result: Result<(), String>, success_message: &str) -> ConnectionResult {
    match result {
        Ok(()) => ConnectionResult {
            success: true,
            message: success_message.to_string(),
        },
        Err(message) => ConnectionResult {
            success: false,
            message,
        },
    }
}

#[tauri::command]
pub async fn test_connection(config: ConnectionConfig) -> Result<ConnectionResult, String> {
    let result = pg::Connection::probe(&config).await;
    Ok(outcome(result, "Connection successful"))
}

#[tauri::command]
pub async fn connect_to_db(
    config: ConnectionConfig,
    state: State<'_, DbState>,
) -> Result<ConnectionResult, String> {
    let connection = match pg::Connection::connect(&config).await {
        Ok(connection) => connection,
        Err(message) => return Ok(outcome(Err(message), "")),
    };

    *state.connection.lock().await = Some(connection);
    *state.config.lock().await = Some(config);

    Ok(outcome(Ok(()), "Connected successfully"))
}

#[tauri::command]
pub async fn disconnect_db(state: State<'_, DbState>) -> Result<(), String> {
    *state.connection.lock().await = None;
    *state.config.lock().await = None;
    Ok(())
}

#[tauri::command]
pub async fn get_tables(state: State<'_, DbState>) -> Result<Vec<TableInfo>, String> {
    let guard = state.connection.lock().await;
    guard.as_ref().ok_or_else(not_connected)?.tables().await
}

#[tauri::command]
pub async fn get_table_data(
    table_name: String,
    schema: String,
    limit: Option<i64>,
    offset: Option<i64>,
    sort_column: Option<String>,
    sort_direction: Option<SortDirection>,
    state: State<'_, DbState>,
) -> Result<TableData, String> {
    let guard = state.connection.lock().await;
    guard
        .as_ref()
        .ok_or_else(not_connected)?
        .table_data(
            &schema,
            &table_name,
            limit.unwrap_or(DEFAULT_PAGE_SIZE),
            offset.unwrap_or(0),
            sort_column.as_deref(),
            sort_direction.unwrap_or_default(),
        )
        .await
}

#[tauri::command]
pub async fn get_foreign_keys(
    table_name: String,
    schema: String,
    state: State<'_, DbState>,
) -> Result<Vec<ForeignKeyInfo>, String> {
    let guard = state.connection.lock().await;
    guard
        .as_ref()
        .ok_or_else(not_connected)?
        .foreign_keys(&schema, &table_name)
        .await
}

#[tauri::command]
pub async fn get_indexes(
    table_name: String,
    schema: String,
    state: State<'_, DbState>,
) -> Result<Vec<IndexInfo>, String> {
    let guard = state.connection.lock().await;
    guard
        .as_ref()
        .ok_or_else(not_connected)?
        .indexes(&schema, &table_name)
        .await
}

#[tauri::command]
pub async fn get_row_by_pk(
    table_name: String,
    schema: String,
    pk_column: String,
    pk_value: serde_json::Value,
    state: State<'_, DbState>,
) -> Result<RowData, String> {
    let guard = state.connection.lock().await;
    guard
        .as_ref()
        .ok_or_else(not_connected)?
        .row_by_pk(&schema, &table_name, &pk_column, &pk_value)
        .await
}

#[tauri::command]
pub async fn get_roles(state: State<'_, DbState>) -> Result<Vec<RoleInfo>, String> {
    let guard = state.connection.lock().await;
    guard.as_ref().ok_or_else(not_connected)?.roles().await
}

#[tauri::command]
pub async fn get_table_privileges(
    state: State<'_, DbState>,
) -> Result<Vec<TablePrivilege>, String> {
    let guard = state.connection.lock().await;
    guard
        .as_ref()
        .ok_or_else(not_connected)?
        .table_privileges()
        .await
}
