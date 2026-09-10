//! Reading row data out of a table.

use super::convert::{bind_key, qualified_name, quote_ident, ColumnPlan};
use super::Connection;
use crate::types::{RowData, SortDirection, TableData};
use tokio_postgres::error::SqlState;

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

    /// Primary-key columns in constraint order. An empty result means the
    /// relation needs the all-column fallback used by [`ColumnPlan::order_by`].
    async fn primary_key_columns(
        &self,
        schema: &str,
        table_name: &str,
    ) -> Result<Vec<String>, String> {
        let query = "
            SELECT attribute.attname
            FROM pg_catalog.pg_class relation
            JOIN pg_catalog.pg_namespace namespace
                ON namespace.oid = relation.relnamespace
            JOIN pg_catalog.pg_index index
                ON index.indrelid = relation.oid
                AND index.indisprimary
            JOIN LATERAL unnest(index.indkey) WITH ORDINALITY
                AS key_column(attnum, position) ON key_column.attnum > 0
            JOIN pg_catalog.pg_attribute attribute
                ON attribute.attrelid = relation.oid
                AND attribute.attnum = key_column.attnum
            WHERE namespace.nspname = $1
                AND relation.relname = $2
            ORDER BY key_column.position
        ";

        self.client
            .query(query, &[&schema, &table_name])
            .await
            .map(|rows| rows.iter().map(|row| row.get(0)).collect())
            .map_err(|e| format!("Failed to inspect primary key: {e}"))
    }

    async fn supports_native_sort(&self, relation: &str, column: &str) -> Result<bool, String> {
        let ident = quote_ident(column)?;
        let probe = format!("SELECT {ident} FROM {relation} ORDER BY {ident} LIMIT 0");

        match self.client.prepare(&probe).await {
            Ok(_) => Ok(true),
            Err(error)
                if error
                    .as_db_error()
                    .is_some_and(|db_error| db_error.code() == &SqlState::UNDEFINED_FUNCTION) =>
            {
                Ok(false)
            }
            Err(error) => Err(format!("Failed to inspect sort support: {error}")),
        }
    }

    pub async fn table_data(
        &self,
        schema: &str,
        table_name: &str,
        limit: i64,
        offset: i64,
        sort_column: Option<&str>,
        sort_direction: SortDirection,
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

        if let Some(column) = sort_column {
            if !plan.has_column(column) {
                return Err(format!("Unknown sort column {column:?}"));
            }
        }

        let primary_key = self.primary_key_columns(schema, table_name).await?;
        let native_sort = match sort_column {
            Some(column) => self.supports_native_sort(&relation, column).await?,
            None => true,
        };
        let data_query = format!(
            "SELECT {} FROM {relation} ORDER BY {} LIMIT $1 OFFSET $2",
            plan.select_list()?,
            plan.order_by(
                &primary_key,
                sort_column,
                sort_direction.is_descending(),
                native_sort,
            )?
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

    /// Mirrors the frontend's `format_value` in `data_table.rs`. What the user
    /// actually sees in a cell.
    fn rendered(value: &Value) -> String {
        match value {
            Value::Null => String::new(),
            Value::String(s) => s.clone(),
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            other => other.to_string(),
        }
    }

    /// Every column type a Postgres inspector can realistically meet, paired
    /// with the value we store in it. Declaration and value sit together so the
    /// DDL and the INSERT can both be generated — adding a type is one line.
    const TYPE_MATRIX: &[(&str, &str, &str)] = &[
        // name, SQL type, SQL literal
        ("c_bool", "boolean", "true"),
        ("c_int2", "smallint", "32767"),
        ("c_int4", "integer", "2147483647"),
        ("c_int8", "bigint", "9223372036854775807"),
        ("c_float4", "real", "0.1"),
        ("c_float8", "double precision", "2.5"),
        (
            "c_numeric",
            "numeric(38,10)",
            "12345678901234567890.1234567890",
        ),
        ("c_numeric_plain", "numeric", "0.30000000000000004"),
        ("c_money", "money", "'1234.56'"),
        ("c_text", "text", "'plain text'"),
        ("c_text_empty", "text", "''"),
        ("c_varchar", "varchar(20)", "'varchar value'"),
        ("c_bpchar", "char(8)", "'ab'"),
        ("c_name", "name", "'a_name'"),
        ("c_char", "\"char\"", "'x'"),
        ("c_bytea", "bytea", "'\\x48656c6c6f'"),
        ("c_uuid", "uuid", "'0b7f2c1e-4a5d-4f8e-9c3a-1d2e3f4a5b6c'"),
        ("c_date", "date", "'2024-01-15'"),
        ("c_time", "time", "'10:30:00'"),
        ("c_timetz", "timetz", "'10:30:00+02'"),
        ("c_timestamp", "timestamp", "'2024-01-15 10:30:00'"),
        ("c_timestamptz", "timestamptz", "'2024-01-15 10:30:00+00'"),
        ("c_interval", "interval", "'3 days 4 hours'"),
        ("c_json", "json", "'{\"a\": 1}'"),
        ("c_jsonb", "jsonb", "'{\"b\": [1, 2]}'"),
        ("c_xml", "xml", "'<r><a>1</a></r>'"),
        ("c_inet", "inet", "'192.168.0.1'"),
        ("c_cidr", "cidr", "'192.168.100.128/25'"),
        ("c_macaddr", "macaddr", "'08:00:2b:01:02:03'"),
        ("c_macaddr8", "macaddr8", "'08:00:2b:01:02:03:04:05'"),
        ("c_bit", "bit(8)", "'10101010'"),
        ("c_varbit", "varbit(16)", "'1010101010101010'"),
        ("c_point", "point", "'(1,2)'"),
        ("c_line", "line", "'{1,2,3}'"),
        ("c_lseg", "lseg", "'((0,0),(1,1))'"),
        ("c_box", "box", "'((1,2),(3,4))'"),
        ("c_path", "path", "'((0,0),(1,1),(2,0))'"),
        ("c_polygon", "polygon", "'((0,0),(1,1),(2,0))'"),
        ("c_circle", "circle", "'<(0,0),5>'"),
        ("c_int4range", "int4range", "'[1,10)'"),
        ("c_numrange", "numrange", "'[1.5,2.5]'"),
        ("c_daterange", "daterange", "'[2024-01-01,2024-02-01)'"),
        (
            "c_tstzrange",
            "tstzrange",
            "'[2024-01-01+00,2024-02-01+00)'",
        ),
        ("c_int4multirange", "int4multirange", "'{[1,5),[10,20)}'"),
        ("c_tsvector", "tsvector", "'a fat cat'"),
        ("c_tsquery", "tsquery", "'fat & rat'"),
        ("c_oid", "oid", "'12345'"),
        ("c_pg_lsn", "pg_lsn", "'16/B374D848'"),
        // User-defined: enum, domain, composite. A domain reports its own OID as
        // the column type, so it exercises the unknown-type path.
        ("c_enum", "videre_types_test.mood", "'wrathful'"),
        ("c_domain", "videre_types_test.short_text", "'ok'"),
        ("c_composite", "videre_types_test.pair", "'(1,x)'"),
        // Arrays, including of exotic element types and multi-dimensional.
        ("c_int4_arr", "integer[]", "ARRAY[1,2,3]"),
        ("c_text_arr", "text[]", "ARRAY['alpha','beta']"),
        (
            "c_uuid_arr",
            "uuid[]",
            "ARRAY['0b7f2c1e-4a5d-4f8e-9c3a-1d2e3f4a5b6c'::uuid]",
        ),
        (
            "c_tstz_arr",
            "timestamptz[]",
            "ARRAY['2024-01-15 10:30:00+00'::timestamptz]",
        ),
        (
            "c_enum_arr",
            "videre_types_test.mood[]",
            "ARRAY['calm'::videre_types_test.mood]",
        ),
        ("c_int4_2d", "integer[][]", "ARRAY[[1,2],[3,4]]"),
        ("c_jsonpath", "jsonpath", "'$.a[*] ? (@ > 2)'"),
    ];

    /// Builds a table with one column per entry in [`TYPE_MATRIX`]: one row of
    /// values and one row that is NULL throughout.
    async fn create_type_matrix(conn: &Connection) {
        let columns_ddl = TYPE_MATRIX
            .iter()
            .map(|(name, sql_type, _)| format!("{name} {sql_type}"))
            .collect::<Vec<_>>()
            .join(",\n                     ");
        let insert_names = TYPE_MATRIX
            .iter()
            .map(|(name, _, _)| *name)
            .collect::<Vec<_>>()
            .join(", ");
        let insert_values = TYPE_MATRIX
            .iter()
            .map(|(_, _, literal)| *literal)
            .collect::<Vec<_>>()
            .join(", ");

        conn.client
            .batch_execute(&format!(
                "DROP SCHEMA IF EXISTS videre_types_test CASCADE;
                 CREATE SCHEMA videre_types_test;
                 CREATE TYPE videre_types_test.mood AS ENUM ('calm', 'wrathful');
                 CREATE TYPE videre_types_test.pair AS (n integer, s text);
                 CREATE DOMAIN videre_types_test.short_text AS text
                     CHECK (length(VALUE) <= 10);
                 CREATE TABLE videre_types_test.every_type (
                     ord integer NOT NULL,
                     {columns_ddl}
                 );
                 INSERT INTO videre_types_test.every_type (ord, {insert_names})
                     VALUES (1, {insert_values});
                 -- Second row leaves every column NULL.
                 INSERT INTO videre_types_test.every_type (ord) VALUES (2);"
            ))
            .await
            .expect("type matrix DDL should apply");
    }

    async fn drop_type_matrix(conn: &Connection) {
        conn.client
            .batch_execute("DROP SCHEMA videre_types_test CASCADE;")
            .await
            .unwrap();
    }

    /// The guarantee: for every type above, what we hand the UI is exactly what
    /// Postgres itself renders, and a cell is blank only when the value really is
    /// an empty string.
    ///
    /// The reference rendering is generated by re-reading each column as `::text`
    /// — the same thing `psql` shows — so a new type added to `TYPE_MATRIX` is
    /// checked with no further work.
    #[tokio::test]
    #[ignore]
    async fn every_type_renders_exactly_as_postgres_does() {
        let conn = connect().await;
        create_type_matrix(&conn).await;

        let relation = qualified_name("videre_types_test", "every_type").unwrap();
        let data = conn
            .table_data(
                "videre_types_test",
                "every_type",
                100,
                0,
                None,
                SortDirection::Asc,
            )
            .await
            .unwrap();

        assert_eq!(
            data.columns.len(),
            TYPE_MATRIX.len() + 1,
            "every declared column should come back"
        );
        assert_eq!(data.total_rows, 2);

        // Reference rendering, generated from the same column list.
        let reference_query = format!(
            "SELECT {} FROM {relation} ORDER BY ord",
            data.columns
                .iter()
                .map(|c| Ok(format!("{}::text", quote_ident(c)?)))
                .collect::<Result<Vec<_>, String>>()
                .unwrap()
                .join(", ")
        );
        let reference = conn.client.query(&reference_query, &[]).await.unwrap();

        let ord = data.columns.iter().position(|c| c == "ord").unwrap();
        let populated = data.rows.iter().find(|r| r[ord] == 1).unwrap();
        let all_null = data.rows.iter().find(|r| r[ord] == 2).unwrap();

        let mut checked = 0;
        for (idx, name) in data.columns.iter().enumerate() {
            let ours = &populated[idx];
            let theirs: Option<String> = reference[0].get(idx);
            let theirs = theirs.expect("populated row has no NULLs except by design");

            // A value must never vanish into null, and must never be the
            // decoder's own error marker.
            assert_ne!(*ours, Value::Null, "{name} rendered as null");
            if let Value::String(s) = ours {
                assert!(!s.starts_with("<unreadable"), "{name}: {s}");
            }

            let shown = rendered(ours);
            let matches = if *name == "c_bpchar" {
                // `character(n)` keeps its padding on the wire, but the cast to
                // text strips trailing blanks. The padded form is the truer one
                // — the blanks are stored data — so compare against it directly.
                assert_eq!(shown, "ab      ", "char(8) should keep its padding");
                true
            } else {
                shown == theirs || numerically_equal(&shown, &theirs)
            };
            assert!(
                matches,
                "{name}: we render {shown:?} but Postgres renders {theirs:?}"
            );
            checked += 1;

            // A genuine NULL must stay NULL, so the UI can show its NULL marker
            // rather than an ambiguous blank.
            if name != "ord" {
                assert_eq!(all_null[idx], Value::Null, "{name} should be NULL");
            }
        }
        assert_eq!(checked, TYPE_MATRIX.len() + 1);

        // The one legitimately blank cell: an actual empty string, which must be
        // an empty string and not a NULL.
        let empty = data
            .columns
            .iter()
            .position(|c| c == "c_text_empty")
            .unwrap();
        assert_eq!(populated[empty], Value::String(String::new()));
        assert_eq!(all_null[empty], Value::Null);

        drop_type_matrix(&conn).await;
    }

    /// Integers and floats can format differently on each side (`1e+30` vs a run
    /// of zeroes) while being the same number.
    fn numerically_equal(ours: &str, theirs: &str) -> bool {
        match (ours.parse::<f64>(), theirs.parse::<f64>()) {
            (Ok(a), Ok(b)) => a == b,
            _ => false,
        }
    }

    /// The exact text a user will see in the cell, pinned by hand.
    ///
    /// `every_type_renders_exactly_as_postgres_does` compares us against the
    /// server, which for server-rendered types is close to tautological — it
    /// proves the value arrives intact but not what it looks like. These are the
    /// literal expected strings, so a change in how anything renders shows up as
    /// a failing assertion rather than passing silently on both sides.
    ///
    /// Several are Postgres *normalizing* the input, which is the behaviour we
    /// want to inherit rather than reimplement.
    const EXPECTED_RENDERING: &[(&str, &str)] = &[
        // Natively decoded — JSON types, not strings.
        ("c_bool", "true"),
        ("c_int2", "32767"),
        ("c_int8", "9223372036854775807"),
        ("c_float4", "0.1"),
        ("c_float8", "2.5"),
        ("c_text", "plain text"),
        ("c_name", "a_name"),
        // `character(n)` keeps the blanks it pads with; they are stored data.
        ("c_bpchar", "ab      "),
        // The one legitimately blank cell.
        ("c_text_empty", ""),
        // Server-rendered, verbatim.
        ("c_numeric", "12345678901234567890.1234567890"),
        ("c_numeric_plain", "0.30000000000000004"),
        ("c_uuid", "0b7f2c1e-4a5d-4f8e-9c3a-1d2e3f4a5b6c"),
        ("c_date", "2024-01-15"),
        ("c_timestamptz", "2024-01-15 10:30:00+00"),
        ("c_timetz", "10:30:00+02"),
        ("c_bytea", "\\x48656c6c6f"),
        ("c_jsonb", "{\"b\": [1, 2]}"),
        ("c_xml", "<r><a>1</a></r>"),
        ("c_bit", "10101010"),
        ("c_macaddr8", "08:00:2b:01:02:03:04:05"),
        ("c_cidr", "192.168.100.128/25"),
        ("c_pg_lsn", "16/B374D848"),
        ("c_point", "(1,2)"),
        ("c_int4range", "[1,10)"),
        ("c_int4multirange", "{[1,5),[10,20)}"),
        ("c_text_arr", "{alpha,beta}"),
        ("c_int4_2d", "{{1,2},{3,4}}"),
        ("c_enum", "wrathful"),
        ("c_domain", "ok"),
        ("c_composite", "(1,x)"),
        ("c_char", "x"),
        // Server-rendered *and normalized* — the input literal differs.
        ("c_money", "$1,234.56"),
        ("c_interval", "3 days 04:00:00"),
        ("c_tsvector", "'a' 'cat' 'fat'"),
        ("c_tsquery", "'fat' & 'rat'"),
        ("c_jsonpath", "$.\"a\"[*]?(@ > 2)"),
        ("c_inet", "192.168.0.1/32"),
        ("c_box", "(3,4),(1,2)"),
        ("c_lseg", "[(0,0),(1,1)]"),
        (
            "c_tstzrange",
            "[\"2024-01-01 00:00:00+00\",\"2024-02-01 00:00:00+00\")",
        ),
    ];

    #[tokio::test]
    #[ignore]
    async fn cells_show_the_exact_expected_text() {
        let conn = connect().await;
        create_type_matrix(&conn).await;

        let data = conn
            .table_data(
                "videre_types_test",
                "every_type",
                100,
                0,
                None,
                SortDirection::Asc,
            )
            .await
            .unwrap();
        let ord = data.columns.iter().position(|c| c == "ord").unwrap();
        let row = data.rows.iter().find(|r| r[ord] == 1).unwrap();

        for (name, expected) in EXPECTED_RENDERING {
            let idx = data
                .columns
                .iter()
                .position(|c| c == name)
                .unwrap_or_else(|| panic!("{name} is not in the type matrix"));
            assert_eq!(
                rendered(&row[idx]),
                *expected,
                "{name} renders differently than expected"
            );
        }

        // Natively decoded columns must arrive as JSON numbers and booleans, not
        // strings — the UI right-aligns on that distinction.
        let of = |name: &str| row[data.columns.iter().position(|c| c == name).unwrap()].clone();
        assert!(of("c_int8").is_number(), "int8 should stay a JSON number");
        assert!(
            of("c_float8").is_number(),
            "float8 should stay a JSON number"
        );
        assert!(of("c_bool").is_boolean(), "bool should stay a JSON bool");
        // `numeric` is deliberately a string: f64 would lose its precision.
        assert!(of("c_numeric").is_string(), "numeric should stay text");

        drop_type_matrix(&conn).await;
    }

    /// The reported bug: `created_at` is `timestamptz` in every seeded table, and
    /// the old try_get chain had no arm for it, so it rendered as `null`.
    #[tokio::test]
    #[ignore]
    async fn seeded_timestamps_are_no_longer_null() {
        let conn = connect().await;
        let data = conn
            .table_data("public", "gods", 5, 0, None, SortDirection::Asc)
            .await
            .unwrap();

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
            .table_data(
                "videre_convert_test",
                "every_type",
                10,
                0,
                None,
                SortDirection::Asc,
            )
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

    #[tokio::test]
    #[ignore]
    async fn pagination_is_deterministic_with_and_without_a_primary_key() {
        let conn = connect().await;
        conn.client
            .batch_execute(
                "DROP SCHEMA IF EXISTS videre_pagination_test CASCADE;
                 CREATE SCHEMA videre_pagination_test;
                 CREATE TABLE videre_pagination_test.with_pk (
                     tenant_id integer NOT NULL,
                     entry_id integer NOT NULL,
                     label text NOT NULL,
                     PRIMARY KEY (tenant_id, entry_id)
                 );
                 INSERT INTO videre_pagination_test.with_pk VALUES
                     (2, 2, 'fourth'), (1, 2, 'second'),
                     (2, 1, 'third'), (1, 1, 'first');
                 CREATE TABLE videre_pagination_test.without_pk (
                     label text,
                     rank integer
                 );
                 INSERT INTO videre_pagination_test.without_pk VALUES
                     ('charlie', 3), ('alpha', 1), ('bravo', 2);",
            )
            .await
            .unwrap();

        let first = conn
            .table_data(
                "videre_pagination_test",
                "with_pk",
                2,
                0,
                None,
                SortDirection::Asc,
            )
            .await
            .unwrap();
        let second = conn
            .table_data(
                "videre_pagination_test",
                "with_pk",
                2,
                2,
                None,
                SortDirection::Asc,
            )
            .await
            .unwrap();
        let pk_rows = first
            .rows
            .into_iter()
            .chain(second.rows)
            .map(|row| (row[0].as_i64().unwrap(), row[1].as_i64().unwrap()))
            .collect::<Vec<_>>();
        assert_eq!(pk_rows, vec![(1, 1), (1, 2), (2, 1), (2, 2)]);

        let first = conn
            .table_data(
                "videre_pagination_test",
                "without_pk",
                2,
                0,
                None,
                SortDirection::Asc,
            )
            .await
            .unwrap();
        let second = conn
            .table_data(
                "videre_pagination_test",
                "without_pk",
                2,
                2,
                None,
                SortDirection::Asc,
            )
            .await
            .unwrap();
        let labels = first
            .rows
            .into_iter()
            .chain(second.rows)
            .map(|row| row[0].as_str().unwrap().to_string())
            .collect::<Vec<_>>();
        assert_eq!(labels, vec!["alpha", "bravo", "charlie"]);

        conn.client
            .batch_execute("DROP SCHEMA videre_pagination_test CASCADE;")
            .await
            .unwrap();
    }

    #[tokio::test]
    #[ignore]
    async fn server_sorting_uses_postgres_types_across_pages() {
        let conn = connect().await;
        conn.client
            .batch_execute(
                "DROP SCHEMA IF EXISTS videre_sort_test CASCADE;
                 CREATE SCHEMA videre_sort_test;
                 CREATE TABLE videre_sort_test.entries (
                     id integer PRIMARY KEY,
                     name text NOT NULL,
                     score integer,
                     occurred_on date NOT NULL,
                     payload json NOT NULL
                 );
                 INSERT INTO videre_sort_test.entries VALUES
                     (1, 'Zeus', 10, '2024-01-10', '{\"n\": 2}'),
                     (2, 'Athena', 2, '2024-01-02', '{\"n\": 10}'),
                     (3, 'Apollo', 2, '2024-01-03', '{\"n\": 3}'),
                     (4, 'Hera', NULL, '2024-01-01', '{\"n\": 4}');",
            )
            .await
            .unwrap();

        let page = |offset, column, direction| {
            conn.table_data(
                "videre_sort_test",
                "entries",
                2,
                offset,
                Some(column),
                direction,
            )
        };

        let first = page(0, "score", SortDirection::Asc).await.unwrap();
        let second = page(2, "score", SortDirection::Asc).await.unwrap();
        let ids = first
            .rows
            .into_iter()
            .chain(second.rows)
            .map(|row| row[0].as_i64().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec![2, 3, 1, 4]);

        let descending = conn
            .table_data(
                "videre_sort_test",
                "entries",
                10,
                0,
                Some("score"),
                SortDirection::Desc,
            )
            .await
            .unwrap();
        let ids = descending
            .rows
            .iter()
            .map(|row| row[0].as_i64().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec![1, 2, 3, 4], "NULL remains last");

        let by_date = conn
            .table_data(
                "videre_sort_test",
                "entries",
                10,
                0,
                Some("occurred_on"),
                SortDirection::Asc,
            )
            .await
            .unwrap();
        let ids = by_date
            .rows
            .iter()
            .map(|row| row[0].as_i64().unwrap())
            .collect::<Vec<_>>();
        assert_eq!(ids, vec![4, 2, 3, 1]);

        let by_json = conn
            .table_data(
                "videre_sort_test",
                "entries",
                10,
                0,
                Some("payload"),
                SortDirection::Asc,
            )
            .await
            .unwrap();
        assert_eq!(by_json.rows.len(), 4, "json falls back to text ordering");

        let error = conn
            .table_data(
                "videre_sort_test",
                "entries",
                10,
                0,
                Some("id; DROP TABLE entries"),
                SortDirection::Asc,
            )
            .await
            .unwrap_err();
        assert!(error.contains("Unknown sort column"), "{error}");

        conn.client
            .batch_execute("DROP SCHEMA videre_sort_test CASCADE;")
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
