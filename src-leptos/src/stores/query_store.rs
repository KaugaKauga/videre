use std::sync::atomic::{AtomicU64, Ordering};

use leptos::prelude::*;
use wasm_bindgen::JsValue;
use wasm_bindgen_futures::{spawn_local, JsFuture};

use crate::tauri;
use crate::types::{ConnectionInfo, QueryResult};

static NEXT_REQUEST: AtomicU64 = AtomicU64::new(1);
const CANCEL_RETRY_DELAYS_MS: [i32; 6] = [25, 50, 100, 200, 400, 800];

#[derive(Clone, Debug)]
struct PendingQuery {
    id: String,
    sql: String,
    cancel_requested: bool,
}

/// Tab-owned state. Async tasks hold only Arc signals, never an arena signal or owner.
#[derive(Clone, Debug)]
pub struct QuerySession {
    pub sql: ArcRwSignal<String>,
    pub result: ArcRwSignal<Option<QueryResult>>,
    pub result_sql: ArcRwSignal<Option<String>>,
    pub error: ArcRwSignal<Option<String>>,
    pub running: ArcRwSignal<bool>,
    pub cancelling: ArcRwSignal<bool>,
    pub current_connection: ArcRwSignal<Option<ConnectionInfo>>,
    pub connection_error: ArcRwSignal<Option<String>>,
    pub closed: ArcRwSignal<bool>,
    pending: ArcRwSignal<Option<PendingQuery>>,
}

impl Default for QuerySession {
    fn default() -> Self {
        Self::new()
    }
}

impl QuerySession {
    pub fn new() -> Self {
        Self {
            sql: ArcRwSignal::new(String::new()),
            result: ArcRwSignal::new(None),
            result_sql: ArcRwSignal::new(None),
            error: ArcRwSignal::new(None),
            running: ArcRwSignal::new(false),
            cancelling: ArcRwSignal::new(false),
            current_connection: ArcRwSignal::new(None),
            connection_error: ArcRwSignal::new(None),
            closed: ArcRwSignal::new(false),
            pending: ArcRwSignal::new(None),
        }
    }

    /// Runs the entire current draft. A session has at most one outstanding execution.
    pub fn run(&self) {
        if self.closed.get_untracked() || self.running.get_untracked() {
            return;
        }
        let id = format!(
            "videre-query-{}-{}",
            js_sys::Date::now() as u64,
            NEXT_REQUEST.fetch_add(1, Ordering::Relaxed)
        );
        let Some(sql) = self.begin_request(id.clone()) else {
            return;
        };
        let session = self.clone();
        // This is deliberately not an owner-scoped Leptos task: switching tabs
        // must not drop the IPC future, including its completion bookkeeping.
        spawn_local(async move {
            let outcome = tauri::execute_query(&sql, &id).await;
            session.finish_request(&id, outcome);
        });
    }

    pub fn cancel(&self) {
        if !self.closed.get_untracked() {
            self.start_cancellation();
        }
    }

    /// Closing is terminal; remounts and late completions cannot revive the session.
    /// Switching tabs must not call this method.
    pub fn close(&self) {
        if self.closed.get_untracked() {
            return;
        }
        self.closed.set(true);
        self.start_cancellation();
    }

    fn begin_request(&self, id: String) -> Option<String> {
        if self.closed.get_untracked() || self.running.get_untracked() {
            return None;
        }
        let sql = self.sql.get_untracked();
        if sql.trim().is_empty() {
            self.error
                .set(Some("Enter a SELECT, WITH, or VALUES query to run.".into()));
            return None;
        }
        self.pending.set(Some(PendingQuery {
            id,
            sql: sql.clone(),
            cancel_requested: false,
        }));
        // Old output must not look like the output of a new run that fails.
        self.result.set(None);
        self.result_sql.set(None);
        self.error.set(None);
        self.cancelling.set(false);
        self.running.set(true);
        Some(sql)
    }

    fn matches_request(&self, id: &str) -> bool {
        self.pending
            .with_untracked(|pending| pending.as_ref().is_some_and(|request| request.id == id))
    }

    fn finish_request(&self, id: &str, outcome: Result<QueryResult, String>) {
        if !self.matches_request(id) {
            return;
        }
        let request = self.pending.get_untracked().expect("matching request");
        self.pending.set(None);
        if !self.closed.get_untracked() {
            match outcome {
                Ok(_) if request.cancel_requested => {
                    self.error.set(Some(
                        "Query cancelled; the result was discarded after cancellation was requested."
                            .into(),
                    ));
                }
                Ok(result) => {
                    self.result_sql.set(Some(request.sql));
                    self.result.set(Some(result));
                    self.error.set(None);
                }
                Err(error) => self.error.set(Some(error)),
            }
        }
        // Only execute_query settling ends the run, not a cancellation acknowledgement.
        self.running.set(false);
        self.cancelling.set(false);
    }

    fn prepare_cancellation(&self) -> Option<String> {
        if self.cancelling.get_untracked() {
            return None;
        }
        let mut id = None;
        self.pending.update(|pending| {
            if let Some(request) = pending {
                request.cancel_requested = true;
                id = Some(request.id.clone());
            }
        });
        if id.is_some() {
            self.cancelling.set(true);
        }
        id
    }

    fn start_cancellation(&self) {
        let Some(id) = self.prepare_cancellation() else {
            return;
        };
        let session = self.clone();
        spawn_local(async move {
            // A closed session still needs to cancel: do not guard this loop with closed.
            for attempt in 0..=CANCEL_RETRY_DELAYS_MS.len() {
                if !session.matches_request(&id) {
                    return;
                }
                match tauri::cancel_query(&id).await {
                    Ok(()) => return,
                    Err(error) => {
                        if !session.matches_request(&id) {
                            return;
                        }
                        let Some(delay) = cancellation_retry_delay(&error, attempt) else {
                            session.cancellation_failed(&id, error);
                            return;
                        };
                        if let Err(error) = delay_ms(delay).await {
                            session.cancellation_failed(&id, error);
                            return;
                        }
                    }
                }
            }
        });
    }

    fn cancellation_failed(&self, id: &str, error: String) {
        if !self.matches_request(id) {
            return;
        }
        if !self.closed.get_untracked() {
            self.error.set(Some(format!(
                "Could not cancel query: {error}\nExecution is still pending; retry Cancel."
            )));
        }
        self.cancelling.set(false);
    }
}

fn cancellation_retry_delay(error: &str, attempt: usize) -> Option<i32> {
    if error.contains("not registered; retry if execution is still starting") {
        CANCEL_RETRY_DELAYS_MS.get(attempt).copied()
    } else {
        None
    }
}

async fn delay_ms(milliseconds: i32) -> Result<(), String> {
    let window = web_sys::window().ok_or("No window available for cancellation retry")?;
    let promise = js_sys::Promise::new(&mut |resolve, reject| {
        if let Err(error) =
            window.set_timeout_with_callback_and_timeout_and_arguments_0(&resolve, milliseconds)
        {
            let _ = reject.call1(&JsValue::UNDEFINED, &error);
        }
    });
    JsFuture::from(promise).await.map(|_| ()).map_err(|error| {
        error
            .as_string()
            .unwrap_or_else(|| "Could not schedule cancellation retry".into())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn result() -> QueryResult {
        QueryResult {
            columns: vec!["answer".into()],
            rows: vec![vec![serde_json::json!(42)]],
            truncated: false,
            row_limit: 1000,
            elapsed_ms: 12,
            connection: ConnectionInfo {
                host: "localhost".into(),
                port: "5432".into(),
                database: "original_database".into(),
                username: "reader".into(),
            },
        }
    }

    #[test]
    fn cloned_session_and_background_completion_survive_owner_disposal() {
        let owner = Owner::new();
        let session = owner.with(QuerySession::new);
        let remount = session.clone();
        owner.with(|| {
            session.sql.set("SELECT 42".into());
            assert_eq!(
                session.begin_request("first".into()),
                Some("SELECT 42".into())
            );
        });
        owner.cleanup();
        assert!(remount.running.get_untracked());
        remount.sql.set("SELECT 43".into());
        session.finish_request("first", Ok(result()));
        assert_eq!(remount.sql.get_untracked(), "SELECT 43");
        assert_eq!(
            remount.result_sql.get_untracked().as_deref(),
            Some("SELECT 42")
        );
        assert_eq!(remount.result.get_untracked(), Some(result()));
        assert!(!remount.running.get_untracked());
    }

    #[test]
    fn result_connection_does_not_replace_live_connection_identity() {
        let session = QuerySession::new();
        let mut current = result().connection;
        current.database = "current_database".into();
        session.current_connection.set(Some(current.clone()));
        session.sql.set("SELECT 42".into());
        session.begin_request("first".into()).unwrap();
        session.finish_request("first", Ok(result()));
        assert_eq!(session.current_connection.get_untracked(), Some(current));
        assert_eq!(
            session.result.get_untracked().unwrap().connection.database,
            "original_database"
        );
    }

    #[test]
    fn sessions_are_independent_and_duplicate_runs_are_rejected() {
        let first = QuerySession::new();
        let second = QuerySession::new();
        first.sql.set("SELECT 1".into());
        second.sql.set("VALUES (2)".into());
        first.begin_request("first".into()).unwrap();
        assert!(first.begin_request("duplicate".into()).is_none());
        assert!(!second.running.get_untracked());
        assert_eq!(second.sql.get_untracked(), "VALUES (2)");
        assert!(first.matches_request("first"));
    }

    #[test]
    fn cancellation_waits_for_execution_and_discards_late_success() {
        let session = QuerySession::new();
        session.sql.set("SELECT 42".into());
        session.begin_request("first".into()).unwrap();
        assert_eq!(session.prepare_cancellation().as_deref(), Some("first"));
        assert!(session.prepare_cancellation().is_none());
        assert!(session.running.get_untracked());
        assert!(session.cancelling.get_untracked());
        assert!(session.error.get_untracked().is_none());
        session.finish_request("first", Ok(result()));
        assert!(!session.running.get_untracked());
        assert!(!session.cancelling.get_untracked());
        assert!(session.result.get_untracked().is_none());
        assert!(session.error.get_untracked().unwrap().contains("cancelled"));
    }

    #[test]
    fn cancellation_failure_allows_retry_without_ending_the_run() {
        let session = QuerySession::new();
        session.sql.set("SELECT 42".into());
        session.begin_request("first".into()).unwrap();
        session.prepare_cancellation().unwrap();
        session.cancellation_failed("first", "transport unavailable".into());
        assert!(session.running.get_untracked());
        assert!(!session.cancelling.get_untracked());
        assert_eq!(session.prepare_cancellation().as_deref(), Some("first"));
        session.finish_request("first", Err("Query cancelled".into()));
        assert_eq!(
            session.error.get_untracked().as_deref(),
            Some("Query cancelled")
        );
    }

    #[test]
    fn stale_completion_and_cancel_failure_cannot_touch_a_new_run() {
        let session = QuerySession::new();
        session.sql.set("SELECT 42".into());
        session.begin_request("first".into()).unwrap();
        session.finish_request("first", Err("first failed".into()));
        session.begin_request("second".into()).unwrap();
        session.finish_request("first", Ok(result()));
        session.cancellation_failed("first", "late error".into());
        assert!(session.running.get_untracked());
        assert!(session.error.get_untracked().is_none());
        assert!(session.result.get_untracked().is_none());
        assert!(session.matches_request("second"));
    }

    #[test]
    fn close_is_terminal_and_late_completion_does_not_publish() {
        let session = QuerySession::new();
        session.sql.set("SELECT 42".into());
        session.begin_request("first".into()).unwrap();
        // Prepare without spawning a browser task in this native test.
        session.prepare_cancellation().unwrap();
        session.close();
        session.close();
        assert!(session.closed.get_untracked());
        assert!(session.running.get_untracked());
        assert!(session.begin_request("after-close".into()).is_none());
        session.run();
        session.cancel();
        session.finish_request("first", Ok(result()));
        assert!(!session.running.get_untracked());
        assert!(session.result.get_untracked().is_none());
        assert!(session.current_connection.get_untracked().is_none());
        assert!(session.error.get_untracked().is_none());
    }

    #[test]
    fn rerun_clears_old_results_and_preserves_draft_on_error() {
        let session = QuerySession::new();
        session.sql.set("SELECT 42".into());
        session.begin_request("first".into()).unwrap();
        session.finish_request("first", Ok(result()));
        session.sql.set("SELECT broken".into());
        session.begin_request("second".into()).unwrap();
        assert!(session.result.get_untracked().is_none());
        assert!(session.result_sql.get_untracked().is_none());
        session.finish_request("second", Err("column does not exist".into()));
        assert_eq!(session.sql.get_untracked(), "SELECT broken");
        assert!(session.result.get_untracked().is_none());
    }

    #[test]
    fn registration_race_retries_are_bounded_and_other_errors_are_not_retried() {
        let race = "Query request ID 'x' is not registered; retry if execution is still starting";
        let delays: Vec<_> = (0..10)
            .filter_map(|attempt| cancellation_retry_delay(race, attempt))
            .collect();
        assert_eq!(delays, CANCEL_RETRY_DELAYS_MS);
        assert_eq!(delays.iter().sum::<i32>(), 1575);
        assert_eq!(cancellation_retry_delay("permission denied", 0), None);
        assert_eq!(
            cancellation_retry_delay(race, CANCEL_RETRY_DELAYS_MS.len()),
            None
        );
    }
}
