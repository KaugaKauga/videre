use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    pub host: String,
    pub port: String,
    pub database: String,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionResult {
    pub success: bool,
    pub message: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableInfo {
    pub name: String,
    pub schema: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TableData {
    pub columns: Vec<String>,
    pub rows: Vec<Vec<serde_json::Value>>,
    pub total_rows: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ForeignKeyInfo {
    pub column_name: String,
    pub foreign_table_schema: String,
    pub foreign_table_name: String,
    pub foreign_column_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TablePrivilege {
    pub grantee: String,
    pub table_schema: String,
    pub table_name: String,
    pub privileges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RowData {
    pub columns: Vec<String>,
    pub values: Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn connection_config_round_trip() {
        let config = ConnectionConfig {
            host: "localhost".into(),
            port: "5432".into(),
            database: "videre_test".into(),
            username: "videre".into(),
            password: "secret".into(),
        };
        let json = serde_json::to_string(&config).unwrap();
        let parsed: ConnectionConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.host, "localhost");
        assert_eq!(parsed.port, "5432");
        assert_eq!(parsed.database, "videre_test");
        assert_eq!(parsed.username, "videre");
        assert_eq!(parsed.password, "secret");
    }

    #[test]
    fn connection_result_round_trip() {
        let result = ConnectionResult {
            success: true,
            message: "Connected".into(),
        };
        let json = serde_json::to_string(&result).unwrap();
        let parsed: ConnectionResult = serde_json::from_str(&json).unwrap();
        assert!(parsed.success);
        assert_eq!(parsed.message, "Connected");
    }

    #[test]
    fn table_info_round_trip() {
        let info = TableInfo {
            name: "gods".into(),
            schema: "public".into(),
        };
        let json = serde_json::to_string(&info).unwrap();
        let parsed: TableInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "gods");
        assert_eq!(parsed.schema, "public");
    }

    #[test]
    fn table_data_round_trip() {
        let data = TableData {
            columns: vec!["id".into(), "name".into()],
            rows: vec![vec![json!(1), json!("Zeus")]],
            total_rows: 1,
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: TableData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.columns.len(), 2);
        assert_eq!(parsed.rows.len(), 1);
        assert_eq!(parsed.total_rows, 1);
    }

    #[test]
    fn table_data_empty_rows() {
        let data = TableData {
            columns: vec!["id".into()],
            rows: vec![],
            total_rows: 0,
        };
        let json = serde_json::to_string(&data).unwrap();
        let parsed: TableData = serde_json::from_str(&json).unwrap();
        assert!(parsed.rows.is_empty());
        assert_eq!(parsed.total_rows, 0);
    }

    #[test]
    fn foreign_key_info_round_trip() {
        let fk = ForeignKeyInfo {
            column_name: "god_id".into(),
            foreign_table_schema: "public".into(),
            foreign_table_name: "gods".into(),
            foreign_column_name: "id".into(),
        };
        let json = serde_json::to_string(&fk).unwrap();
        let parsed: ForeignKeyInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.column_name, "god_id");
        assert_eq!(parsed.foreign_table_name, "gods");
    }

    #[test]
    fn index_info_round_trip() {
        let idx = IndexInfo {
            index_name: "pk_gods".into(),
            table_schema: "public".into(),
            table_name: "gods".into(),
            columns: vec!["id".into()],
            is_unique: true,
            is_primary: true,
            index_type: "btree".into(),
            size_bytes: 16384,
        };
        let json = serde_json::to_string(&idx).unwrap();
        let parsed: IndexInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.index_name, "pk_gods");
        assert!(parsed.is_unique);
        assert!(parsed.is_primary);
        assert_eq!(parsed.size_bytes, 16384);
    }

    #[test]
    fn role_info_round_trip() {
        let role = RoleInfo {
            role_name: "videre".into(),
            is_superuser: false,
            can_login: true,
            can_create_db: false,
            can_create_role: false,
            connection_limit: -1,
            valid_until: None,
            member_of: vec!["pg_read_all_data".into()],
        };
        let json = serde_json::to_string(&role).unwrap();
        let parsed: RoleInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.role_name, "videre");
        assert!(!parsed.is_superuser);
        assert!(parsed.can_login);
        assert!(parsed.valid_until.is_none());
        assert_eq!(parsed.member_of, vec!["pg_read_all_data"]);
    }

    #[test]
    fn role_info_with_valid_until() {
        let role = RoleInfo {
            role_name: "temp".into(),
            is_superuser: false,
            can_login: true,
            can_create_db: false,
            can_create_role: false,
            connection_limit: 5,
            valid_until: Some("2026-12-31".into()),
            member_of: vec![],
        };
        let json = serde_json::to_string(&role).unwrap();
        let parsed: RoleInfo = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.valid_until, Some("2026-12-31".into()));
        assert_eq!(parsed.connection_limit, 5);
    }

    #[test]
    fn table_privilege_round_trip() {
        let priv_ = TablePrivilege {
            grantee: "videre".into(),
            table_schema: "public".into(),
            table_name: "gods".into(),
            privileges: vec!["SELECT".into(), "INSERT".into()],
        };
        let json = serde_json::to_string(&priv_).unwrap();
        let parsed: TablePrivilege = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.grantee, "videre");
        assert_eq!(parsed.privileges.len(), 2);
    }

    #[test]
    fn row_data_with_mixed_values() {
        let row = RowData {
            columns: vec!["id".into(), "name".into(), "active".into()],
            values: vec![json!(42), json!("Heracles"), json!(true)],
        };
        let json = serde_json::to_string(&row).unwrap();
        let parsed: RowData = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.columns.len(), 3);
        assert_eq!(parsed.values[0], json!(42));
        assert_eq!(parsed.values[1], json!("Heracles"));
        assert_eq!(parsed.values[2], json!(true));
    }

    #[test]
    fn row_data_with_null_values() {
        let row = RowData {
            columns: vec!["id".into(), "notes".into()],
            values: vec![json!(1), json!(null)],
        };
        let json = serde_json::to_string(&row).unwrap();
        let parsed: RowData = serde_json::from_str(&json).unwrap();
        assert!(parsed.values[1].is_null());
    }

    #[test]
    fn deserialize_from_external_json() {
        let json_str = r#"{"name":"heroes","schema":"public"}"#;
        let info: TableInfo = serde_json::from_str(json_str).unwrap();
        assert_eq!(info.name, "heroes");
        assert_eq!(info.schema, "public");
    }
}