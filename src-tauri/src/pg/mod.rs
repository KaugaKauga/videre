//! Everything Postgres-specific: connection setup, catalog SQL, identifier
//! quoting, wire-type decoding. Nothing outside this module should mention
//! `tokio_postgres`.
//!
//! Not behind a trait yet — deliberately. One implementation can't shape that
//! contract: the `::text` cast in [`convert`] has no MySQL equivalent, SQLite
//! has no per-column type to ask for, and the privilege and schema models
//! differ enough that the shared types need reshaping too. The method set on
//! [`Connection`] is the shape that trait will take, discovered rather than
//! guessed. When a second engine lands, the seam goes here.

pub mod catalog;
pub mod convert;
pub mod data;
pub mod relation_structure;

use tokio_postgres::{Client, NoTls};

use crate::types::ConnectionConfig;

/// A live Postgres connection.
pub struct Connection {
    // Private, so only this module and its children touch the driver. Sibling
    // modules (`catalog`, `data`, `relation_structure`) reach it as `self.client`.
    client: Client,
}

impl Connection {
    /// Open a connection and hand back the handle.
    pub async fn connect(config: &ConnectionConfig) -> Result<Self, String> {
        let client = establish(config).await?;
        Ok(Self { client })
    }

    /// Open a connection, run a trivial query, and drop it. Used by the
    /// connection form to check credentials before committing to them.
    pub async fn probe(config: &ConnectionConfig) -> Result<(), String> {
        let client = establish(config).await?;
        client
            .query("SELECT 1", &[])
            .await
            .map_err(|e| format!("Query failed: {e}"))?;
        Ok(())
    }
}

async fn establish(config: &ConnectionConfig) -> Result<Client, String> {
    let connection_string = format!(
        "host={} port={} dbname={} user={} password={}",
        config.host, config.port, config.database, config.username, config.password
    );

    let (client, connection) = tokio_postgres::connect(&connection_string, NoTls)
        .await
        .map_err(|e| format!("Connection failed: {e}"))?;

    // Drives the connection until the client is dropped. Note that nothing
    // observes termination, so a server restart leaves a dead handle in state.
    tokio::spawn(async move {
        if let Err(e) = connection.await {
            eprintln!("connection error: {e}");
        }
    });

    Ok(client)
}
