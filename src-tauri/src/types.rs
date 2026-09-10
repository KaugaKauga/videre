//! The IPC contract: what crosses the boundary to the frontend.
//!
//! Engine-agnostic on purpose. Nothing in here may depend on `tokio_postgres` or
//! on anything in [`crate::pg`] — if a type only makes sense for one engine, it
//! does not belong in this file. That constraint is what keeps a second adapter
//! from needing a second set of DTOs.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Serialize)]
pub struct ConnectionResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Serialize)]
pub struct TableInfo {
    pub name: String,
    pub schema: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct ForeignKeyInfo {
    pub column_name: String,
    pub foreign_table_schema: String,
    pub foreign_table_name: String,
    pub foreign_column_name: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct IndexInfo {
    pub index_name: String,
    pub table_schema: String,
    pub table_name: String,
    pub columns: Vec<String>,
    pub is_unique: bool,
    pub is_primary: bool,
    pub index_type: String,
    pub size_bytes: i64,
}

#[derive(Debug, Clone, Serialize)]
pub struct RoleInfo {
    pub role_name: String,
    pub is_superuser: bool,
    pub can_login: bool,
    pub can_create_db: bool,
    pub can_create_role: bool,
    pub connection_limit: i32,
    pub valid_until: Option<String>,
    pub member_of: Vec<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TablePrivilege {
    pub grantee: String,
    pub table_schema: String,
    pub table_name: String,
    pub privileges: Vec<String>,
}

#[derive(Debug, Clone, Copy, Default, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SortDirection {
    #[default]
    Asc,
    Desc,
}

impl SortDirection {
    pub fn is_descending(self) -> bool {
        matches!(self, Self::Desc)
    }
}

#[derive(Debug, Serialize)]
pub struct TableData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub total_rows: i64,
}

#[derive(Debug, Serialize)]
pub struct RowData {
    pub columns: Vec<String>,
    pub values: Vec<serde_json::Value>,
}

/// Engine-neutral description of a relation and all of its visible columns.
#[derive(Debug, Clone, Serialize)]
pub struct RelationStructure {
    pub schema: String,
    pub relation: String,
    pub relation_kind: RelationKind,
    pub columns: Vec<ColumnInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RelationKind {
    Table,
    PartitionedTable,
    View,
    MaterializedView,
    ForeignTable,
}

#[derive(Debug, Clone, Serialize)]
pub struct ColumnInfo {
    pub name: String,
    pub ordinal_position: i16,
    pub data_type: String,
    pub nullable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub collation: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_expression: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub identity: Option<IdentityKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generated: Option<GeneratedKind>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub generation_expression: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comment: Option<String>,
    pub constraints: Vec<ColumnConstraint>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum IdentityKind {
    Always,
    ByDefault,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum GeneratedKind {
    Stored,
    Virtual,
}

#[derive(Debug, Clone, Serialize)]
pub struct ColumnConstraint {
    pub name: String,
    pub kind: ConstraintKind,
    pub definition: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub column_position: Option<i16>,
    pub source_columns: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_schema: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_relation: Option<String>,
    pub target_columns: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_update: Option<ForeignKeyAction>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub on_delete: Option<ForeignKeyAction>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ConstraintKind {
    PrimaryKey,
    Unique,
    ForeignKey,
    Check,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ForeignKeyAction {
    NoAction,
    Restrict,
    Cascade,
    SetNull,
    SetDefault,
}
