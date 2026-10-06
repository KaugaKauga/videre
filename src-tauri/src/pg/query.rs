use std::collections::HashMap;
use std::future::Future;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use serde_json::Value;
use tokio::sync::watch;
use tokio::task::JoinHandle;
use tokio::time::{timeout, Instant};
use tokio_postgres::{Client, Error, NoTls, SimpleQueryMessage};

use crate::types::{ConnectionConfig, ConnectionInfo, QueryResult};

const ROW_LIMIT: usize = 1000;
const FETCH_BATCH_SIZE: usize = 16;
const MAX_ROW_BYTES: usize = 1024 * 1024;
const MAX_PREVIEW_BYTES: usize = 8 * 1024 * 1024;
const MAX_COLUMNS: usize = 1024;
const MAX_METADATA_BYTES: usize = 64 * 1024;
const ROW_BUDGET_ERROR: &str = "Query row exceeds the 1 MiB preview row budget. Select narrower columns or smaller values and retry.";
const PREVIEW_BUDGET_ERROR: &str = "Query result exceeds the 8 MiB preview payload budget. Select narrower columns or smaller values, or return fewer rows, and retry.";
const MAX_SQL_BYTES: usize = 1024 * 1024;
const MAX_REQUEST_ID_BYTES: usize = 256;
const MAX_RUNNING_QUERIES: usize = 4;
const WALL_TIMEOUT: Duration = Duration::from_secs(40);
const CLEANUP_TIMEOUT: Duration = Duration::from_secs(2);

struct Signal {
    cancelled: watch::Sender<bool>,
    finished: watch::Sender<bool>,
}

/// Driver-free application seam. Registration precedes even the active-config
/// lookup, so cancellation also works while waiting for browsing or connecting.
#[derive(Default)]
pub struct QueryRegistry {
    entries: Arc<Mutex<HashMap<String, Arc<Signal>>>>,
}

pub struct QueryRequest {
    id: String,
    signal: Arc<Signal>,
    entries: Arc<Mutex<HashMap<String, Arc<Signal>>>>,
}

impl QueryRegistry {
    pub fn register(&self, sql: &str, request_id: &str) -> Result<QueryRequest, String> {
        validate_input(sql, request_id)?;
        let mut entries = self.entries.lock().unwrap();
        if entries.contains_key(request_id) {
            return Err(format!(
                "Query request ID '{request_id}' is already registered"
            ));
        }
        if entries.len() >= MAX_RUNNING_QUERIES {
            return Err(format!(
                "At most {MAX_RUNNING_QUERIES} queries may run concurrently"
            ));
        }
        let signal = Arc::new(Signal {
            cancelled: watch::channel(false).0,
            finished: watch::channel(false).0,
        });
        entries.insert(request_id.to_owned(), signal.clone());
        Ok(QueryRequest {
            id: request_id.to_owned(),
            signal,
            entries: self.entries.clone(),
        })
    }

    pub async fn cancel(&self, request_id: &str) -> Result<(), String> {
        let signal = self
            .entries
            .lock()
            .unwrap()
            .get(request_id)
            .cloned()
            .ok_or_else(|| format!("Query request ID '{request_id}' is not registered; retry if execution is still starting"))?;
        let mut finished = signal.finished.subscribe();
        signal.cancelled.send_replace(true);
        // A successful cancellation acknowledgement means the isolated driver
        // has been dropped, not just that the UI can forget its result.
        finished
            .wait_for(|done| *done)
            .await
            .map_err(|_| "Query cancellation acknowledgement was lost".to_string())?;
        Ok(())
    }

    pub fn cancel_all(&self) {
        for signal in self.entries.lock().unwrap().values() {
            signal.cancelled.send_replace(true);
        }
        // Keep slots occupied until their drivers actually stop. Eager removal
        // would allow reconnects to bypass the concurrency limit.
    }
}

impl Drop for QueryRequest {
    fn drop(&mut self) {
        let mut entries = self.entries.lock().unwrap();
        if entries
            .get(&self.id)
            .is_some_and(|signal| Arc::ptr_eq(signal, &self.signal))
        {
            entries.remove(&self.id);
        }
        self.signal.finished.send_replace(true);
    }
}

impl QueryRequest {
    pub async fn execute(
        self,
        sql: &str,
        config: impl Future<Output = Result<ConnectionConfig, String>>,
    ) -> Result<QueryResult, String> {
        let started = Instant::now();
        let deadline = started + WALL_TIMEOUT;
        let mut cancelled = self.signal.cancelled.subscribe();
        let connect = async {
            let config = config.await?;
            let connection = QueryConnection::connect(&config).await?;
            Ok::<_, String>((config, connection))
        };
        let (config, connection) = tokio::select! {
            biased;
            _ = wait_for_cancellation(&mut cancelled) => return Err("Query cancelled".into()),
            _ = tokio::time::sleep_until(deadline) => return Err("Query exceeded the 40-second wall-time limit".into()),
            result = connect => result?,
        };

        let result = tokio::select! {
            biased;
            _ = wait_for_cancellation(&mut cancelled) => {
                connection.cancel().await;
                Err("Query cancelled".into())
            },
            _ = tokio::time::sleep_until(deadline) => {
                connection.cancel().await;
                Err("Query exceeded the 40-second wall-time limit".into())
            },
            result = connection.preview(sql) => result,
        };
        // Rollback is best effort on errors; dropping the isolated socket is
        // unconditional and also rolls back an aborted or disconnected session.
        let _ = timeout(CLEANUP_TIMEOUT, connection.client.execute("ROLLBACK", &[])).await;
        connection.close().await;
        result.map(|(columns, rows, truncated)| QueryResult {
            columns,
            rows,
            truncated,
            row_limit: ROW_LIMIT,
            elapsed_ms: started.elapsed().as_millis().min(u64::MAX as u128) as u64,
            connection: ConnectionInfo::from(&config),
        })
    }
}

struct QueryConnection {
    client: Client,
    driver: JoinHandle<Result<(), Error>>,
}

impl Drop for QueryConnection {
    fn drop(&mut self) {
        // Dropping a tokio-postgres Client alone can leave an executing driver
        // alive. Abort it as well so cancellation/timeout cannot leak a session.
        self.driver.abort();
    }
}

impl QueryConnection {
    async fn connect(config: &ConnectionConfig) -> Result<Self, String> {
        let port = config.port.parse::<u16>().map_err(|_| {
            "Invalid PostgreSQL port: expected a number from 0 to 65535".to_string()
        })?;
        let mut driver_config = tokio_postgres::Config::new();
        driver_config
            .host(&config.host)
            .port(port)
            .dbname(&config.database)
            .user(&config.username)
            .password(&config.password)
            .connect_timeout(Duration::from_secs(5));
        let (client, connection) = driver_config
            .connect(NoTls)
            .await
            .map_err(|error| pg_error("Query connection failed", &error))?;
        Ok(Self {
            client,
            driver: tokio::spawn(connection),
        })
    }

    async fn close(mut self) {
        self.driver.abort();
        let _ = (&mut self.driver).await;
    }

    async fn cancel(&self) {
        // Send PostgreSQL's out-of-band cancellation packet before closing the
        // driver. The timeout bounds an unreachable cancellation endpoint.
        let _ = timeout(
            CLEANUP_TIMEOUT,
            self.client.cancel_token().cancel_query(NoTls),
        )
        .await;
        self.driver.abort();
    }

    async fn preview(&self, sql: &str) -> Result<(Vec<String>, Vec<Vec<Value>>, bool), String> {
        self.client
            .execute("BEGIN READ ONLY", &[])
            .await
            .map_err(|error| pg_error("Could not start read-only query transaction", &error))?;
        for setting in [
            "SET LOCAL statement_timeout = '30s'",
            "SET LOCAL lock_timeout = '5s'",
        ] {
            self.client
                .execute(setting, &[])
                .await
                .map_err(|error| pg_error("Could not set query timeout", &error))?;
        }

        // Parse through the extended protocol BEFORE embedding the unchanged
        // SQL in DECLARE. PostgreSQL enforces exactly one statement here.
        let statement = self
            .client
            .prepare(sql)
            .await
            .map_err(|error| pg_error("Could not prepare query", &error))?;
        if !statement.params().is_empty() {
            return Err(
                "Query parameters ($1, $2, ...) are not supported in the SQL editor".into(),
            );
        }
        if statement.columns().is_empty() {
            return Err(unsupported_query());
        }
        let mut preview_bytes =
            metadata_bytes(statement.columns().iter().map(|column| column.name()))?;
        let columns = statement
            .columns()
            .iter()
            .map(|column| column.name().to_owned())
            .collect::<Vec<_>>();
        self.client
            .execute(
                &format!("DECLARE videre_preview NO SCROLL CURSOR FOR {sql}"),
                &[],
            )
            .await
            .map_err(|error| pg_error(&unsupported_query(), &error))?;

        // Only internally generated, small FETCH batches use the simple
        // protocol, preserving exact PostgreSQL text without rewriting user SQL.
        // The driver decodes each entire batch BEFORE these checks: even one
        // oversized row can still allocate heavily. These are retention budgets,
        // not an absolute driver/IPC memory bound; batching only limits the burst.
        let mut rows = Vec::new();
        loop {
            let fetch_count = preview_fetch_size(rows.len());
            let messages = self
                .client
                .simple_query(&format!("FETCH FORWARD {fetch_count} FROM videre_preview"))
                .await
                .map_err(|error| pg_error("Query execution failed", &error))?;
            let mut fetched = 0;
            for message in messages {
                if let SimpleQueryMessage::Row(row) = message {
                    fetched += 1;
                    if row.len() != columns.len() {
                        return Err("Query result column count changed while executing".into());
                    }
                    let row_bytes = preview_row_bytes((0..row.len()).map(|index| row.get(index)))?;
                    if rows.len() == ROW_LIMIT {
                        // Check the sentinel's row budget, but neither retain it
                        // nor charge it against the retained preview payload.
                        return Ok((columns, rows, true));
                    }
                    reserve_preview_bytes(&mut preview_bytes, row_bytes)?;
                    rows.push(
                        (0..row.len())
                            .map(|index| {
                                row.get(index)
                                    .map_or(Value::Null, |text| Value::String(text.to_owned()))
                            })
                            .collect(),
                    );
                }
            }
            if fetched < fetch_count {
                return Ok((columns, rows, false));
            }
        }
    }
}

fn preview_fetch_size(retained_rows: usize) -> usize {
    FETCH_BATCH_SIZE.min(ROW_LIMIT + 1 - retained_rows)
}

// Include basic container storage so empty strings/NULLs in wide results still
// consume budget. Allocator overhead, JSON escaping and driver buffers are not
// measured here; this is deliberately not a claim about total process memory.
fn metadata_bytes<'a>(names: impl Iterator<Item = &'a str>) -> Result<usize, String> {
    let mut bytes = 0usize;
    for (index, name) in names.enumerate() {
        if index >= MAX_COLUMNS {
            return Err(format!("Query metadata exceeds the {MAX_COLUMNS}-column preview limit. Select fewer columns and retry."));
        }
        bytes = bytes
            .checked_add(std::mem::size_of::<String>())
            .and_then(|bytes| bytes.checked_add(name.len()))
            .filter(|bytes| *bytes <= MAX_METADATA_BYTES)
            .ok_or_else(|| "Query metadata exceeds the 64 KiB preview budget. Select fewer columns or shorter column aliases and retry.".to_string())?;
    }
    Ok(bytes)
}

fn preview_row_bytes<'a>(cells: impl Iterator<Item = Option<&'a str>>) -> Result<usize, String> {
    let mut bytes = std::mem::size_of::<Vec<Value>>();
    for cell in cells {
        bytes = bytes
            .checked_add(std::mem::size_of::<Value>())
            .and_then(|bytes| bytes.checked_add(cell.map_or(0, str::len)))
            .filter(|bytes| *bytes <= MAX_ROW_BYTES)
            .ok_or_else(|| ROW_BUDGET_ERROR.to_string())?;
    }
    Ok(bytes)
}

fn reserve_preview_bytes(preview_bytes: &mut usize, row_bytes: usize) -> Result<(), String> {
    *preview_bytes = preview_bytes
        .checked_add(row_bytes)
        .filter(|bytes| *bytes <= MAX_PREVIEW_BYTES)
        .ok_or_else(|| PREVIEW_BUDGET_ERROR.to_string())?;
    Ok(())
}

async fn wait_for_cancellation(receiver: &mut watch::Receiver<bool>) {
    // Do not retain watch::Ref across an await in a select branch: that guard
    // is not Send and would also block other cancellation signal writers.
    let _ = receiver.wait_for(|cancelled| *cancelled).await;
}

fn unsupported_query() -> String {
    "Only one row-returning SELECT, WITH or VALUES query is supported; writes, transaction control and cursor-incompatible statements such as EXPLAIN/SHOW are not supported".into()
}

fn validate_input(sql: &str, request_id: &str) -> Result<(), String> {
    if sql.trim().is_empty() {
        return Err("SQL must not be empty".into());
    }
    if sql.len() > MAX_SQL_BYTES {
        return Err(format!("SQL exceeds the {MAX_SQL_BYTES}-byte input limit"));
    }
    if request_id.trim().is_empty() || request_id.len() > MAX_REQUEST_ID_BYTES {
        return Err(format!(
            "requestId must be non-empty and at most {MAX_REQUEST_ID_BYTES} bytes"
        ));
    }
    Ok(())
}

fn pg_error(context: &str, error: &Error) -> String {
    match error.as_db_error() {
        Some(db) => {
            let mut message = format!(
                "{context}: {} (SQLSTATE {})",
                db.message(),
                db.code().code()
            );
            if let Some(detail) = db.detail() {
                message.push_str(&format!("\nDetail: {detail}"));
            }
            if let Some(hint) = db.hint() {
                message.push_str(&format!("\nHint: {hint}"));
            }
            message
        }
        None => format!("{context}: {error}"),
    }
}

#[cfg(test)]
mod tests;
