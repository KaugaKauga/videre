use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tauri::State;
use tokio::sync::Mutex;
use tokio_postgres::{Client, NoTls};
use uuid::Uuid;

use crate::pg::{qualified_name, quote_ident, ColumnPlan};

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

/// Ask the server for the shape of `SELECT *` before reading any rows.
///
/// `prepare` only parses and plans, so this is cheaper than the
/// `information_schema.columns` lookup it replaces — and it gives us the column
/// types as well as the names, from the exact statement we are about to run.
async fn column_plan(client: &Client, relation: &str) -> Result<ColumnPlan, String> {
    let stmt = client
        .prepare(&format!("SELECT * FROM {relation}"))
        .await
        .map_err(|e| format!("Failed to inspect columns: {e}"))?;
    Ok(ColumnPlan::from_columns(stmt.columns()))
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
    let client = client_lock
        .as_ref()
        .ok_or_else(|| "Not connected to database".to_string())?;

    let limit = limit.unwrap_or(100);
    let offset = offset.unwrap_or(0);

    let relation = qualified_name(&schema, &table_name)?;
    let plan = column_plan(client, &relation).await?;

    // `CREATE TABLE t ()` is legal; an empty SELECT list is not.
    if plan.is_empty() {
        return Ok(TableData {
            columns: Vec::new(),
            rows: Vec::new(),
            total_rows: 0,
        });
    }

    let data_query = format!(
        "SELECT {} FROM {relation} LIMIT $1 OFFSET $2",
        plan.select_list()?
    );
    let rows = client
        .query(&data_query, &[&limit, &offset])
        .await
        .map_err(|e| format!("Failed to fetch data: {e}"))?;

    let count_query = format!("SELECT COUNT(*) FROM {relation}");
    let total_rows: i64 = client
        .query_one(&count_query, &[])
        .await
        .map_err(|e| format!("Failed to count rows: {e}"))?
        .get(0);

    Ok(TableData {
        columns: plan.names(),
        rows: rows.iter().map(|row| plan.decode_row(row)).collect(),
        total_rows,
    })
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
    let client = client_lock
        .as_ref()
        .ok_or_else(|| "Not connected to database".to_string())?;

    let relation = qualified_name(&schema, &table_name)?;
    let plan = column_plan(client, &relation).await?;

    if plan.is_empty() {
        return Err(format!("{relation} has no columns"));
    }

    let data_query = format!(
        "SELECT {} FROM {relation} WHERE {} = $1 LIMIT 1",
        plan.select_list()?,
        quote_ident(&pk_column)?
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
        Err(e) => return Err(format!("Failed to fetch row: {e}")),
    };

    Ok(RowData {
        columns: plan.names(),
        values: plan.decode_row(&row),
    })
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

/// Tests that need a live server.
///
/// Ignored by default so `cargo test` stays offline. Bring the fixture up with
/// `docker-compose up -d`, then:
///
/// ```text
/// cargo test --lib live -- --ignored
/// ```
#[cfg(test)]
mod live {
    use super::*;
    use serde_json::Value;

    async fn connect() -> Client {
        let conn_str = std::env::var("VIDERE_TEST_DB").unwrap_or_else(|_| {
            "host=localhost port=5432 dbname=videre_test user=videre password=videre".to_string()
        });
        let (client, connection) = tokio_postgres::connect(&conn_str, NoTls)
            .await
            .expect("test database unreachable — is `docker-compose up -d` running?");
        tokio::spawn(async move {
            let _ = connection.await;
        });
        client
    }

    /// The reported bug: `created_at` is `timestamptz` in every seeded table, and
    /// the old try_get chain had no arm for it, so it rendered as `null`.
    #[tokio::test]
    #[ignore]
    async fn seeded_timestamps_are_no_longer_null() {
        let client = connect().await;
        let relation = qualified_name("public", "gods").unwrap();
        let plan = column_plan(&client, &relation).await.unwrap();

        let created_at = plan
            .names()
            .iter()
            .position(|n| n == "created_at")
            .expect("gods.created_at should exist");

        let query = format!(
            "SELECT {} FROM {relation} LIMIT 5",
            plan.select_list().unwrap()
        );
        let rows = client.query(&query, &[]).await.unwrap();
        assert!(!rows.is_empty(), "gods should have seed data");

        for row in &rows {
            let cell = &plan.decode_row(row)[created_at];
            assert!(
                matches!(cell, Value::String(s) if !s.is_empty()),
                "created_at should render as a timestamp, got {cell:?}"
            );
        }
    }

    /// Every type that used to fall through to `Value::Null` must now come back
    /// with a value — and a genuine NULL must still come back as `null`.
    #[tokio::test]
    #[ignore]
    async fn no_type_silently_renders_as_null() {
        let client = connect().await;
        client
            .batch_execute(
                "DROP SCHEMA IF EXISTS videre_convert_test CASCADE;
                 CREATE SCHEMA videre_convert_test;
                 CREATE TYPE videre_convert_test.mood AS ENUM ('calm', 'wrathful');
                 CREATE TABLE videre_convert_test.every_type (
                     c_int2        smallint,
                     c_int4        integer,
                     c_int8        bigint,
                     c_float4      real,
                     c_float8      double precision,
                     c_numeric     numeric(38,10),
                     c_bool        boolean,
                     c_text        text,
                     c_varchar     varchar(20),
                     c_uuid        uuid,
                     c_timestamptz timestamp with time zone,
                     c_timestamp   timestamp,
                     c_date        date,
                     c_time        time,
                     c_interval    interval,
                     c_json        json,
                     c_jsonb       jsonb,
                     c_bytea       bytea,
                     c_inet        inet,
                     c_text_arr    text[],
                     c_enum        videre_convert_test.mood,
                     c_always_null integer
                 );
                 INSERT INTO videre_convert_test.every_type VALUES (
                     32767, 2147483647, 9223372036854775807,
                     0.1, 2.5, 12345678901234567890.1234567890,
                     true, 'text', 'varchar',
                     '0b7f2c1e-4a5d-4f8e-9c3a-1d2e3f4a5b6c',
                     '2024-01-15 10:30:00+00', '2024-01-15 10:30:00',
                     '2024-01-15', '10:30:00', '3 days 4 hours',
                     '{\"a\":1}', '{\"b\":2}', '\\x48656c6c6f',
                     '192.168.0.1', ARRAY['alpha','beta'], 'wrathful',
                     NULL
                 );",
            )
            .await
            .unwrap();

        let relation = qualified_name("videre_convert_test", "every_type").unwrap();
        let plan = column_plan(&client, &relation).await.unwrap();
        let query = format!("SELECT {} FROM {relation}", plan.select_list().unwrap());
        let row = client.query_one(&query, &[]).await.unwrap();

        let names = plan.names();
        let values = plan.decode_row(&row);

        for (name, value) in names.iter().zip(&values) {
            if name == "c_always_null" {
                // A real SQL NULL must still be null — that distinction is the
                // whole point of the fix.
                assert_eq!(*value, Value::Null, "{name} is genuinely NULL");
            } else {
                assert_ne!(*value, Value::Null, "{name} silently rendered as null");
            }
            if let Value::String(s) = value {
                assert!(!s.starts_with("<unreadable"), "{name}: {s}");
            }
        }

        let cell = |name: &str| values[names.iter().position(|n| n == name).unwrap()].clone();

        // Natively decoded types keep their JSON type so the UI can align them.
        assert_eq!(cell("c_int2"), Value::from(32767));
        assert_eq!(cell("c_int8"), Value::from(9223372036854775807i64));
        assert_eq!(cell("c_float4"), Value::from(0.1));
        assert_eq!(cell("c_bool"), Value::Bool(true));

        // `numeric` goes through text precisely so this precision survives.
        assert_eq!(
            cell("c_numeric"),
            Value::String("12345678901234567890.1234567890".into())
        );

        // Server-rendered types read the way psql prints them.
        assert_eq!(cell("c_date"), Value::String("2024-01-15".into()));
        assert_eq!(cell("c_text_arr"), Value::String("{alpha,beta}".into()));
        assert_eq!(cell("c_enum"), Value::String("wrathful".into()));
        assert_eq!(cell("c_bytea"), Value::String("\\x48656c6c6f".into()));

        client
            .batch_execute("DROP SCHEMA videre_convert_test CASCADE;")
            .await
            .unwrap();
    }
}
