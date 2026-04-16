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

impl Default for DbState {
    fn default() -> Self {
        Self::new()
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
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_primary: bool,
    pub index_type: String,
    pub size_bytes: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoleInfo {
    pub role_name: String,
    pub is_superuser: bool,
    pub can_login: bool,
    pub can_create_db: bool,
    pub can_create_role: bool,
    pub connection_limit: i32,
    pub valid_until: Option<String>,
    pub member_of: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TablePrivilege {
    pub grantee: String,
    pub table_schema: String,
    pub table_name: String,
    pub privileges: Vec<String>,
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
    pub total_rows: i64,
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
            let column_query = "SELECT column_name FROM information_schema.columns
                 WHERE table_schema = $1 AND table_name = $2
                 ORDER BY ordinal_position";

            let columns: Vec<String> =
                match client.query(column_query, &[&schema, &table_name]).await {
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
                total_rows,
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
            let column_query = "SELECT column_name FROM information_schema.columns
                 WHERE table_schema = $1 AND table_name = $2
                 ORDER BY ordinal_position";

            let columns: Vec<String> =
                match client.query(column_query, &[&schema, &table_name]).await {
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
            // Query that consolidates columns into an array and includes type + size
            let query = "
                SELECT
                    i.relname AS index_name,
                    n.nspname AS schema_name,
                    t.relname AS table_name,
                    ARRAY_AGG(a.attname ORDER BY array_position(ix.indkey, a.attnum)) AS columns,
                    ix.indisunique AS is_unique,
                    ix.indisprimary AS is_primary,
                    am.amname AS index_type,
                    pg_relation_size(i.oid) AS size_bytes
                FROM pg_index ix
                JOIN pg_class i ON i.oid = ix.indexrelid
                JOIN pg_class t ON t.oid = ix.indrelid
                JOIN pg_namespace n ON n.oid = t.relnamespace
                JOIN pg_am am ON i.relam = am.oid
                JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
                WHERE n.nspname = $1 AND t.relname = $2
                GROUP BY i.relname, n.nspname, t.relname, ix.indisunique, ix.indisprimary, am.amname, i.oid
                ORDER BY i.relname
            ";

            match client.query(query, &[&schema, &table_name]).await {
                Ok(rows) => {
                    let indexes = rows
                        .iter()
                        .map(|row| IndexInfo {
                            index_name: row.get(0),
                            table_schema: row.get(1),
                            table_name: row.get(2),
                            columns: row.get(3),
                            is_unique: row.get(4),
                            is_primary: row.get(5),
                            index_type: row.get(6),
                            size_bytes: row.get(7),
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
pub async fn get_roles(state: State<'_, DbState>) -> Result<Vec<RoleInfo>, String> {
    let client_lock = state.client.lock().await;

    match client_lock.as_ref() {
        Some(client) => {
            let query = "
                SELECT
                    r.rolname AS role_name,
                    r.rolsuper AS is_superuser,
                    r.rolcanlogin AS can_login,
                    r.rolcreatedb AS can_create_db,
                    r.rolcreaterole AS can_create_role,
                    r.rolconnlimit AS connection_limit,
                    r.rolvaliduntil::text AS valid_until,
                    COALESCE(
                        ARRAY_AGG(g.rolname ORDER BY g.rolname) FILTER (WHERE g.rolname IS NOT NULL),
                        ARRAY[]::text[]
                    ) AS member_of
                FROM pg_roles r
                LEFT JOIN pg_auth_members m ON r.oid = m.member
                LEFT JOIN pg_roles g ON m.roleid = g.oid
                WHERE r.rolname NOT LIKE 'pg_%'
                GROUP BY r.rolname, r.rolsuper, r.rolcanlogin, r.rolcreatedb, r.rolcreaterole, r.rolconnlimit, r.rolvaliduntil
                ORDER BY r.rolname
            ";

            match client.query(query, &[]).await {
                Ok(rows) => {
                    let roles = rows
                        .iter()
                        .map(|row| RoleInfo {
                            role_name: row.get(0),
                            is_superuser: row.get(1),
                            can_login: row.get(2),
                            can_create_db: row.get(3),
                            can_create_role: row.get(4),
                            connection_limit: row.get(5),
                            valid_until: row.get(6),
                            member_of: row.get(7),
                        })
                        .collect();
                    Ok(roles)
                }
                Err(e) => Err(format!("Failed to fetch roles: {}", e)),
            }
        }
        None => Err("Not connected to database".to_string()),
    }
}

#[tauri::command]
pub async fn get_table_privileges(
    state: State<'_, DbState>,
) -> Result<Vec<TablePrivilege>, String> {
    let client_lock = state.client.lock().await;

    match client_lock.as_ref() {
        Some(client) => {
            let query = "
                SELECT
                    grantee::text,
                    table_schema::text,
                    table_name::text,
                    ARRAY_AGG(privilege_type::text ORDER BY privilege_type) AS privileges
                FROM information_schema.table_privileges
                WHERE grantee NOT LIKE 'pg_%'
                  AND table_schema NOT IN ('pg_catalog', 'information_schema')
                GROUP BY grantee, table_schema, table_name
                ORDER BY grantee, table_schema, table_name
            ";

            match client.query(query, &[]).await {
                Ok(rows) => {
                    let privileges = rows
                        .iter()
                        .map(|row| TablePrivilege {
                            grantee: row.get(0),
                            table_schema: row.get(1),
                            table_name: row.get(2),
                            privileges: row.get(3),
                        })
                        .collect();
                    Ok(privileges)
                }
                Err(e) => Err(format!("Failed to fetch table privileges: {}", e)),
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

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_config() -> ConnectionConfig {
        ConnectionConfig {
            host: "localhost".to_string(),
            port: "5432".to_string(),
            database: "videre_test".to_string(),
            username: "videre".to_string(),
            password: "videre".to_string(),
        }
    }

    #[test]
    fn db_state_new_has_no_client() {
        let state = DbState::new();
        let client = state.client.try_lock().unwrap();
        assert!(client.is_none());
    }

    #[test]
    fn db_state_new_has_no_config() {
        let state = DbState::new();
        let config = state.config.try_lock().unwrap();
        assert!(config.is_none());
    }

    #[test]
    fn connection_config_serializes_to_json() {
        let config = sample_config();
        let json = serde_json::to_value(&config).unwrap();
        assert_eq!(json["host"], "localhost");
        assert_eq!(json["port"], "5432");
        assert_eq!(json["database"], "videre_test");
        assert_eq!(json["username"], "videre");
        assert_eq!(json["password"], "videre");
    }

    #[test]
    fn connection_config_deserializes_from_json() {
        let json = serde_json::json!({
            "host": "127.0.0.1",
            "port": "5433",
            "database": "mydb",
            "username": "user",
            "password": "pass"
        });
        let config: ConnectionConfig = serde_json::from_value(json).unwrap();
        assert_eq!(config.host, "127.0.0.1");
        assert_eq!(config.port, "5433");
        assert_eq!(config.database, "mydb");
    }

    #[test]
    fn connection_config_rejects_missing_fields() {
        let json = serde_json::json!({
            "host": "localhost",
            "port": "5432"
        });
        let result: Result<ConnectionConfig, _> = serde_json::from_value(json);
        assert!(result.is_err());
    }

    #[test]
    fn connection_result_serializes_success() {
        let result = ConnectionResult {
            success: true,
            message: "Connected".to_string(),
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["success"], true);
        assert_eq!(json["message"], "Connected");
    }

    #[test]
    fn connection_result_serializes_failure() {
        let result = ConnectionResult {
            success: false,
            message: "Connection refused".to_string(),
        };
        let json = serde_json::to_value(&result).unwrap();
        assert_eq!(json["success"], false);
        assert_eq!(json["message"], "Connection refused");
    }

    #[test]
    fn table_info_serializes() {
        let info = TableInfo {
            name: "users".to_string(),
            schema: "public".to_string(),
        };
        let json = serde_json::to_value(&info).unwrap();
        assert_eq!(json["name"], "users");
        assert_eq!(json["schema"], "public");
    }

    #[test]
    fn table_data_serializes_with_rows() {
        let data = TableData {
            columns: vec!["id".to_string(), "name".to_string()],
            rows: vec![vec![
                serde_json::Value::Number(1.into()),
                serde_json::Value::String("Alice".to_string()),
            ]],
            total_rows: 1,
        };
        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["columns"], serde_json::json!(["id", "name"]));
        assert_eq!(json["total_rows"], 1);
        assert_eq!(json["rows"][0][1], "Alice");
    }

    #[test]
    fn table_data_serializes_empty() {
        let data = TableData {
            columns: vec![],
            rows: vec![],
            total_rows: 0,
        };
        let json = serde_json::to_value(&data).unwrap();
        assert_eq!(json["total_rows"], 0);
        assert!(json["rows"].as_array().unwrap().is_empty());
    }

    #[test]
    fn foreign_key_info_serializes() {
        let fk = ForeignKeyInfo {
            column_name: "user_id".to_string(),
            foreign_table_schema: "public".to_string(),
            foreign_table_name: "users".to_string(),
            foreign_column_name: "id".to_string(),
        };
        let json = serde_json::to_value(&fk).unwrap();
        assert_eq!(json["column_name"], "user_id");
        assert_eq!(json["foreign_table_name"], "users");
        assert_eq!(json["foreign_column_name"], "id");
    }

    #[test]
    fn index_info_serializes() {
        let idx = IndexInfo {
            index_name: "idx_users_email".to_string(),
            table_schema: "public".to_string(),
            table_name: "users".to_string(),
            columns: vec!["email".to_string()],
            is_unique: true,
            is_primary: false,
            index_type: "btree".to_string(),
            size_bytes: 8192,
        };
        let json = serde_json::to_value(&idx).unwrap();
        assert_eq!(json["index_name"], "idx_users_email");
        assert_eq!(json["is_unique"], true);
        assert_eq!(json["is_primary"], false);
        assert_eq!(json["index_type"], "btree");
        assert_eq!(json["size_bytes"], 8192);
        assert_eq!(json["columns"], serde_json::json!(["email"]));
    }

    #[test]
    fn role_info_serializes() {
        let role = RoleInfo {
            role_name: "admin".to_string(),
            is_superuser: true,
            can_login: true,
            can_create_db: true,
            can_create_role: false,
            connection_limit: -1,
            valid_until: None,
            member_of: vec!["pg_read_all_data".to_string()],
        };
        let json = serde_json::to_value(&role).unwrap();
        assert_eq!(json["role_name"], "admin");
        assert_eq!(json["is_superuser"], true);
        assert_eq!(json["can_login"], true);
        assert!(json["valid_until"].is_null());
        assert_eq!(json["member_of"], serde_json::json!(["pg_read_all_data"]));
    }

    #[test]
    fn role_info_serializes_with_expiry() {
        let role = RoleInfo {
            role_name: "temp_user".to_string(),
            is_superuser: false,
            can_login: true,
            can_create_db: false,
            can_create_role: false,
            connection_limit: 5,
            valid_until: Some("2025-12-31".to_string()),
            member_of: vec![],
        };
        let json = serde_json::to_value(&role).unwrap();
        assert_eq!(json["valid_until"], "2025-12-31");
        assert_eq!(json["connection_limit"], 5);
    }

    #[test]
    fn table_privilege_serializes() {
        let priv_info = TablePrivilege {
            grantee: "app_user".to_string(),
            table_schema: "public".to_string(),
            table_name: "users".to_string(),
            privileges: vec!["SELECT".to_string(), "INSERT".to_string()],
        };
        let json = serde_json::to_value(&priv_info).unwrap();
        assert_eq!(json["grantee"], "app_user");
        assert_eq!(json["privileges"], serde_json::json!(["SELECT", "INSERT"]));
    }

    #[test]
    fn row_data_serializes() {
        let row = RowData {
            columns: vec!["id".to_string(), "active".to_string()],
            values: vec![
                serde_json::Value::Number(42.into()),
                serde_json::Value::Bool(true),
            ],
        };
        let json = serde_json::to_value(&row).unwrap();
        assert_eq!(json["columns"], serde_json::json!(["id", "active"]));
        assert_eq!(json["values"][0], 42);
        assert_eq!(json["values"][1], true);
    }

    #[test]
    fn row_data_handles_null_values() {
        let row = RowData {
            columns: vec!["id".to_string(), "name".to_string()],
            values: vec![
                serde_json::Value::Number(1.into()),
                serde_json::Value::Null,
            ],
        };
        let json = serde_json::to_value(&row).unwrap();
        assert!(json["values"][1].is_null());
    }

    #[tokio::test]
    async fn db_state_config_can_be_set_and_cleared() {
        let state = DbState::new();

        {
            let mut config = state.config.lock().await;
            *config = Some(sample_config());
        }

        {
            let config = state.config.lock().await;
            assert!(config.is_some());
            assert_eq!(config.as_ref().unwrap().host, "localhost");
        }

        {
            let mut config = state.config.lock().await;
            *config = None;
        }

        {
            let config = state.config.lock().await;
            assert!(config.is_none());
        }
    }

    #[tokio::test]
    async fn db_state_client_starts_none_and_can_be_cleared() {
        let state = DbState::new();

        let client = state.client.lock().await;
        assert!(client.is_none());
        drop(client);

        // Clearing an already-None client should work fine
        let mut client = state.client.lock().await;
        *client = None;
        assert!(client.is_none());
    }

    #[test]
    fn connection_config_clone() {
        let config = sample_config();
        let cloned = config.clone();
        assert_eq!(config.host, cloned.host);
        assert_eq!(config.port, cloned.port);
        assert_eq!(config.database, cloned.database);
        assert_eq!(config.username, cloned.username);
        assert_eq!(config.password, cloned.password);
    }

    #[test]
    fn connection_config_debug_format() {
        let config = sample_config();
        let debug = format!("{:?}", config);
        assert!(debug.contains("ConnectionConfig"));
        assert!(debug.contains("localhost"));
    }

    #[test]
    fn foreign_key_info_clone() {
        let fk = ForeignKeyInfo {
            column_name: "user_id".to_string(),
            foreign_table_schema: "public".to_string(),
            foreign_table_name: "users".to_string(),
            foreign_column_name: "id".to_string(),
        };
        let cloned = fk.clone();
        assert_eq!(fk.column_name, cloned.column_name);
        assert_eq!(fk.foreign_table_name, cloned.foreign_table_name);
    }

    #[test]
    fn index_info_clone() {
        let idx = IndexInfo {
            index_name: "pk".to_string(),
            table_schema: "public".to_string(),
            table_name: "t".to_string(),
            columns: vec!["id".to_string()],
            is_unique: true,
            is_primary: true,
            index_type: "btree".to_string(),
            size_bytes: 0,
        };
        let cloned = idx.clone();
        assert_eq!(idx.index_name, cloned.index_name);
        assert_eq!(idx.columns, cloned.columns);
    }
}
