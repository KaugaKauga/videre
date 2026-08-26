//! Everything Postgres-specific: catalog SQL, identifier quoting, wire-type
//! decoding. Nothing engine-agnostic should reach into here.
//!
//! Not behind a trait yet — deliberately. One implementation can't shape that
//! contract: the `::text` cast in [`convert`] has no MySQL equivalent, SQLite
//! has no per-column type to ask for, and the privilege and schema models
//! differ enough that the shared types need reshaping too. When a second engine
//! lands, the seam goes here.

pub mod convert;

pub use convert::{qualified_name, quote_ident, ColumnPlan};
