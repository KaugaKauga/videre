use std::sync::atomic::{AtomicU64, Ordering};

use leptos::prelude::*;
use wasm_bindgen_futures::spawn_local;

use crate::components::data_table::{CellPosition, DataTable};
use crate::stores::QuerySession;
use crate::tauri;
use crate::types::{ConnectionInfo, SortSpec};

static NEXT_EDITOR: AtomicU64 = AtomicU64::new(1);

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

    let active_cell = RwSignal::new(None::<CellPosition>);
    let sort = RwSignal::new(None::<SortSpec>);
    let keyboard_enabled = Signal::derive(move || !closed.get() && !running.get());
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
        <section class="query-page" aria-label="SQL query" on:keydown=on_keydown>
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
                <textarea
                    id=editor_id
                    class="query-sql-input"

                    rows="8"
                    spellcheck="false"
                    autocomplete="off"
                    autocapitalize="off"
                    disabled=move || closed.get()
                    prop:value=move || sql.get()
                    on:input=move |event| {
                        if !closed.get_untracked() {
                            sql.set(event_target::<web_sys::HtmlTextAreaElement>(&event).value());
                        }
                    }
                ></textarea>
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
                let output = result.get()?;
                let original_sql = result_sql.get_untracked().unwrap_or_default();
                let connection = connection_label(&output.connection);
                let summary = format!(
                    "{} rows returned · {} ms · Executed on {}",
                    output.rows.len(), output.elapsed_ms, connection
                );
                let truncated = output.truncated;
                let row_limit = output.row_limit;
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
                            </p>
                        })}
                        <DataTable
                            columns=output.columns
                            rows=output.rows
                            active_cell=active_cell
                            keyboard_enabled=keyboard_enabled
                            sort=sort
                            on_sort=Callback::new(|_| {})
                            sortable=false
                        />
                    </section>
                })
            }}
        </section>
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
