//! Reading row data out of a table.

use super::convert::{bind_key, qualified_name, quote_ident, ColumnPlan};
use super::Connection;
use crate::types::{RowData, TableData};

impl Connection {
    /// Ask the server for the shape of `SELECT *` before reading any rows.
    ///
    /// `prepare` only parses and plans, so this is cheaper than the
    /// `information_schema.columns` lookup it replaces — and it gives us the
    /// column types as well as the names, from the exact statement we run.
    async fn column_plan(&self, relation: &str) -> Result<ColumnPlan, String> {
        let stmt = self
            .client
            .prepare(&format!("SELECT * FROM {relation}"))
            .await
            .map_err(|e| format!("Failed to inspect columns: {e}"))?;
        Ok(ColumnPlan::from_columns(stmt.columns()))
    }

    pub async fn table_data(
        &self,
        schema: &str,
        table_name: &str,
        limit: i64,
        offset: i64,
    ) -> Result<TableData, String> {
        let relation = qualified_name(schema, table_name)?;
        let plan = self.column_plan(&relation).await?;

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
        let rows = self
            .client
            .query(&data_query, &[&limit, &offset])
            .await
            .map_err(|e| format!("Failed to fetch data: {e}"))?;

        let count_query = format!("SELECT COUNT(*) FROM {relation}");
        let total_rows: i64 = self
            .client
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

    pub async fn row_by_pk(
        &self,
        schema: &str,
        table_name: &str,
        pk_column: &str,
        pk_value: &serde_json::Value,
    ) -> Result<RowData, String> {
        let relation = qualified_name(schema, table_name)?;
        let plan = self.column_plan(&relation).await?;

        if plan.is_empty() {
            return Err(format!("{relation} has no columns"));
        }

        // Both the comparison and the parameter come from the column's own type,
        // so Postgres gets what it inferred for `$1`. Guessing from the JSON
        // value's shape is what broke every non-`int4` key.
        let predicate = plan
            .key_predicate(pk_column)
            .ok_or_else(|| format!("{relation} has no column named {pk_column:?}"))?;
        let param = bind_key(pk_value, &predicate)?;

        let data_query = format!(
            "SELECT {} FROM {relation} WHERE {} = $1 LIMIT 1",
            plan.select_list()?,
            predicate.lhs(&quote_ident(pk_column)?)
        );

        let row = self
            .client
            .query_opt(&data_query, &[&*param])
            .await
            .map_err(|e| format!("Failed to fetch row: {e}"))?
            .ok_or_else(|| "Row not found".to_string())?;

        Ok(RowData {
            columns: plan.names(),
            values: plan.decode_row(&row),
        })
    }
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
    use crate::types::ConnectionConfig;
    use serde_json::Value;

    fn test_config() -> ConnectionConfig {
        let var =
            |name: &str, default: &str| std::env::var(name).unwrap_or_else(|_| default.to_string());
        ConnectionConfig {
            host: var("VIDERE_TEST_PG_HOST", "localhost"),
            port: var("VIDERE_TEST_PG_PORT", "5432"),
            database: var("VIDERE_TEST_PG_DB", "videre_test"),
            username: var("VIDERE_TEST_PG_USER", "videre"),
            password: var("VIDERE_TEST_PG_PASSWORD", "videre"),
        }
    }

    async fn connect() -> Connection {
        Connection::connect(&test_config())
            .await
            .expect("test database unreachable — is `docker-compose up -d` running?")
    }

    /// The reported bug: `created_at` is `timestamptz` in every seeded table, and
    /// the old try_get chain had no arm for it, so it rendered as `null`.
    #[tokio::test]
    #[ignore]
    async fn seeded_timestamps_are_no_longer_null() {
        let conn = connect().await;
        let data = conn.table_data("public", "gods", 5, 0).await.unwrap();

        let created_at = data
            .columns
            .iter()
            .position(|n| n == "created_at")
            .expect("gods.created_at should exist");

        assert!(!data.rows.is_empty(), "gods should have seed data");
        for row in &data.rows {
            let cell = &row[created_at];
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
        let conn = connect().await;
        conn.client
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

        let data = conn
            .table_data("videre_convert_test", "every_type", 10, 0)
            .await
            .unwrap();
        let names = &data.columns;
        let values = &data.rows[0];

        for (name, value) in names.iter().zip(values) {
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

        conn.client
            .batch_execute("DROP SCHEMA videre_convert_test CASCADE;")
            .await
            .unwrap();
    }

    /// Every one of these key types used to fail with `error serializing
    /// parameter 0`, because the parameter was bound from the JSON value's shape
    /// instead of the column's type. Only `int4` worked.
    #[tokio::test]
    #[ignore]
    async fn keys_of_every_type_resolve() {
        let conn = connect().await;
        conn.client
            .batch_execute(
                "DROP SCHEMA IF EXISTS videre_key_test CASCADE;
                 CREATE SCHEMA videre_key_test;
                 CREATE TABLE videre_key_test.k_int8 (id bigint PRIMARY KEY, label text);
                 INSERT INTO videre_key_test.k_int8 VALUES (42, 'small'), (9007199254740993, 'big');
                 CREATE TABLE videre_key_test.k_int2 (id smallint PRIMARY KEY, label text);
                 INSERT INTO videre_key_test.k_int2 VALUES (7, 'seven');
                 CREATE TABLE videre_key_test.k_int4 (id integer PRIMARY KEY, label text);
                 INSERT INTO videre_key_test.k_int4 VALUES (3, 'three');
                 CREATE TABLE videre_key_test.k_uuid (id uuid PRIMARY KEY, label text);
                 INSERT INTO videre_key_test.k_uuid
                     VALUES ('0b7f2c1e-4a5d-4f8e-9c3a-1d2e3f4a5b6c', 'by-uuid');
                 CREATE TABLE videre_key_test.k_text (id text PRIMARY KEY, label text);
                 INSERT INTO videre_key_test.k_text
                     VALUES ('0b7f2c1e-4a5d-4f8e-9c3a-1d2e3f4a5b6c', 'uuid-shaped-text');
                 CREATE TABLE videre_key_test.k_numeric (id numeric(20,4) PRIMARY KEY, label text);
                 INSERT INTO videre_key_test.k_numeric VALUES (12345.6700, 'by-numeric');
                 CREATE TABLE videre_key_test.k_date (id date PRIMARY KEY, label text);
                 INSERT INTO videre_key_test.k_date VALUES ('2024-01-15', 'by-date');",
            )
            .await
            .unwrap();

        let cases = [
            ("k_int8", serde_json::json!(42), "small"),
            // Sent as a string: past 2^53 that is the only lossless JSON form.
            ("k_int8", serde_json::json!("9007199254740993"), "big"),
            ("k_int2", serde_json::json!(7), "seven"),
            ("k_int4", serde_json::json!(3), "three"),
            (
                "k_uuid",
                serde_json::json!("0b7f2c1e-4a5d-4f8e-9c3a-1d2e3f4a5b6c"),
                "by-uuid",
            ),
            (
                "k_text",
                serde_json::json!("0b7f2c1e-4a5d-4f8e-9c3a-1d2e3f4a5b6c"),
                "uuid-shaped-text",
            ),
            // Exotic key types go through the server's text form in both
            // directions, so the value read out of a cell is what matches.
            ("k_numeric", serde_json::json!("12345.6700"), "by-numeric"),
            ("k_date", serde_json::json!("2024-01-15"), "by-date"),
        ];

        for (table, key, expected_label) in cases {
            let row = conn
                .row_by_pk("videre_key_test", table, "id", &key)
                .await
                .unwrap_or_else(|e| panic!("{table} with key {key}: {e}"));
            let label = row.columns.iter().position(|c| c == "label").unwrap();
            assert_eq!(
                row.values[label],
                Value::String(expected_label.into()),
                "{table} with key {key}"
            );
        }

        // A key too large for its column is reported, not wrapped into a
        // different row.
        let err = conn
            .row_by_pk(
                "videre_key_test",
                "k_int4",
                "id",
                &serde_json::json!(2147483648i64),
            )
            .await
            .unwrap_err();
        assert!(err.contains("out of range"), "{err}");

        // An unknown column is caught before it becomes SQL.
        let err = conn
            .row_by_pk("videre_key_test", "k_int4", "nope", &serde_json::json!(3))
            .await
            .unwrap_err();
        assert!(err.contains("no column named"), "{err}");

        conn.client
            .batch_execute("DROP SCHEMA videre_key_test CASCADE;")
            .await
            .unwrap();
    }

    /// The catalog reads, exercised end to end against the seed schema.
    #[tokio::test]
    #[ignore]
    async fn catalog_reads_return_seed_data() {
        let conn = connect().await;

        let tables = conn.tables().await.unwrap();
        assert!(tables
            .iter()
            .any(|t| t.name == "gods" && t.schema == "public"));

        let indexes = conn.indexes("public", "gods").await.unwrap();
        assert!(indexes.iter().any(|i| i.is_primary));

        let fks = conn.foreign_keys("public", "gods").await.unwrap();
        assert!(!fks.is_empty(), "gods has foreign keys in init-db.sql");

        assert!(!conn.roles().await.unwrap().is_empty());
        assert!(!conn.table_privileges().await.unwrap().is_empty());
    }
}
