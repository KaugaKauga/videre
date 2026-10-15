use super::*;
use crate::state::DbState;
use serde_json::json;

fn unique_name(prefix: &str) -> String {
    use std::sync::atomic::{AtomicU64, Ordering};
    static NEXT: AtomicU64 = AtomicU64::new(0);
    format!(
        "{prefix}_{}_{}_{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )
}

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

async fn query(registry: &QueryRegistry, sql: &str, id: &str) -> Result<QueryResult, String> {
    registry
        .register(sql, id)?
        .execute(sql, async { Ok(test_config()) })
        .await
}

#[test]
fn fetch_batches_stop_at_the_single_extra_row() {
    for retained in [0, 16, 992, 999, 1000] {
        let count = preview_fetch_size(retained);
        assert!(count > 0 && count <= FETCH_BATCH_SIZE);
        assert!(count <= ROW_LIMIT + 1 - retained);
    }
    assert_eq!(preview_fetch_size(992), 9);
    assert_eq!(preview_fetch_size(1000), 1);
    let mut fetched = 0;
    while fetched <= ROW_LIMIT {
        fetched += preview_fetch_size(fetched);
    }
    assert_eq!(fetched, ROW_LIMIT + 1);
}

#[test]
fn row_budget_counts_utf8_all_cells_and_container_storage() {
    let overhead = std::mem::size_of::<Vec<Value>>() + std::mem::size_of::<Value>();
    let exact = "x".repeat(MAX_ROW_BYTES - overhead);
    assert_eq!(
        preview_row_bytes([Some(exact.as_str())].into_iter()).unwrap(),
        MAX_ROW_BYTES
    );
    let excessive = format!("{exact}x");
    assert_eq!(
        preview_row_bytes([Some(excessive.as_str())].into_iter()).unwrap_err(),
        ROW_BUDGET_ERROR
    );
    assert_eq!(
        preview_row_bytes([Some("é")].into_iter()).unwrap(),
        overhead + 2
    );
    assert_eq!(preview_row_bytes([None].into_iter()).unwrap(), overhead);
    let cell = "x".repeat(MAX_ROW_BYTES / 2);
    assert!(preview_row_bytes([Some(cell.as_str()), Some(cell.as_str())].into_iter()).is_err());
}

#[test]
fn preview_budget_accepts_the_boundary_and_does_not_charge_rejected_rows() {
    let mut bytes = MAX_PREVIEW_BYTES - MAX_ROW_BYTES;
    reserve_preview_bytes(&mut bytes, MAX_ROW_BYTES).unwrap();
    assert_eq!(bytes, MAX_PREVIEW_BYTES);
    assert_eq!(
        reserve_preview_bytes(&mut bytes, 1).unwrap_err(),
        PREVIEW_BUDGET_ERROR
    );
    assert_eq!(bytes, MAX_PREVIEW_BYTES);
    assert!(reserve_preview_bytes(&mut bytes, usize::MAX).is_err());
    assert_eq!(bytes, MAX_PREVIEW_BYTES);
}

#[test]
fn metadata_is_bounded_before_copying_names() {
    assert_eq!(
        metadata_bytes(["é", "same", "same"].into_iter()).unwrap(),
        3 * std::mem::size_of::<String>() + 10
    );
    assert!(metadata_bytes(std::iter::repeat("x").take(MAX_COLUMNS)).is_ok());
    let count_error = metadata_bytes(std::iter::repeat("x").take(MAX_COLUMNS + 1)).unwrap_err();
    assert!(count_error.contains("column preview limit"));
    let exact = "x".repeat(MAX_METADATA_BYTES - std::mem::size_of::<String>());
    assert_eq!(
        metadata_bytes([exact.as_str()].into_iter()).unwrap(),
        MAX_METADATA_BYTES
    );
    let too_long = format!("{exact}x");
    let size_error = metadata_bytes([too_long.as_str()].into_iter()).unwrap_err();
    assert!(size_error.contains("64 KiB"));
}

#[test]
fn input_is_bounded_without_keyword_filtering() {
    assert!(validate_input(" \n", "id").is_err());
    assert!(validate_input("SELECT 1", " ").is_err());
    assert!(validate_input("SELECT 1", &"x".repeat(MAX_REQUEST_ID_BYTES + 1)).is_err());
    assert!(validate_input(&"x".repeat(MAX_SQL_BYTES + 1), "id").is_err());
    assert!(validate_input(&"x".repeat(MAX_SQL_BYTES), "id").is_ok());
    assert!(validate_input("/* comment */ WITH x AS (SELECT 1) SELECT * FROM x", "id").is_ok());
}

#[test]
fn registry_rejects_duplicates_and_limits_concurrency_and_releases_slots() {
    let registry = QueryRegistry::default();
    let mut requests = Vec::new();
    for n in 0..MAX_RUNNING_QUERIES {
        requests.push(registry.register("SELECT 1", &n.to_string()).unwrap());
    }
    assert!(registry
        .register("SELECT 1", "0")
        .err()
        .unwrap()
        .contains("already registered"));
    assert!(registry
        .register("SELECT 1", "extra")
        .err()
        .unwrap()
        .contains("concurrently"));
    requests.pop();
    assert!(registry.register("SELECT 1", "extra").is_ok());
    drop(requests);
    assert!(registry.entries.lock().unwrap().is_empty());
}

#[tokio::test]
async fn missing_cancellation_is_an_error_not_a_false_acknowledgement() {
    let registry = QueryRegistry::default();
    assert!(registry
        .cancel("missing")
        .await
        .unwrap_err()
        .contains("not registered"));
}

#[tokio::test]
async fn cancellation_during_config_lookup_cleans_the_entry() {
    let state = Arc::new(DbState::new());
    let browsing_lock = state.connection.lock().await;
    let worker_state = state.clone();
    let worker =
        tokio::spawn(async move { worker_state.execute_query("SELECT 1", "waiting").await });
    tokio::task::yield_now().await;
    timeout(Duration::from_secs(2), state.queries.cancel("waiting"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(worker.await.unwrap().unwrap_err(), "Query cancelled");
    assert!(state.queries.entries.lock().unwrap().is_empty());
    drop(browsing_lock);
    assert!(state
        .connection_info()
        .await
        .unwrap_err()
        .contains("Not connected"));
}

#[tokio::test]
async fn cancellation_during_connection_startup_closes_the_socket() {
    use tokio::io::AsyncReadExt;

    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
    let mut config = test_config();
    config.host = "127.0.0.1".into();
    config.port = listener.local_addr().unwrap().port().to_string();
    let registry = QueryRegistry::default();
    let request = registry.register("SELECT 1", "connecting").unwrap();
    let worker =
        tokio::spawn(async move { request.execute("SELECT 1", async { Ok(config) }).await });
    let (mut socket, _) = timeout(Duration::from_secs(2), listener.accept())
        .await
        .unwrap()
        .unwrap();
    timeout(Duration::from_secs(2), registry.cancel("connecting"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(worker.await.unwrap().unwrap_err(), "Query cancelled");
    let mut bytes = Vec::new();
    timeout(Duration::from_secs(2), socket.read_to_end(&mut bytes))
        .await
        .unwrap()
        .unwrap();
    assert!(registry.entries.lock().unwrap().is_empty());
}

#[tokio::test]
async fn connection_errors_release_registration() {
    let registry = QueryRegistry::default();
    let request = registry.register("SELECT 1", "bad-port").unwrap();
    let mut config = test_config();
    config.port = "not-a-port".into();
    let error = request
        .execute("SELECT 1", async { Ok(config) })
        .await
        .unwrap_err();
    assert!(error.contains("Invalid PostgreSQL port"));
    assert!(registry.entries.lock().unwrap().is_empty());
}

#[tokio::test]
async fn cancellation_before_execution_and_cancel_all_release_entries() {
    let registry = QueryRegistry::default();
    let request = registry.register("SELECT 1", "pending").unwrap();
    registry.cancel_all();
    assert_eq!(
        request
            .execute("SELECT 1", std::future::pending())
            .await
            .unwrap_err(),
        "Query cancelled"
    );
    assert!(registry.entries.lock().unwrap().is_empty());
}

#[tokio::test]
async fn dropping_the_execution_future_releases_registration() {
    let registry = QueryRegistry::default();
    let request = registry.register("SELECT 1", "dropped").unwrap();
    let worker = tokio::spawn(request.execute("SELECT 1", std::future::pending()));
    tokio::task::yield_now().await;
    worker.abort();
    assert!(worker.await.unwrap_err().is_cancelled());
    assert!(registry.entries.lock().unwrap().is_empty());
}

#[tokio::test]
async fn not_connected_errors_release_registration() {
    let state = DbState::new();
    assert!(state
        .execute_query("SELECT 1", "offline")
        .await
        .unwrap_err()
        .contains("Not connected"));
    assert!(state.queries.entries.lock().unwrap().is_empty());
}

#[test]
fn ipc_result_has_only_the_requested_fields_and_never_passwords() {
    let config = test_config();
    let result = QueryResult {
        columns: vec!["?column?".into()],
        column_sources: vec![None],
        rows: vec![vec![Value::String("1".into()), Value::Null]],
        truncated: false,
        row_limit: ROW_LIMIT,
        elapsed_ms: 12,
        connection: ConnectionInfo::from(&config),
    };
    let value = serde_json::to_value(result).unwrap();
    assert_eq!(value.as_object().unwrap().len(), 7);
    assert_eq!(value["column_sources"], json!([null]));
    assert_eq!(value["rows"], json!([["1", null]]));
    assert_eq!(value["row_limit"], 1000);
    assert_eq!(value["elapsed_ms"], 12);
    assert_eq!(value["connection"].as_object().unwrap().len(), 4);
    assert!(value["connection"].get("password").is_none());
}

#[tokio::test]
#[ignore]
async fn live_select_joins_grouping_with_and_values() {
    let registry = QueryRegistry::default();
    let result = query(&registry, "/* editor SQL */ WITH a(id, amount) AS (VALUES (1, 2), (1, 3), (2, 7)), b(id, label) AS (VALUES (1, 'one'), (2, 'two')) SELECT b.label, sum(a.amount) AS total FROM a JOIN b USING (id) GROUP BY b.label ORDER BY b.label; -- trailing comment", "joins").await.unwrap();
    assert_eq!(result.columns, vec!["label", "total"]);
    assert_eq!(
        result.rows,
        vec![
            vec![json!("one"), json!("5")],
            vec![json!("two"), json!("7")]
        ]
    );
    assert!(!result.truncated);
    assert_eq!(result.connection, ConnectionInfo::from(&test_config()));
    assert_eq!(
        query(&registry, "VALUES (1), (2)", "values")
            .await
            .unwrap()
            .rows,
        vec![vec![json!("1")], vec![json!("2")]]
    );
    assert!(registry.entries.lock().unwrap().is_empty());
}

#[test]
fn only_direct_table_columns_have_origins() {
    let origins = column_origins(
        [
            (Some(16384), Some(2)),
            (Some(0), Some(0)),
            (None, None),
            (Some(16384), Some(0)),
            (Some(16390), Some(1)),
        ]
        .into_iter(),
    );
    assert_eq!(origins, vec![(0, 16384, 2), (4, 16390, 1)]);
}

#[tokio::test]
#[ignore]
async fn live_result_columns_report_their_table_column_sources() {
    let registry = QueryRegistry::default();
    let result = query(
        &registry,
        "SELECT c.relname, c.relname AS alias, 1 AS computed, upper(c.relname) FROM pg_catalog.pg_class c LIMIT 1",
        "sources",
    )
    .await
    .unwrap();
    let relname = Some(ResultColumnSource {
        schema: "pg_catalog".into(),
        relation: "pg_class".into(),
        column: "relname".into(),
    });
    assert_eq!(
        result.column_sources,
        vec![relname.clone(), relname, None, None]
    );
}

#[tokio::test]
#[ignore]
async fn live_zero_rows_duplicate_and_unnamed_columns() {
    let registry = QueryRegistry::default();
    let empty = query(
        &registry,
        "SELECT 1 AS same, 2 AS same, 3 + 4 WHERE false",
        "empty",
    )
    .await
    .unwrap();
    assert_eq!(empty.columns, vec!["same", "same", "?column?"]);
    assert!(empty.rows.is_empty());
    assert!(!empty.truncated);
    let data = query(&registry, "SELECT 1 AS same, 2 AS same, 3 + 4", "duplicate")
        .await
        .unwrap();
    assert_eq!(data.columns, empty.columns);
    assert_eq!(data.rows, vec![vec![json!("1"), json!("2"), json!("7")]]);
}

#[tokio::test]
#[ignore]
async fn live_exact_postgresql_text_and_nulls_for_arbitrary_types() {
    let registry = QueryRegistry::default();
    let sql = "SELECT true, 9223372036854775807::bigint, 12345678901234567890.1234567890::numeric, 0.1::real, 'NaN'::float8, 'x'::char(4), ''::text, E'\\\\x00ff'::bytea, ARRAY[1, NULL, 3], '{\"b\": 1, \"a\": 2}'::jsonb, '2024-01-02 03:04:05.123456'::timestamp, '(1,2)'::point, '[1,5)'::int4range, ROW(1, 'x'), 'a & b'::tsquery, NULL::integer, false, NULL::boolean";
    let reference = super::super::Connection::connect(&test_config())
        .await
        .unwrap();
    let expected = reference
        .client
        .simple_query(sql)
        .await
        .unwrap()
        .into_iter()
        .filter_map(|message| {
            if let SimpleQueryMessage::Row(row) = message {
                Some(
                    (0..row.len())
                        .map(|i| row.get(i).map_or(Value::Null, |text| json!(text)))
                        .collect::<Vec<_>>(),
                )
            } else {
                None
            }
        })
        .collect::<Vec<_>>();
    let result = query(&registry, sql, "types").await.unwrap();
    // Booleans are the one type shown as JSON values, like the table view.
    let mut expected = expected;
    assert_eq!(
        (&expected[0][0], &expected[0][16]),
        (&json!("t"), &json!("f"))
    );
    expected[0][0] = json!(true);
    expected[0][16] = json!(false);
    assert_eq!(result.rows, expected);
    assert_eq!(result.rows[0][17], Value::Null);
    assert_eq!(result.rows[0][5], json!("x   "));
    assert_eq!(result.rows[0][6], json!(""));
    assert_eq!(result.rows[0][15], Value::Null);
}

#[tokio::test]
#[ignore]
async fn live_preview_reads_exactly_one_extra_row() {
    let registry = QueryRegistry::default();
    for count in [0, 1, 15, 16, 17, 31, 32, 33, 992, 999, 1000, 1001, 10000] {
        let result = query(
            &registry,
            &format!("SELECT n FROM generate_series(1, {count}) AS n ORDER BY n"),
            "cap",
        )
        .await
        .unwrap();
        assert_eq!(result.rows.len(), (count as usize).min(ROW_LIMIT));
        assert_eq!(result.truncated, count > ROW_LIMIT);
        assert_eq!(result.row_limit, ROW_LIMIT);
        if count > 0 {
            assert_eq!(result.rows[0], vec![json!("1")]);
            assert_eq!(
                result.rows.last().unwrap(),
                &vec![json!(count.min(ROW_LIMIT).to_string())]
            );
        }
    }
    // Evaluation of row 1002 would fail: the final batch must not fetch past
    // the one sentinel even though a full 16-row request would do so.
    let result = query(
        &registry,
        "SELECT n, 1 / CASE WHEN n <= 1001 THEN 1 ELSE 0 END FROM generate_series(1, 1002) AS n",
        "no-overfetch",
    )
    .await
    .unwrap();
    assert_eq!(result.rows.len(), ROW_LIMIT);
    assert!(result.truncated);
    assert_eq!(
        result.rows.last().unwrap(),
        &vec![json!("1000"), json!("1")]
    );
}

#[tokio::test]
#[ignore]
async fn live_preview_payload_rejects_oversized_rows() {
    let registry = QueryRegistry::default();
    let result = query(&registry, "SELECT repeat('x', 2097152)", "oversized-row").await;
    assert!(
        result.is_err(),
        "a 2 MiB row must exceed the preview row budget"
    );
    let error = result.err().unwrap();
    assert!(
        error.contains("1 MiB")
            && error.contains("row")
            && error.contains("narrower columns or smaller values"),
        "{error}"
    );
    assert!(registry.entries.lock().unwrap().is_empty());
    assert_eq!(
        query(&registry, "SELECT 1", "oversized-row")
            .await
            .unwrap()
            .rows,
        vec![vec![json!("1")]]
    );
}

#[tokio::test]
#[ignore]
async fn live_preview_payload_preserves_text_across_batches() {
    let registry = QueryRegistry::default();
    let result = query(
        &registry,
        "SELECT n, repeat('x', 262144) FROM generate_series(1, 20) AS n",
        "payload-ok",
    )
    .await
    .unwrap();
    assert_eq!(result.rows.len(), 20);
    assert!(!result.truncated);
    for (index, row) in result.rows.iter().enumerate() {
        assert_eq!(row[0], json!((index + 1).to_string()));
        let text = row[1].as_str().unwrap();
        assert_eq!(text.len(), 262144);
        assert!(text.bytes().all(|byte| byte == b'x'));
    }
}

#[tokio::test]
#[ignore]
async fn live_preview_payload_rejects_cumulative_overflow() {
    let registry = QueryRegistry::default();
    // Thirty-six 256 KiB rows total 9 MiB: safe for the fixture and individually
    // below the row budget. Overflow occurs across batches, not just within one.
    let result = query(
        &registry,
        "SELECT repeat('x', 262144) FROM generate_series(1, 36)",
        "oversized-result",
    )
    .await;
    assert!(
        result.is_err(),
        "a 9 MiB result must exceed the cumulative preview budget"
    );
    let error = result.err().unwrap();
    assert!(
        error.contains("8 MiB")
            && error.contains("preview")
            && error.contains("narrower columns or smaller values"),
        "{error}"
    );
    assert!(registry.entries.lock().unwrap().is_empty());
    assert_eq!(
        query(&registry, "SELECT 2", "oversized-result")
            .await
            .unwrap()
            .rows,
        vec![vec![json!("2")]]
    );
}

#[tokio::test]
#[ignore]
async fn live_syntax_parameters_multistatements_and_incompatible_commands() {
    let registry = QueryRegistry::default();
    let syntax = query(&registry, "SELECT FROM", "syntax").await.unwrap_err();
    assert!(syntax.contains("SQLSTATE 42601"), "{syntax}");
    assert!(syntax.contains("syntax error"), "{syntax}");
    let parameter = query(&registry, "SELECT $1::int", "param")
        .await
        .unwrap_err();
    assert!(parameter.contains("parameters"), "{parameter}");
    for sql in [
        "SELECT 1; SELECT 2",
        "SELECT 1; COMMIT",
        "ROLLBACK; SELECT 1",
        "COMMIT",
        "BEGIN",
        "SET transaction_read_only = off",
        "EXPLAIN SELECT 1",
        "SHOW server_version",
        "COPY (SELECT 1) TO STDOUT",
    ] {
        let error = query(&registry, sql, "reject").await.unwrap_err();
        assert!(
            error.contains("SQLSTATE") || error.contains("Only one row-returning"),
            "{sql}: {error}"
        );
    }
    let next = query(
        &registry,
        "SELECT current_setting('transaction_read_only')",
        "next",
    )
    .await
    .unwrap();
    assert_eq!(next.rows, vec![vec![json!("on")]]);
    assert!(registry.entries.lock().unwrap().is_empty());
}

#[tokio::test]
#[ignore]
async fn live_writes_functions_ddl_and_transaction_escapes_leave_fixture_unchanged() {
    let fixture = super::super::Connection::connect(&test_config())
        .await
        .unwrap();
    let schema = unique_name("videre_query_test");
    fixture.client.batch_execute(&format!(
        "CREATE SCHEMA {schema}; CREATE TABLE {schema}.items (id integer PRIMARY KEY, value text); INSERT INTO {schema}.items VALUES (1, 'original'); CREATE FUNCTION {schema}.write_item() RETURNS integer LANGUAGE plpgsql AS 'BEGIN UPDATE {schema}.items SET value = ''changed''; RETURN 1; END'; CREATE FUNCTION {schema}.fail_with_details() RETURNS integer LANGUAGE plpgsql AS 'BEGIN RAISE EXCEPTION USING MESSAGE = ''fixture failure'', DETAIL = ''fixture detail'', HINT = ''fixture hint'', ERRCODE = ''P0001''; END';"
    )).await.unwrap();
    let worker_schema = schema.clone();
    let worker = tokio::spawn(async move {
        let registry = QueryRegistry::default();
        let schema = worker_schema;
        for sql in [
            format!("INSERT INTO {schema}.items VALUES (2, 'new') RETURNING *"),
            format!("UPDATE {schema}.items SET value = 'changed' RETURNING *"),
            format!("DELETE FROM {schema}.items RETURNING *"),
            format!("WITH changed AS (UPDATE {schema}.items SET value = 'changed' RETURNING *) SELECT * FROM changed"),
            format!("SELECT {schema}.write_item()"),
            format!("CREATE TABLE {schema}.unexpected (id integer)"),
            format!("DROP TABLE {schema}.items"),
            format!("SELECT * INTO {schema}.unexpected FROM {schema}.items"),
            format!("COMMIT; UPDATE {schema}.items SET value = 'changed'"),
            format!("SELECT 1; COMMIT; DELETE FROM {schema}.items"),
            format!("SELECT * FROM {schema}.items FOR UPDATE"),
        ] {
            let error = query(&registry, &sql, "write").await.unwrap_err();
            assert!(error.contains("SQLSTATE") || error.contains("Only one row-returning"), "{sql}: {error}");
            let data = query(&registry, &format!("SELECT id, value FROM {schema}.items ORDER BY id"), "unchanged").await.unwrap();
            assert_eq!(data.rows, vec![vec![json!("1"), json!("original")]], "{sql}");
        }
        let error = query(
            &registry,
            &format!("SELECT {schema}.fail_with_details()"),
            "details",
        )
        .await
        .unwrap_err();
        for expected in [
            "fixture failure",
            "Detail: fixture detail",
            "Hint: fixture hint",
            "SQLSTATE P0001",
        ] {
            assert!(error.contains(expected), "{error}");
        }
        let unexpected = query(
            &registry,
            &format!("SELECT to_regclass('{schema}.unexpected')"),
            "ddl",
        )
        .await
        .unwrap();
        assert_eq!(unexpected.rows, vec![vec![Value::Null]]);
    });
    // Even an assertion panic in the worker must clean up its committed test
    // schema; editor execution itself never commits any fixture mutations.
    let outcome = worker.await;
    fixture
        .client
        .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    outcome.unwrap();
}

#[tokio::test]
#[ignore]
async fn live_statement_timeout_and_next_query_are_usable() {
    let registry = QueryRegistry::default();
    let started = Instant::now();
    let error = query(&registry, "SELECT pg_sleep(35)", "timeout")
        .await
        .unwrap_err();
    assert!(
        error.contains("SQLSTATE 57014") && error.contains("statement timeout"),
        "{error}"
    );
    assert!(started.elapsed() < Duration::from_secs(40));
    assert!(registry.entries.lock().unwrap().is_empty());
    assert_eq!(
        query(&registry, "SELECT 42", "timeout").await.unwrap().rows,
        vec![vec![json!("42")]]
    );
}

#[tokio::test]
#[ignore]
async fn live_cancellation_interrupts_the_server_and_next_query_is_usable() {
    let registry = Arc::new(QueryRegistry::default());
    let marker = unique_name("videre_cancel");
    let sql = format!("SELECT set_config('application_name', '{marker}', true), pg_sleep(20)");
    let request = registry.register(&sql, "cancel").unwrap();
    let worker =
        tokio::spawn(async move { request.execute(&sql, async { Ok(test_config()) }).await });
    let observer = super::super::Connection::connect(&test_config())
        .await
        .unwrap();
    timeout(Duration::from_secs(5), async {
        loop {
            let running: bool = observer.client.query_one(
                "SELECT EXISTS (SELECT FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND state = 'active' AND application_name = $1)",
                &[&marker],
            ).await.unwrap().get(0);
            if running { break; }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }).await.unwrap();
    timeout(Duration::from_secs(5), registry.cancel("cancel"))
        .await
        .unwrap()
        .unwrap();
    assert_eq!(worker.await.unwrap().unwrap_err(), "Query cancelled");
    timeout(Duration::from_secs(5), async {
        loop {
            // Refresh stats rather than relying on a transaction-cached view.
            let running: bool = observer.client.query_one(
                "SELECT EXISTS (SELECT FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND state = 'active' AND application_name = $1)",
                &[&marker],
            ).await.unwrap().get(0);
            if !running { break; }
            tokio::time::sleep(Duration::from_millis(20)).await;
        }
    }).await.unwrap();
    assert!(registry.entries.lock().unwrap().is_empty());
    assert_eq!(
        query(&registry, "SELECT 7", "cancel").await.unwrap().rows,
        vec![vec![json!("7")]]
    );
}

#[tokio::test]
#[ignore]
async fn live_lock_timeout_preparation_cancellation_and_wall_timeout() {
    let fixture = super::super::Connection::connect(&test_config())
        .await
        .unwrap();
    let schema = unique_name("videre_query_limits");
    fixture.client.batch_execute(&format!(
        "BEGIN; CREATE SCHEMA {schema}; CREATE TABLE {schema}.items (id integer); CREATE FUNCTION {schema}.plan_delay() RETURNS integer LANGUAGE plpgsql IMMUTABLE AS 'BEGIN PERFORM pg_sleep(25); RETURN 1; END'; COMMIT;"
    )).await.unwrap();
    let worker_schema = schema.clone();
    let outcome = tokio::spawn(async move {
        let schema = worker_schema;
        let registry = QueryRegistry::default();
        let blocker = super::super::Connection::connect(&test_config()).await.unwrap();
        blocker.client.batch_execute(&format!("BEGIN; LOCK TABLE {schema}.items IN ACCESS EXCLUSIVE MODE")).await.unwrap();
        let error = query(&registry, &format!("SELECT * FROM {schema}.items"), "lock").await.unwrap_err();
        assert!(error.contains("SQLSTATE 55P03") && error.contains("lock timeout"), "{error}");

        let marker = unique_name("preparing");
        let sql = format!("SELECT * FROM {schema}.items /* {marker} */");
        let request = registry.register(&sql, "prepare").unwrap();
        let preparing = tokio::spawn(async move { request.execute(&sql, async { Ok(test_config()) }).await });
        let observer = super::super::Connection::connect(&test_config()).await.unwrap();
        timeout(Duration::from_secs(3), async {
            loop {
                let waiting: bool = observer.client.query_one(
                    "SELECT EXISTS (SELECT FROM pg_stat_activity WHERE pid <> pg_backend_pid() AND wait_event_type = 'Lock' AND query LIKE $1)",
                    &[&format!("%{marker}%")],
                ).await.unwrap().get(0);
                if waiting { break; }
                tokio::time::sleep(Duration::from_millis(20)).await;
            }
        }).await.unwrap();
        timeout(Duration::from_secs(3), registry.cancel("prepare")).await.unwrap().unwrap();
        assert_eq!(preparing.await.unwrap().unwrap_err(), "Query cancelled");
        blocker.client.execute("ROLLBACK", &[]).await.unwrap();
        assert!(query(&registry, &format!("SELECT * FROM {schema}.items"), "prepare").await.unwrap().rows.is_empty());

        // Planning DECLARE and executing FETCH each stay below statement_timeout,
        // but together exceed the editor's independent whole-request deadline.
        let started = Instant::now();
        let error = query(&registry, &format!("SELECT {schema}.plan_delay(), pg_sleep(25)"), "wall").await.unwrap_err();
        assert!(error.contains("wall-time limit"), "{error}");
        assert!(started.elapsed() < Duration::from_secs(45));
        assert!(registry.entries.lock().unwrap().is_empty());
        assert_eq!(query(&registry, "SELECT 9", "wall").await.unwrap().rows, vec![vec![json!("9")]]);
    }).await;
    fixture
        .client
        .batch_execute(&format!("DROP SCHEMA {schema} CASCADE"))
        .await
        .unwrap();
    outcome.unwrap();
}

#[tokio::test]
#[ignore]
async fn live_connection_change_cancels_isolated_work_without_holding_browse_lock() {
    let state = Arc::new(DbState::new());
    let connection = super::super::Connection::connect(&test_config())
        .await
        .unwrap();
    *state.connection.lock().await = Some(connection);
    *state.config.lock().await = Some(test_config());
    let worker_state = state.clone();
    let worker = tokio::spawn(async move {
        worker_state
            .execute_query("SELECT pg_sleep(20)", "disconnect")
            .await
    });
    tokio::time::sleep(Duration::from_millis(200)).await;
    timeout(Duration::from_secs(1), async {
        let mut connection = state.connection.lock().await;
        let mut config = state.config.lock().await;
        state.queries.cancel_all();
        *connection = None;
        *config = None;
    })
    .await
    .unwrap();
    assert_eq!(
        timeout(Duration::from_secs(5), worker)
            .await
            .unwrap()
            .unwrap()
            .unwrap_err(),
        "Query cancelled"
    );
    assert!(state.queries.entries.lock().unwrap().is_empty());
    assert!(state.connection_info().await.is_err());
}
