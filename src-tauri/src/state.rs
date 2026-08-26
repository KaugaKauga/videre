use std::sync::Arc;
use tokio::sync::Mutex;

use crate::pg;
use crate::types::ConnectionConfig;

/// Application state: the one live connection, if any.
pub struct DbState {
    /// The only place shared state names an engine. When a second one lands,
    /// this field becomes an `enum Connection { Postgres(..), .. }` and nothing
    /// else in `state.rs` or `commands.rs` needs to change shape.
    pub connection: Arc<Mutex<Option<pg::Connection>>>,
    pub config: Arc<Mutex<Option<ConnectionConfig>>>,
}

impl DbState {
    pub fn new() -> Self {
        Self {
            connection: Arc::new(Mutex::new(None)),
            config: Arc::new(Mutex::new(None)),
        }
    }
}
