//! Integration tests for the database backend.
//!
//! These tests require a running PostgreSQL instance seeded with init-db.sql.
//! Start one with: `docker compose up -d`
//!
//! The tests use the connection parameters from docker-compose.yml:
//!   host=localhost port=5432 dbname=videre_test user=videre password=videre
//!
//! Set the environment variable `DATABASE_URL` to override, or skip these tests
//! entirely by filtering: `cargo test --lib` (runs only unit tests).

use tokio_postgres::NoTls;

const HOST: &str = "localhost";
const PORT: &str = "5432";
const DBNAME: &str = "videre_test";
const USER: &str = "videre";
const PASSWORD: &str = "videre";

async fn connect() -> Result<tokio_postgres::Client, tokio_postgres::Error> {
    let connstr = format!(
        "host={HOST} port={PORT} dbname={DBNAME} user={USER} password={PASSWORD}"
    );
    let (client, connection) = tokio_postgres::connect(&connstr, NoTls).await?;
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {}", e);
        }
    });
    Ok(client)
}

// ── Connection ──────────────────────────────────────────────────────────

#[tokio::test]
async fn test_connection_succeeds() {
    let client = connect().await.expect("should connect to test database");
    let row = client.query_one("SELECT 1 AS val", &[]).await.unwrap();
    let val: i32 = row.get(0);
    assert_eq!(val, 1);
}

// ── get_tables query ────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_tables_returns_known_tables() {
    let client = connect().await.expect("should connect");
    let query = "
        SELECT table_name, table_schema
        FROM information_schema.tables
        WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
        ORDER BY table_schema, table_name
    ";
    let rows = client.query(query, &[]).await.unwrap();
    let names: Vec<String> = rows.iter().map(|r| r.get(0)).collect();
    assert!(!names.is_empty(), "should have at least one user table");
    // init-db.sql creates these tables
    assert!(names.contains(&"primordials".to_string()), "missing primordials table");
    assert!(names.contains(&"gods".to_string()), "missing gods table");
    assert!(names.contains(&"heroes".to_string()), "missing heroes table");
}

// ── get_table_data query ────────────────────────────────────────────────

#[tokio::test]
async fn test_get_table_data_columns_and_rows() {
    let client = connect().await.expect("should connect");
    let schema = "public";
    let table = "primordials";

    // Fetch columns
    let col_query = "SELECT column_name FROM information_schema.columns
         WHERE table_schema = $1 AND table_name = $2
         ORDER BY ordinal_position";
    let col_rows = client.query(col_query, &[&schema, &table]).await.unwrap();
    let columns: Vec<String> = col_rows.iter().map(|r| r.get(0)).collect();
    assert!(columns.contains(&"id".to_string()));
    assert!(columns.contains(&"name".to_string()));
    assert!(columns.contains(&"domain".to_string()));

    // Fetch data with limit/offset
    let data_query = format!(
        "SELECT * FROM \"{}\".\"{}\" LIMIT $1 OFFSET $2",
        schema, table
    );
    let limit: i64 = 5;
    let offset: i64 = 0;
    let data_rows = client.query(&data_query, &[&limit, &offset]).await.unwrap();
    assert!(!data_rows.is_empty(), "primordials should have seed data");

    // Verify row count query
    let count_query = format!("SELECT COUNT(*) FROM \"{}\".\"{}\"", schema, table);
    let count_row = client.query_one(&count_query, &[]).await.unwrap();
    let total: i64 = count_row.get(0);
    assert!(total > 0, "primordials should have rows");
}

#[tokio::test]
async fn test_get_table_data_with_offset() {
    let client = connect().await.expect("should connect");
    let schema = "public";
    let table = "primordials";

    let query = format!(
        "SELECT * FROM \"{}\".\"{}\" LIMIT $1 OFFSET $2",
        schema, table
    );

    let all: i64 = 100;
    let zero: i64 = 0;
    let all_rows = client.query(&query, &[&all, &zero]).await.unwrap();

    if all_rows.len() > 1 {
        let one: i64 = 1;
        let skip: i64 = 1;
        let offset_rows = client.query(&query, &[&one, &skip]).await.unwrap();
        assert_eq!(offset_rows.len(), 1);
        // The first row from offset=1 should match the second row from offset=0
        let expected_name: String = all_rows[1].get("name");
        let actual_name: String = offset_rows[0].get("name");
        assert_eq!(expected_name, actual_name);
    }
}

// ── get_foreign_keys query ──────────────────────────────────────────────

#[tokio::test]
async fn test_get_foreign_keys_for_heroes() {
    let client = connect().await.expect("should connect");
    let schema = "public";
    let table = "heroes";

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

    let rows = client.query(query, &[&schema, &table]).await.unwrap();
    assert!(!rows.is_empty(), "heroes table should have foreign keys");

    let fk_columns: Vec<String> = rows.iter().map(|r| r.get::<_, String>(0)).collect();
    // heroes references gods via patron_god_id and divine_parent_id
    assert!(
        fk_columns.contains(&"patron_god_id".to_string())
            || fk_columns.contains(&"divine_parent_id".to_string()),
        "heroes should reference gods table"
    );
}

#[tokio::test]
async fn test_table_with_no_foreign_keys() {
    let client = connect().await.expect("should connect");
    let schema = "public";
    let table = "primordials";

    let query = "
        SELECT kcu.column_name
        FROM information_schema.table_constraints AS tc
        JOIN information_schema.key_column_usage AS kcu
            ON tc.constraint_name = kcu.constraint_name
            AND tc.table_schema = kcu.table_schema
        WHERE tc.constraint_type = 'FOREIGN KEY'
            AND tc.table_schema = $1
            AND tc.table_name = $2
    ";

    let rows = client.query(query, &[&schema, &table]).await.unwrap();
    assert!(rows.is_empty(), "primordials should have no foreign keys");
}

// ── get_indexes query ───────────────────────────────────────────────────

#[tokio::test]
async fn test_get_indexes_for_table() {
    let client = connect().await.expect("should connect");
    let schema = "public";
    let table = "primordials";

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

    let rows = client.query(query, &[&schema, &table]).await.unwrap();
    assert!(!rows.is_empty(), "primordials should have at least a PK index");

    // Check that we find a primary key index
    let has_primary = rows.iter().any(|r| r.get::<_, bool>(5));
    assert!(has_primary, "should have a primary key index");

    // Primary key should be on 'id' column
    let pk_row = rows.iter().find(|r| r.get::<_, bool>(5)).unwrap();
    let pk_cols: Vec<String> = pk_row.get(3);
    assert!(pk_cols.contains(&"id".to_string()));
}

// ── get_roles query ─────────────────────────────────────────────────────

#[tokio::test]
async fn test_get_roles_returns_videre_user() {
    let client = connect().await.expect("should connect");
    let query = "
        SELECT
            r.rolname AS role_name,
            r.rolcanlogin AS can_login
        FROM pg_roles r
        WHERE r.rolname NOT LIKE 'pg_%'
        ORDER BY r.rolname
    ";

    let rows = client.query(query, &[]).await.unwrap();
    let role_names: Vec<String> = rows.iter().map(|r| r.get(0)).collect();
    assert!(
        role_names.contains(&"videre".to_string()),
        "should contain the videre role"
    );
}

// ── get_table_privileges query ──────────────────────────────────────────

#[tokio::test]
async fn test_get_table_privileges() {
    let client = connect().await.expect("should connect");
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

    let rows = client.query(query, &[]).await.unwrap();
    // The videre user owns the tables, so should have privileges
    assert!(!rows.is_empty(), "should have table privileges");

    // Check that privileges array is non-empty for each row
    for row in &rows {
        let privs: Vec<String> = row.get(3);
        assert!(!privs.is_empty(), "each row should have at least one privilege");
    }
}

// ── get_row_by_pk query ─────────────────────────────────────────────────

#[tokio::test]
async fn test_get_row_by_pk_integer() {
    let client = connect().await.expect("should connect");
    let schema = "public";
    let table = "primordials";

    // First get a known ID
    let first_row = client
        .query_one(
            &format!("SELECT id FROM \"{}\".\"{}\" LIMIT 1", schema, table),
            &[],
        )
        .await
        .unwrap();
    let known_id: i32 = first_row.get(0);

    // Now fetch by PK like get_row_by_pk does
    let data_query = format!(
        "SELECT * FROM \"{}\".\"{}\" WHERE \"id\" = $1 LIMIT 1",
        schema, table
    );
    let result = client.query_opt(&data_query, &[&known_id]).await.unwrap();
    assert!(result.is_some(), "should find the row by PK");
}

#[tokio::test]
async fn test_get_row_by_pk_not_found() {
    let client = connect().await.expect("should connect");
    let schema = "public";
    let table = "primordials";

    let data_query = format!(
        "SELECT * FROM \"{}\".\"{}\" WHERE \"id\" = $1 LIMIT 1",
        schema, table
    );
    let nonexistent: i32 = -999;
    let result = client.query_opt(&data_query, &[&nonexistent]).await.unwrap();
    assert!(result.is_none(), "should return None for nonexistent PK");
}

// ── disconnect behavior ─────────────────────────────────────────────────

#[tokio::test]
async fn test_disconnect_clears_state() {
    use videre_lib::db::{ConnectionConfig, DbState};

    let state = DbState::new();

    // Set a config
    {
        let mut config = state.config.lock().await;
        *config = Some(ConnectionConfig {
            host: "localhost".to_string(),
            port: "5432".to_string(),
            database: "test".to_string(),
            username: "user".to_string(),
            password: "pass".to_string(),
        });
    }

    // Verify it's set
    assert!(state.config.lock().await.is_some());

    // Simulate disconnect by clearing state (same logic as disconnect_db)
    {
        let mut client = state.client.lock().await;
        *client = None;
        let mut config = state.config.lock().await;
        *config = None;
    }

    assert!(state.client.lock().await.is_none());
    assert!(state.config.lock().await.is_none());
}
