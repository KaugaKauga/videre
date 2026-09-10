//! Reading PostgreSQL catalogs for database objects: relations, indexes, roles, and privileges.

use super::Connection;
use crate::types::{ForeignKeyInfo, IndexInfo, RoleInfo, TableInfo, TablePrivilege};

impl Connection {
    pub async fn tables(&self) -> Result<Vec<TableInfo>, String> {
        let query = "
            SELECT table_name, table_schema
            FROM information_schema.tables
            WHERE table_schema NOT IN ('pg_catalog', 'information_schema')
            ORDER BY table_schema, table_name
        ";

        let rows = self
            .client
            .query(query, &[])
            .await
            .map_err(|e| format!("Failed to fetch tables: {e}"))?;

        Ok(rows
            .iter()
            .map(|row| TableInfo {
                name: row.get(0),
                schema: row.get(1),
            })
            .collect())
    }
    pub async fn foreign_keys(
        &self,
        schema: &str,
        table_name: &str,
    ) -> Result<Vec<ForeignKeyInfo>, String> {
        let query = "
            SELECT
                kcu.column_name,
                ccu.table_schema AS foreign_table_schema,
                ccu.table_name AS foreign_table_name,
                ccu.column_name AS foreign_column_name
            FROM information_schema.table_constraints AS tc
            JOIN information_schema.key_column_usage AS kcu
                ON tc.constraint_name = kcu.constraint_name
                AND tc.table_schema = kcu.table_schema
            JOIN information_schema.constraint_column_usage AS ccu
                ON ccu.constraint_name = tc.constraint_name
                AND ccu.table_schema = tc.table_schema
            WHERE tc.constraint_type = 'FOREIGN KEY'
                AND tc.table_schema = $1
                AND tc.table_name = $2
        ";

        let rows = self
            .client
            .query(query, &[&schema, &table_name])
            .await
            .map_err(|e| format!("Failed to fetch foreign keys: {e}"))?;

        Ok(rows
            .iter()
            .map(|row| ForeignKeyInfo {
                column_name: row.get(0),
                foreign_table_schema: row.get(1),
                foreign_table_name: row.get(2),
                foreign_column_name: row.get(3),
            })
            .collect())
    }

    pub async fn indexes(&self, schema: &str, table_name: &str) -> Result<Vec<IndexInfo>, String> {
        // Consolidates columns into an array and includes type + size.
        let query = "
            SELECT
                i.relname AS index_name,
                n.nspname AS schema_name,
                t.relname AS table_name,
                ARRAY_AGG(a.attname ORDER BY array_position(ix.indkey, a.attnum)) AS columns,
                ix.indisunique AS is_unique,
                ix.indisprimary AS is_primary,
                am.amname AS index_type,
                pg_relation_size(i.oid) AS size_bytes
            FROM pg_index ix
            JOIN pg_class i ON i.oid = ix.indexrelid
            JOIN pg_class t ON t.oid = ix.indrelid
            JOIN pg_namespace n ON n.oid = t.relnamespace
            JOIN pg_am am ON i.relam = am.oid
            JOIN pg_attribute a ON a.attrelid = t.oid AND a.attnum = ANY(ix.indkey)
            WHERE n.nspname = $1 AND t.relname = $2
            GROUP BY i.relname, n.nspname, t.relname, ix.indisunique, ix.indisprimary, am.amname, i.oid
            ORDER BY i.relname
        ";

        let rows = self
            .client
            .query(query, &[&schema, &table_name])
            .await
            .map_err(|e| format!("Failed to fetch indexes: {e}"))?;

        Ok(rows
            .iter()
            .map(|row| IndexInfo {
                index_name: row.get(0),
                table_schema: row.get(1),
                table_name: row.get(2),
                columns: row.get(3),
                is_unique: row.get(4),
                is_primary: row.get(5),
                index_type: row.get(6),
                size_bytes: row.get(7),
            })
            .collect())
    }

    pub async fn roles(&self) -> Result<Vec<RoleInfo>, String> {
        let query = "
            SELECT
                r.rolname AS role_name,
                r.rolsuper AS is_superuser,
                r.rolcanlogin AS can_login,
                r.rolcreatedb AS can_create_db,
                r.rolcreaterole AS can_create_role,
                r.rolconnlimit AS connection_limit,
                r.rolvaliduntil::text AS valid_until,
                COALESCE(
                    ARRAY_AGG(g.rolname ORDER BY g.rolname) FILTER (WHERE g.rolname IS NOT NULL),
                    ARRAY[]::text[]
                ) AS member_of
            FROM pg_roles r
            LEFT JOIN pg_auth_members m ON r.oid = m.member
            LEFT JOIN pg_roles g ON m.roleid = g.oid
            WHERE r.rolname NOT LIKE 'pg_%'
            GROUP BY r.rolname, r.rolsuper, r.rolcanlogin, r.rolcreatedb, r.rolcreaterole, r.rolconnlimit, r.rolvaliduntil
            ORDER BY r.rolname
        ";

        let rows = self
            .client
            .query(query, &[])
            .await
            .map_err(|e| format!("Failed to fetch roles: {e}"))?;

        Ok(rows
            .iter()
            .map(|row| RoleInfo {
                role_name: row.get(0),
                is_superuser: row.get(1),
                can_login: row.get(2),
                can_create_db: row.get(3),
                can_create_role: row.get(4),
                connection_limit: row.get(5),
                valid_until: row.get(6),
                member_of: row.get(7),
            })
            .collect())
    }

    pub async fn table_privileges(&self) -> Result<Vec<TablePrivilege>, String> {
        let query = "
            SELECT
                grantee::text,
                table_schema::text,
                table_name::text,
                ARRAY_AGG(privilege_type::text ORDER BY privilege_type) AS privileges
            FROM information_schema.table_privileges
            WHERE grantee NOT LIKE 'pg_%'
              AND table_schema NOT IN ('pg_catalog', 'information_schema')
            GROUP BY grantee, table_schema, table_name
            ORDER BY grantee, table_schema, table_name
        ";

        let rows = self
            .client
            .query(query, &[])
            .await
            .map_err(|e| format!("Failed to fetch table privileges: {e}"))?;

        Ok(rows
            .iter()
            .map(|row| TablePrivilege {
                grantee: row.get(0),
                table_schema: row.get(1),
                table_name: row.get(2),
                privileges: row.get(3),
            })
            .collect())
    }
}
