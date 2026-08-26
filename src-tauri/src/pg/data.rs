//! Reading row data out of a table.

use tokio_postgres::Row;
use uuid::Uuid;

use super::convert::{qualified_name, quote_ident, ColumnPlan};
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

        let data_query = format!(
            "SELECT {} FROM {relation} WHERE {} = $1 LIMIT 1",
            plan.select_list()?,
            quote_ident(pk_column)?
        );

        let row = self.query_by_pk(&data_query, pk_value).await?;

        Ok(RowData {
            columns: plan.names(),
            values: plan.decode_row(&row),
        })
    }

    /// Bind `pk_value` and fetch the single matching row.
    ///
    /// The parameter type is inferred from the JSON shape rather than read from
    /// the column, which is why a `bigint` key above 2^31 truncates and a `text`
    /// key holding a UUID-shaped string fails. Both want the type from
    /// `column_plan` instead.
    async fn query_by_pk(
        &self,
        data_query: &str,
        pk_value: &serde_json::Value,
    ) -> Result<Row, String> {
        let row_result = match pk_value {
            serde_json::Value::Number(n) => {
                if let Some(i) = n.as_i64() {
                    self.client.query_opt(data_query, &[&(i as i32)]).await
                } else if let Some(f) = n.as_f64() {
                    self.client.query_opt(data_query, &[&f]).await
                } else {
                    return Err("Invalid number type".to_string());
                }
            }
            serde_json::Value::String(s) => {
                // Try parsing as UUID first
                if let Ok(uuid) = s.parse::<Uuid>() {
                    self.client.query_opt(data_query, &[&uuid]).await
                } else {
                    self.client.query_opt(data_query, &[&s]).await
                }
            }
            _ => return Err("Unsupported primary key type".to_string()),
        };

        match row_result {
            Ok(Some(row)) => Ok(row),
            Ok(None) => Err("Row not found".to_string()),
            Err(e) => Err(format!("Failed to fetch row: {e}")),
        }
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
