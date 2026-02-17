use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

pub struct DbState {
    pub client: Arc<Mutex<Option<Client>>>,
    pub config: Arc<Mutex<Option<ConnectionConfig>>>,
}

impl DbState {
    pub fn new() -> Self {
        Self {
            client: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(None)),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ConnectionResult {
    pub success: bool,
    pub message: String,
}

#[tauri::command]
pub async fn test_connection(config: ConnectionConfig) -> Result<ConnectionResult, String> {
    let connection_string = format!(
        "host={} port={} dbname={} user={} password={}",
        config.host, config.port, config.database, config.username, config.password
    );

    match tokio_postgres::connect(&connection_string, NoTls).await {
        Ok((client, connection)) => {
            // Spawn connection in background
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("connection error: {}", e);
                }
            });

            // Test with a simple query
            match client.query("SELECT 1", &[]).await {
                Ok(_) => Ok(ConnectionResult {
                    success: true,
                    message: "Connection successful".to_string(),
                }),
                Err(e) => Ok(ConnectionResult {
                    success: false,
                    message: format!("Query failed: {}", e),
                }),
            }
        }
        Err(e) => Ok(ConnectionResult {
            success: false,
            message: format!("Connection failed: {}", e),
        }),
    }
}

#[tauri::command]
pub async fn connect_to_db(
    config: ConnectionConfig,
    state: State<'_, DbState>,
) -> Result<ConnectionResult, String> {
    let connection_string = format!(
        "host={} port={} dbname={} user={} password={}",
        config.host, config.port, config.database, config.username, config.password
    );

    match tokio_postgres::connect(&connection_string, NoTls).await {
        Ok((client, connection)) => {
            // Spawn connection in background
            tokio::spawn(async move {
                if let Err(e) = connection.await {
                    eprintln!("connection error: {}", e);
                }
            });

            // Store the client and config
            let mut client_lock = state.client.lock().await;
            *client_lock = Some(client);

            let mut config_lock = state.config.lock().await;
            *config_lock = Some(config);

            Ok(ConnectionResult {
                success: true,
                message: "Connected successfully".to_string(),
            })
        }
        Err(e) => Ok(ConnectionResult {
            success: false,
            message: format!("Connection failed: {}", e),
        }),
    }
}

#[derive(Debug, Serialize)]
pub struct TableInfo {
    pub name: String,
    pub schema: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForeignKeyInfo {
    pub column_name: String,
    pub foreign_table_schema: String,
    pub foreign_table_name: String,
    pub foreign_column_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexInfo {
    pub index_name: String,
    pub table_schema: String,
    pub table_name: String,
    pub column_name: String,
    pub is_unique: bool,
    pub is_primary: bool,
}

#[tauri::command]
pub async fn get_tables(state: State<'_, DbState>) -> Result<Vec<TableInfo>, String> {
    let client_lock = state.client.lock().await;

    match client_lock.as_ref() {
        Some(client) => {
            let query = "
                SELECT table_name, table_schema
                FROM information_schema.tables
                WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
                ORDER BY table_schema, table_name
            ";

            match client.query(query, &[]).await {
                Ok(rows) => {
                    let tables = rows
                        .iter()
                        .map(|row| TableInfo {
                            name: row.get(0),
                            schema: row.get(1),
                        })
                        .collect();
                    Ok(tables)
                }
                Err(e) => Err(format!("Failed to fetch tables: {}", e)),
            }
        }
        None => Err("Not connected to database".to_string()),
    }
}

#[derive(Debug, Serialize)]
pub struct TableData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub total_rows: usize,
}

#[tauri::command]
pub async fn get_table_data(
    table_name: String,
    schema: String,
    limit: Option<i64>,
    offset: Option<i64>,
    state: State<'_, DbState>,
) -> Result<TableData, String> {
    let client_lock = state.client.lock().await;

    match client_lock.as_ref() {
        Some(client) => {
            let limit = limit.unwrap_or(100);
            let offset = offset.unwrap_or(0);

            // Get column information
            let column_query = format!(
                "SELECT column_name FROM information_schema.columns
                 WHERE table_schema = $1 AND table_name = $2
                 ORDER BY ordinal_position"
            );

            let columns: Vec<String> =
                match client.query(&column_query, &[&schema, &table_name]).await {
                    Ok(rows) => rows.iter().map(|row| row.get(0)).collect(),
                    Err(e) => return Err(format!("Failed to fetch columns: {}", e)),
                };

            // Get table data
            let data_query = format!(
                "SELECT * FROM \"{}\".\"{}\" LIMIT $1 OFFSET $2",
                schema, table_name
            );

            let rows_result = match client.query(&data_query, &[&limit, &offset]).await {
                Ok(rows) => rows,
                Err(e) => return Err(format!("Failed to fetch data: {}", e)),
            };

            // Convert rows to JSON
            let mut data_rows = Vec::new();
            for row in rows_result {
                let mut row_data = Vec::new();
                for i in 0..columns.len() {
                    // Try to get value as various types and convert to JSON
                    let value: serde_json::Value = if let Ok(v) = row.try_get::<_, Option<Uuid>>(i)
                    {
                        // Handle UUID (including NULL)
                        match v {
                            Some(uuid) => serde_json::Value::String(uuid.to_string()),
                            None => serde_json::Value::Null,
                        }
                    } else if let Ok(v) = row.try_get::<_, Option<String>>(i) {
                        match v {
                            Some(s) => serde_json::Value::String(s),
                            None => serde_json::Value::Null,
                        }
                    } else if let Ok(v) = row.try_get::<_, Option<i32>>(i) {
                        match v {
                            Some(n) => serde_json::Value::Number(n.into()),
                            None => serde_json::Value::Null,
                        }
                    } else if let Ok(v) = row.try_get::<_, Option<i64>>(i) {
                        match v {
                            Some(n) => serde_json::Value::Number(n.into()),
                            None => serde_json::Value::Null,
                        }
                    } else if let Ok(v) = row.try_get::<_, Option<f64>>(i) {
                        match v {
                            Some(f) => serde_json::json!(f),
                            None => serde_json::Value::Null,
                        }
                    } else if let Ok(v) = row.try_get::<_, Option<bool>>(i) {
                        match v {
                            Some(b) => serde_json::Value::Bool(b),
                            None => serde_json::Value::Null,
                        }
                    } else {
                        // If we can't determine the type, return NULL
                        serde_json::Value::Null
                    };
                    row_data.push(value);
                }
                data_rows.push(row_data);
            }

            // Get total count
            let count_query = format!("SELECT COUNT(*) FROM \"{}\".\"{}\"", schema, table_name);
            let total_rows: i64 = match client.query_one(&count_query, &[]).await {
                Ok(row) => row.get(0),
                Err(e) => return Err(format!("Failed to count rows: {}", e)),
            };

            Ok(TableData {
                columns,
                rows: data_rows,
                total_rows: total_rows as usize,
            })
        }
        None => Err("Not connected to database".to_string()),
    }
}

#[tauri::command]
pub async fn get_foreign_keys(
    table_name: String,
    schema: String,
    state: State<'_, DbState>,
) -> Result<Vec<ForeignKeyInfo>, String> {
    let client_lock = state.client.lock().await;

    match client_lock.as_ref() {
        Some(client) => {
            let query = "
                SELECT
                    kcu.column_name,
                    ccu.table_schema AS foreign_table_schema,
                    ccu.table_name AS foreign_table_name,
                    ccu.column_name AS foreign_column_name
                FROM information_schema.table_constraints AS tc
                JOIN information_schema.key_column_usage AS kcu
                    ON tc.constraint_name = kcu.constraint_name
                    AND tc.table_schema = kcu.table_schema
                JOIN information_schema.constraint_column_usage AS ccu
                    ON ccu.constraint_name = tc.constraint_name
                    AND ccu.table_schema = tc.table_schema
                WHERE tc.constraint_type = 'FOREIGN KEY'
                    AND tc.table_schema = $1
                    AND tc.table_name = $2
            ";

            match client.query(query, &[&schema, &table_name]).await {
                Ok(rows) => {
                    let fks = rows
                        .iter()
                        .map(|row| ForeignKeyInfo {
                            column_name: row.get(0),
                            foreign_table_schema: row.get(1),
                            foreign_table_name: row.get(2),
                            foreign_column_name: row.get(3),
                        })
                        .collect();
                    Ok(fks)
                }
                Err(e) => Err(format!("Failed to fetch foreign keys: {}", e)),
            }
        }
        None => Err("Not connected to database".to_string()),
    }
}

#[derive(Debug, Serialize)]
pub struct RowData {
    pub columns: Vec<String>,
    pub values: Vec<serde_json::Value>,
}

#[tauri::command]
pub async fn get_row_by_pk(
    table_name: String,
    schema: String,
    pk_column: String,
    pk_value: serde_json::Value,
    state: State<'_, DbState>,
) -> Result<RowData, String> {
    let client_lock = state.client.lock().await;

    match client_lock.as_ref() {
        Some(client) => {
            // Get column information
            let column_query = format!(
                "SELECT column_name FROM information_schema.columns
                 WHERE table_schema = $1 AND table_name = $2
                 ORDER BY ordinal_position"
            );

            let columns: Vec<String> =
                match client.query(&column_query, &[&schema, &table_name]).await {
                    Ok(rows) => rows.iter().map(|row| row.get(0)).collect(),
                    Err(e) => return Err(format!("Failed to fetch columns: {}", e)),
                };

            // Build query based on pk_value type
            let data_query = format!(
                "SELECT * FROM \"{}\".\"{}\" WHERE \"{}\" = $1 LIMIT 1",
                schema, table_name, pk_column
            );

            // Execute query with appropriate type
            let row_result = match &pk_value {
                serde_json::Value::Number(n) => {
                    if let Some(i) = n.as_i64() {
                        client.query_opt(&data_query, &[&(i as i32)]).await
                    } else if let Some(f) = n.as_f64() {
                        client.query_opt(&data_query, &[&f]).await
                    } else {
                        return Err("Invalid number type".to_string());
                    }
                }
                serde_json::Value::String(s) => {
                    // Try parsing as UUID first
                    if let Ok(uuid) = s.parse::<Uuid>() {
                        client.query_opt(&data_query, &[&uuid]).await
                    } else {
                        client.query_opt(&data_query, &[&s]).await
                    }
                }
                _ => return Err("Unsupported primary key type".to_string()),
            };

            let row = match row_result {
                Ok(Some(row)) => row,
                Ok(None) => return Err("Row not found".to_string()),
                Err(e) => return Err(format!("Failed to fetch row: {}", e)),
            };

            // Convert row to JSON values
            let mut values = Vec::new();
            for i in 0..columns.len() {
                let value: serde_json::Value = if let Ok(v) = row.try_get::<_, Option<Uuid>>(i) {
                    match v {
                        Some(uuid) => serde_json::Value::String(uuid.to_string()),
                        None => serde_json::Value::Null,
                    }
                } else if let Ok(v) = row.try_get::<_, Option<String>>(i) {
                    match v {
                        Some(s) => serde_json::Value::String(s),
                        None => serde_json::Value::Null,
                    }
                } else if let Ok(v) = row.try_get::<_, Option<i32>>(i) {
                    match v {
                        Some(n) => serde_json::Value::Number(n.into()),
                        None => serde_json::Value::Null,
                    }
                } else if let Ok(v) = row.try_get::<_, Option<i64>>(i) {
                    match v {
                        Some(n) => serde_json::Value::Number(n.into()),
                        None => serde_json::Value::Null,
                    }
                } else if let Ok(v) = row.try_get::<_, Option<f64>>(i) {
                    match v {
                        Some(f) => serde_json::json!(f),
                        None => serde_json::Value::Null,
                    }
                } else if let Ok(v) = row.try_get::<_, Option<bool>>(i) {
                    match v {
                        Some(b) => serde_json::Value::Bool(b),
                        None => serde_json::Value::Null,
                    }
                } else {
                    serde_json::Value::Null
                };
                values.push(value);
            }

            Ok(RowData { columns, values })
        }
        None => Err("Not connected to database".to_string()),
    }
}

#[tauri::command]
pub async fn get_indexes(
    table_name: String,
    schema: String,
    state: State<'_, DbState>,
) -> Result<Vec<IndexInfo>, String> {
    let client_lock = state.client.lock().await;

    match client_lock.as_ref() {
        Some(client) => {
            let query = "
                SELECT
                    i.relname AS index_name,
                    n.nspname AS schema_name,
                    t.relname AS table_name,
                    a.attname AS column_name,
                    ix.indisunique AS is_unique,
                    ix.indisprimary AS is_primary
                FROM pg_index ix
                JOIN pg_class i ON i.oid = ix.indexrelid
                JOIN pg_class t ON t.oid = ix.indrelid
                JOIN pg_namespace n ON n.oid = t.relnamespace
                JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
                WHERE n.nspname = $1 AND t.relname = $2
                ORDER BY i.relname, a.attnum
            ";

            match client.query(query, &[&schema, &table_name]).await {
                Ok(rows) => {
                    let indexes = rows
                        .iter()
                        .map(|row| IndexInfo {
                            index_name: row.get(0),
                            table_schema: row.get(1),
                            table_name: row.get(2),
                            column_name: row.get(3),
                            is_unique: row.get(4),
                            is_primary: row.get(5),
                        })
                        .collect();
                    Ok(indexes)
                }
                Err(e) => Err(format!("Failed to fetch indexes: {}", e)),
            }
        }
        None => Err("Not connected to database".to_string()),
    }
}

#[tauri::command]
pub async fn disconnect_db(state: State<'_, DbState>) -> Result<(), String> {
    let mut client_lock = state.client.lock().await;
    *client_lock = None;

    let mut config_lock = state.config.lock().await;
    *config_lock = None;

    Ok(())
}
