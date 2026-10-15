use std::cmp::Ordering as CmpOrdering;
use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicU64, Ordering};

use leptos::{html, prelude::*};
use wasm_bindgen_futures::spawn_local;

use crate::components::data_table::{CellPosition, ColumnSort, DataTable};
use crate::components::relation_drawers::{ColumnTarget, RelationDrawerPanels, RelationDrawers};
use crate::components::sql_editor::SqlEditor;
use crate::stores::db_store::DbStore;
use crate::stores::QuerySession;
use crate::tauri;
use crate::types::{ConnectionInfo, ForeignKeyInfo, ResultColumnSource, SortDirection};

static NEXT_EDITOR: AtomicU64 = AtomicU64::new(1);

type Row = Vec<serde_json::Value>;

/// Plain decimal text (`-12.50`, `9007199254740993`) compared exactly, so
/// bigint and numeric values beyond f64 precision still sort correctly.
#[derive(Debug, PartialEq, Eq)]
struct ExactDecimal<'a> {
    negative: bool,
    /// Without leading zeros.
    integer: &'a str,
    /// Without trailing zeros.
    fraction: &'a str,
}

impl<'a> ExactDecimal<'a> {
    fn parse(text: &'a str) -> Option<Self> {
        let (negative, unsigned) = match text.as_bytes().first()? {
            b'-' => (true, &text[1..]),
            b'+' => (false, &text[1..]),
            _ => (false, text),
        };
        let (integer, fraction) = unsigned.split_once('.').unwrap_or((unsigned, ""));
        let digits = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
        if (integer.is_empty() && fraction.is_empty()) || !digits(integer) || !digits(fraction) {
            return None;
        }
        let integer = integer.trim_start_matches('0');
        let fraction = fraction.trim_end_matches('0');
        Some(Self {
            negative: negative && !(integer.is_empty() && fraction.is_empty()),
            integer,
            fraction,
        })
    }
}

impl Ord for ExactDecimal<'_> {
    fn cmp(&self, other: &Self) -> CmpOrdering {
        let magnitude = || {
            self.integer
                .len()
                .cmp(&other.integer.len())
                .then_with(|| self.integer.cmp(other.integer))
                .then_with(|| self.fraction.cmp(other.fraction))
        };
        match (self.negative, other.negative) {
            (false, true) => CmpOrdering::Greater,
            (true, false) => CmpOrdering::Less,
            (false, false) => magnitude(),
            (true, true) => magnitude().reverse(),
        }
    }
}

impl PartialOrd for ExactDecimal<'_> {
    fn partial_cmp(&self, other: &Self) -> Option<CmpOrdering> {
        Some(self.cmp(other))
    }
}

/// Orders by f64 value, then exactly among plain decimals that f64 cannot tell
/// apart. Rounding is monotonic, so this agrees with exact decimal order while
/// still placing exponent forms, NaN and infinities, and stays a total order.
#[derive(Debug, PartialEq)]
struct NumericKey<'a> {
    value: f64,
    exact: Option<ExactDecimal<'a>>,
}

impl<'a> NumericKey<'a> {
    fn parse(value: &'a serde_json::Value) -> Option<Self> {
        match value {
            serde_json::Value::String(text) => {
                let text = text.trim();
                Some(Self {
                    value: text.parse().ok()?,
                    exact: ExactDecimal::parse(text),
                })
            }
            serde_json::Value::Number(number) => Some(Self {
                value: number.as_f64()?,
                exact: None,
            }),
            _ => None,
        }
    }

    fn cmp(&self, other: &Self) -> CmpOrdering {
        self.value
            .total_cmp(&other.value)
            .then_with(|| self.exact.cmp(&other.exact))
    }
}

/// Display order of the returned preview, as indices into its rows, so the
/// preview is sorted without re-running user SQL. A column whose values all
/// parse as numbers sorts numerically, otherwise by text, so every column has a
/// total order. NULLs sort last ascending, first descending, matching
/// PostgreSQL's defaults. Ties keep the query's own order.
fn sort_order(rows: &[Row], sort: Option<ColumnSort>) -> Vec<usize> {
    let mut order: Vec<usize> = (0..rows.len()).collect();
    let Some(sort) = sort else {
        return order;
    };
    let numeric = rows.iter().all(|row| {
        row.get(sort.column)
            .is_none_or(|value| value.is_null() || NumericKey::parse(value).is_some())
    });
    let text = |value: &serde_json::Value| match value {
        serde_json::Value::String(text) => text.clone(),
        other => other.to_string(),
    };
    let compare = |a: &Row, b: &Row| -> CmpOrdering {
        let (a, b) = (a.get(sort.column), b.get(sort.column));
        let (a, b) = (a.filter(|v| !v.is_null()), b.filter(|v| !v.is_null()));
        match (a, b) {
            (None, None) => CmpOrdering::Equal,
            (None, Some(_)) => CmpOrdering::Greater,
            (Some(_), None) => CmpOrdering::Less,
            (Some(a), Some(b)) if numeric => match (NumericKey::parse(a), NumericKey::parse(b)) {
                (Some(a), Some(b)) => a.cmp(&b),
                _ => CmpOrdering::Equal,
            },
            (Some(a), Some(b)) => text(a).cmp(&text(b)),
        }
    };
    match sort.direction {
        SortDirection::Asc => order.sort_by(|&a, &b| compare(&rows[a], &rows[b])),
        SortDirection::Desc => order.sort_by(|&a, &b| compare(&rows[b], &rows[a])),
    }
    order
}

/// The display row that shows the same record once the order changes.
fn remapped_row(row: usize, from: &[usize], to: &[usize]) -> Option<usize> {
    let record = *from.get(row)?;
    to.iter().position(|&index| index == record)
}

/// FK links for result columns that read a foreign-key column directly.
fn foreign_key_columns(
    sources: &[Option<ResultColumnSource>],
    foreign_keys: &HashMap<String, Vec<ForeignKeyInfo>>,
) -> HashMap<usize, ForeignKeyInfo> {
    sources
        .iter()
        .enumerate()
        .filter_map(|(position, source)| {
            let source = source.as_ref()?;
            let fk = foreign_keys
                .get(&format!("{}.{}", source.schema, source.relation))?
                .iter()
                .find(|fk| fk.column_name == source.column)?;
            Some((position, fk.clone()))
        })
        .collect()
}

fn run_shortcut(
    key: &str,
    is_mac: bool,
    ctrl: bool,
    meta: bool,
    alt: bool,
    shift: bool,
    composing: bool,
) -> bool {
    key == "Enter"
        && !alt
        && !shift
        && !composing
        && if is_mac { meta && !ctrl } else { ctrl && !meta }
}

fn connection_label(connection: &ConnectionInfo) -> String {
    format!(
        "{}@{}:{}/{}",
        connection.username, connection.host, connection.port, connection.database
    )
}

fn publish_connection_info(
    session: &QuerySession,
    mounted: &ArcRwSignal<bool>,
    outcome: Result<ConnectionInfo, String>,
) {
    if !mounted.get_untracked() || session.closed.get_untracked() {
        return;
    }
    match outcome {
        Ok(connection) => {
            session.current_connection.set(Some(connection));
            session.connection_error.set(None);
        }
        Err(error) => {
            session.current_connection.set(None);
            session.connection_error.set(Some(error));
        }
    }
}

#[component]
pub fn QueryPage(session: QuerySession) -> impl IntoView {
    // These arena handles are only used by mounted DOM callbacks. No async task
    // captures them; all asynchronous work uses the session's Arc signals.
    let sql = RwSignal::from(session.sql.clone());
    let result = RwSignal::from(session.result.clone());
    let result_sql = RwSignal::from(session.result_sql.clone());
    let error = RwSignal::from(session.error.clone());
    let running = RwSignal::from(session.running.clone());
    let cancelling = RwSignal::from(session.cancelling.clone());
    let current_connection = RwSignal::from(session.current_connection.clone());
    let connection_error = RwSignal::from(session.connection_error.clone());
    let closed = RwSignal::from(session.closed.clone());

    let mounted = ArcRwSignal::new(true);
    let cleanup_mounted = mounted.clone();
    on_cleanup(move || cleanup_mounted.set(false));
    if !session.closed.get_untracked() {
        let connection_session = session.clone();
        spawn_local(async move {
            // The task may first be polled after this page has already unmounted.
            if !mounted.get_untracked() || connection_session.closed.get_untracked() {
                return;
            }
            let outcome = tauri::get_connection_info().await;
            publish_connection_info(&connection_session, &mounted, outcome);
        });
    }

    let db = use_context::<DbStore>().expect("DbStore not provided");
    let active_cell = RwSignal::new(None::<CellPosition>);
    let sort = RwSignal::new(None::<ColumnSort>);
    // A new execution starts in the order its SQL produced.
    Effect::new(move |previous: Option<()>| {
        result.track();
        if previous.is_some() {
            sort.set(None);
        }
    });
    let order = Memo::new(move |_| {
        let sort = sort.get();
        result.with(|output| {
            output.as_ref().map_or_else(Vec::new, |output| {
                let sort = sort.filter(|sort| sort.column < output.columns.len());
                sort_order(&output.rows, sort)
            })
        })
    });
    // Positions index the display order, so keep the focused record, not its row.
    let on_sort = Callback::new(move |next: Option<ColumnSort>| {
        let before = order.get_untracked();
        sort.set(next);
        let Some(mut cell) = active_cell.get_untracked() else {
            return;
        };
        if let Some(row) = order.with_untracked(|after| remapped_row(cell.row, &before, after)) {
            if row != cell.row {
                cell.row = row;
                active_cell.set(Some(cell));
            }
        }
    });
    let page_ref = NodeRef::<html::Section>::new();
    let drawers = RelationDrawers::new();
    let drawer_root = Callback::new(move |()| page_ref.get_untracked().map(Into::into));
    let on_fk_click = Callback::new(move |(fk, value): (ForeignKeyInfo, serde_json::Value)| {
        drawers.open_foreign_key(fk, value);
    });
    let keyboard_enabled =
        Signal::derive(move || !closed.get() && !running.get() && !drawers.is_open());
    let editor_id = format!("query-sql-{}", NEXT_EDITOR.fetch_add(1, Ordering::Relaxed));
    let is_mac = web_sys::window()
        .and_then(|window| window.navigator().platform().ok())
        .is_some_and(|platform| platform.contains("Mac"));
    let shortcut_label = if is_mac { "Cmd+Enter" } else { "Ctrl+Enter" };

    let keyboard_session = session.clone();
    let on_keydown = move |event: web_sys::KeyboardEvent| {
        if !event.default_prevented()
            && !event.repeat()
            && !keyboard_session.closed.get_untracked()
            && run_shortcut(
                &event.key(),
                is_mac,
                event.ctrl_key(),
                event.meta_key(),
                event.alt_key(),
                event.shift_key(),
                event.is_composing(),
            )
        {
            event.prevent_default();
            keyboard_session.run();
        }
    };
    let run_session = session.clone();
    let cancel_session = session;
    let on_cancel = Callback::new(move |()| cancel_session.cancel());

    view! {
        <section class="query-page" aria-label="SQL query" node_ref=page_ref on:keydown=on_keydown>
            <header class="query-page-header">
                <div class="query-heading">
                    <h2>"Query"</h2>
                    <span class="query-read-only">"Read only"</span>
                    <span class="query-connection" aria-live="polite">
                        {move || {
                            if let Some(connection) = current_connection.get() {
                                connection_label(&connection)
                            } else if let Some(message) = connection_error.get() {
                                format!("Connection unavailable: {message}")
                            } else {
                                "Loading connection…".into()
                            }
                        }}
                    </span>
                </div>
                <div class="query-actions">
                    <button
                        type="button"
                        class="btn btn-primary btn-sm"
                        title=format!("Run entire query ({shortcut_label})")
                        disabled=move || closed.get() || running.get() || sql.with(|sql| sql.trim().is_empty())
                        on:click=move |_| run_session.run()
                    >
                        "Run"
                    </button>
                    <Show when=move || running.get()>
                        <button
                            type="button"
                            class="btn btn-ghost btn-sm"
                            disabled=move || closed.get() || cancelling.get()
                            on:click=move |_| on_cancel.run(())
                        >
                            {move || if cancelling.get() { "Cancelling…" } else { "Cancel" }}
                        </button>
                    </Show>
                </div>
            </header>

            <div class="query-editor">
                <label class="query-editor-label" for=editor_id.clone()>
                    {format!("SQL — run entire query with {shortcut_label}")}
                </label>
                <SqlEditor id=editor_id sql=sql disabled=Signal::derive(move || closed.get()) />
            </div>

            <p class="query-status" role="status" aria-live="polite">
                {move || {
                    if closed.get() {
                        "Query tab closed."
                    } else if cancelling.get() {
                        "Cancellation requested; waiting for execution to stop…"
                    } else if running.get() {
                        "Running read-only query…"
                    } else if error.get().is_some() {
                        "Query did not complete successfully."
                    } else if result.get().is_some() {
                        "Execution complete."
                    } else {
                        "Single SELECT, WITH, or VALUES query. Preview limited to 1,000 rows."
                    }
                }}
            </p>
            {move || error.get().map(|message| view! {
                <pre class="query-error" role="alert">{message}</pre>
            })}

            {move || {
                // Copies only what the view keeps; rows are cloned once, in display order.
                let (columns, sources, rows, summary, truncated, row_limit) = result.with(|output| {
                    let output = output.as_ref()?;
                    let summary = format!(
                        "{} rows returned · {} ms · Executed on {}",
                        output.rows.len(),
                        output.elapsed_ms,
                        connection_label(&output.connection)
                    );
                    let rows: Vec<Row> = order.with(|order| {
                        order.iter().map(|&index| output.rows[index].clone()).collect()
                    });
                    Some((
                        output.columns.clone(),
                        output.column_sources.clone(),
                        rows,
                        summary,
                        output.truncated,
                        output.row_limit,
                    ))
                })?;
                let original_sql = result_sql.get_untracked().unwrap_or_default();
                let active_sort = sort.get().filter(|sort| sort.column < columns.len());
                // Tracked: links appear once metadata loads, including after a reconnect.
                let fk_columns = db
                    .foreign_keys
                    .with(|foreign_keys| foreign_key_columns(&sources, foreign_keys));
                let info_columns: HashSet<usize> = sources
                    .iter()
                    .enumerate()
                    .filter_map(|(position, source)| source.as_ref().map(|_| position))
                    .collect();
                let on_column_info = Callback::new(move |position: usize| {
                    if let Some(Some(source)) = sources.get(position) {
                        drawers.open_column(ColumnTarget {
                            position,
                            schema: source.schema.clone(),
                            relation: source.relation.clone(),
                            column: source.column.clone(),
                        });
                    }
                });
                Some(view! {
                    <section class="query-results" aria-label="Query results">
                        <div class="query-results-summary">
                            <span>{summary}</span>
                            <Show when=move || sql.with(|draft| draft != &original_sql)>
                                <span>"SQL has changed; these results are from the previous execution."</span>
                            </Show>
                        </div>
                        {truncated.then(|| view! {
                            <p class="query-truncated" role="status">
                                {format!("Preview truncated at {row_limit} rows. Additional rows were not returned.")}
                                {active_sort.map(|_| " Sorting applies to the returned preview only.")}
                            </p>
                        })}
                        <DataTable
                            columns=columns
                            rows=rows
                            active_cell=active_cell
                            keyboard_enabled=keyboard_enabled
                            sort=Signal::from(sort)
                            on_sort=on_sort
                            fk_columns=fk_columns
                            fk_click=on_fk_click
                            column_info_click=on_column_info
                            column_info_columns=info_columns
                        />
                    </section>
                })
            }}
            <RelationDrawerPanels drawers=drawers root=drawer_root active_cell=active_cell />
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::{json, Value};

    fn column(rows: &[Row], index: usize) -> Vec<Value> {
        rows.iter().map(|row| row[index].clone()).collect()
    }

    fn sorted_rows(rows: Vec<Row>, sort: Option<ColumnSort>) -> Vec<Row> {
        sort_order(&rows, sort)
            .into_iter()
            .map(|index| rows[index].clone())
            .collect()
    }

    #[test]
    fn focused_row_follows_its_record_across_orders() {
        let unsorted = [0, 1, 2, 3];
        let sorted = [2, 0, 3, 1];
        assert_eq!(remapped_row(0, &unsorted, &sorted), Some(1));
        assert_eq!(remapped_row(1, &sorted, &unsorted), Some(0));
        assert_eq!(remapped_row(3, &sorted, &sorted), Some(3));
        assert_eq!(remapped_row(4, &unsorted, &sorted), None);
    }

    #[test]
    fn numeric_text_columns_sort_by_value_with_postgres_null_placement() {
        let rows: Vec<Row> = ["10", "9", "-1.5", "1e3"]
            .into_iter()
            .map(|n| vec![json!(n)])
            .chain([vec![Value::Null]])
            .collect();
        let ascending = sorted_rows(
            rows.clone(),
            Some(ColumnSort {
                column: 0,
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(
            column(&ascending, 0),
            [
                json!("-1.5"),
                json!("9"),
                json!("10"),
                json!("1e3"),
                Value::Null
            ]
        );
        let descending = sorted_rows(
            rows.clone(),
            Some(ColumnSort {
                column: 0,
                direction: SortDirection::Desc,
            }),
        );
        assert_eq!(
            column(&descending, 0),
            [
                Value::Null,
                json!("1e3"),
                json!("10"),
                json!("9"),
                json!("-1.5")
            ]
        );
        assert_eq!(sorted_rows(rows.clone(), None), rows);
    }

    #[test]
    fn numeric_columns_sort_exactly_beyond_f64_precision() {
        let values = [
            "9007199254740993",
            "9007199254740992",
            "-9007199254740993",
            "-9007199254740992",
            "12345678901234567.25",
            "12345678901234567.2",
            "0.10",
            "-0",
            "1e2",
            "100.5",
        ];
        let rows: Vec<Row> = values.into_iter().map(|n| vec![json!(n)]).collect();
        let ascending = sorted_rows(
            rows,
            Some(ColumnSort {
                column: 0,
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(
            column(&ascending, 0),
            [
                json!("-9007199254740993"),
                json!("-9007199254740992"),
                json!("-0"),
                json!("0.10"),
                json!("1e2"),
                json!("100.5"),
                json!("9007199254740992"),
                json!("9007199254740993"),
                json!("12345678901234567.2"),
                json!("12345678901234567.25"),
            ]
        );
    }

    #[test]
    fn mixed_columns_sort_as_text_and_ties_keep_query_order() {
        let rows: Vec<Row> = vec![
            vec![json!("b"), json!(1)],
            vec![json!("10"), json!(2)],
            vec![json!("b"), json!(3)],
            vec![json!("9"), json!(4)],
        ];
        let sorted = sorted_rows(
            rows,
            Some(ColumnSort {
                column: 0,
                direction: SortDirection::Asc,
            }),
        );
        assert_eq!(
            column(&sorted, 0),
            [json!("10"), json!("9"), json!("b"), json!("b")]
        );
        assert_eq!(column(&sorted, 1), [json!(2), json!(4), json!(1), json!(3)]);
    }

    #[test]
    fn only_direct_foreign_key_columns_link_by_result_position() {
        let source = |relation: &str, column: &str| {
            Some(ResultColumnSource {
                schema: "public".into(),
                relation: relation.into(),
                column: column.into(),
            })
        };
        let owner_fk = ForeignKeyInfo {
            column_name: "owner_id".into(),
            foreign_table_schema: "public".into(),
            foreign_table_name: "owners".into(),
            foreign_column_name: "id".into(),
        };
        let foreign_keys =
            HashMap::from([("public.creatures".to_string(), vec![owner_fk.clone()])]);
        let links = foreign_key_columns(
            &[
                source("creatures", "id"),
                None,
                source("creatures", "owner_id"),
                source("owners", "owner_id"),
                source("creatures", "owner_id"),
            ],
            &foreign_keys,
        );
        assert_eq!(
            links.keys().copied().collect::<HashSet<_>>(),
            HashSet::from([2, 4])
        );
        assert_eq!(links[&2].foreign_table_name, "owners");
    }

    #[test]
    fn run_shortcut_uses_only_the_platform_command_modifier() {
        assert!(run_shortcut(
            "Enter", true, false, true, false, false, false
        ));
        assert!(run_shortcut(
            "Enter", false, true, false, false, false, false
        ));
        assert!(!run_shortcut(
            "Enter", true, true, false, false, false, false
        ));
        assert!(!run_shortcut(
            "Enter", false, false, true, false, false, false
        ));
        for is_mac in [false, true] {
            assert!(!run_shortcut(
                "Enter", is_mac, true, true, false, false, false
            ));
            assert!(!run_shortcut(
                "Enter", is_mac, false, false, false, false, false
            ));
        }
    }

    #[test]
    fn extra_modifiers_composition_and_native_editor_keys_do_not_run() {
        for is_mac in [false, true] {
            let (ctrl, meta) = (!is_mac, is_mac);
            assert!(!run_shortcut(
                "Enter", is_mac, ctrl, meta, true, false, false
            ));
            assert!(!run_shortcut(
                "Enter", is_mac, ctrl, meta, false, true, false
            ));
            assert!(!run_shortcut(
                "Enter", is_mac, ctrl, meta, false, false, true
            ));
            for key in [
                "Tab",
                "ArrowLeft",
                "ArrowRight",
                "ArrowUp",
                "ArrowDown",
                "j",
                "k",
            ] {
                assert!(!run_shortcut(
                    key, is_mac, false, false, false, false, false
                ));
            }
        }
    }

    #[test]
    fn connection_completion_is_ignored_after_unmount_or_close() {
        let session = QuerySession::new();
        let mounted = ArcRwSignal::new(false);
        let connection = ConnectionInfo {
            host: "localhost".into(),
            port: "5432".into(),
            database: "original".into(),
            username: "reader".into(),
        };
        publish_connection_info(&session, &mounted, Ok(connection.clone()));
        assert!(session.current_connection.get_untracked().is_none());
        mounted.set(true);
        session.close();
        publish_connection_info(&session, &mounted, Ok(connection));
        publish_connection_info(&session, &mounted, Err("late error".into()));
        assert!(session.current_connection.get_untracked().is_none());
        assert!(session.connection_error.get_untracked().is_none());
    }

    #[test]
    fn connection_identity_labels_original_database_without_credentials() {
        assert_eq!(
            connection_label(&ConnectionInfo {
                host: "db.example".into(),
                port: "5433".into(),
                database: "original".into(),
                username: "reader".into(),
            }),
            "reader@db.example:5433/original"
        );
    }
}
