use std::sync::Arc;
use tokio::sync::Mutex;

use crate::pg;
use crate::types::{ConnectionConfig, ConnectionInfo, QueryResult};

/// Application state: the one live connection, if any.
pub struct DbState {
    /// The only place shared state names an engine. When a second one lands,
    /// this field becomes an `enum Connection { Postgres(..), .. }` and nothing
    /// else in `state.rs` or `commands.rs` needs to change shape.
    pub connection: Arc<Mutex<Option<pg::Connection>>>,
    pub config: Arc<Mutex<Option<ConnectionConfig>>>,
    pub queries: pg::QueryRegistry,
}

impl DbState {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(None)),
            queries: pg::QueryRegistry::default(),
        }
    }

    // Always acquire connection before config, matching connect/disconnect.
    // Return an owned snapshot so editor queries never hold the browsing lock.
    async fn active_config(&self) -> Result<ConnectionConfig, String> {
        let connection = self.connection.lock().await;
        let config = self.config.lock().await;
        if connection.is_none() {
            return Err("Not connected to database".into());
        }
        config
            .clone()
            .ok_or_else(|| "Not connected to database".into())
    }

    pub async fn connection_info(&self) -> Result<ConnectionInfo, String> {
        self.active_config()
            .await
            .map(|config| ConnectionInfo::from(&config))
    }

    pub async fn execute_query(&self, sql: &str, request_id: &str) -> Result<QueryResult, String> {
        let request = self.queries.register(sql, request_id)?;
        request.execute(sql, self.active_config()).await
    }
}
