//! PostgreSQL relation-wide column and constraint inspection.

use super::Connection;
use crate::types::{
    ColumnConstraint, ColumnInfo, ConstraintKind, ForeignKeyAction, GeneratedKind, IdentityKind,
    RelationKind, RelationStructure,
};

impl Connection {
    /// Load the complete column structure in one relation-scoped request.
    ///
    /// Catalog names are bound as values rather than interpolated into SQL, so quoted,
    /// mixed-case, and otherwise unusual identifiers retain their exact meaning.
    pub async fn relation_structure(
        &self,
        schema: &str,
        relation: &str,
    ) -> Result<RelationStructure, String> {
        let relation_query = "
            SELECT relation.oid, namespace.nspname, relation.relname, relation.relkind::text
            FROM pg_catalog.pg_class AS relation
            JOIN pg_catalog.pg_namespace AS namespace
                ON namespace.oid = relation.relnamespace
            WHERE namespace.nspname = $1
              AND relation.relname = $2
              AND relation.relkind IN ('r', 'p', 'v', 'm', 'f')
        ";
        let relation_row = self
            .client
            .query_opt(relation_query, &[&schema, &relation])
            .await
            .map_err(|error| format!("Failed to inspect relation structure: {error}"))?
            .ok_or_else(|| {
                format!("Relation {schema}.{relation} was not found or is not supported")
            })?;

        let relation_oid: u32 = relation_row.get(0);
        let relation_kind = relation_kind_from_catalog(&relation_row.get::<_, String>(3))
            .ok_or_else(|| "PostgreSQL returned an unsupported relation kind".to_string())?;

        let columns_query = "
            SELECT
                attribute.attnum,
                attribute.attname,
                CASE typ.typname
                    WHEN 'varchar' THEN regexp_replace(
                        pg_catalog.format_type(attribute.atttypid, attribute.atttypmod),
                        '^character varying',
                        'varchar'
                    )
                    WHEN '_varchar' THEN regexp_replace(
                        pg_catalog.format_type(attribute.atttypid, attribute.atttypmod),
                        '^character varying',
                        'varchar'
                    )
                    WHEN 'bpchar' THEN regexp_replace(
                        pg_catalog.format_type(attribute.atttypid, attribute.atttypmod),
                        '^character',
                        'char'
                    )
                    WHEN '_bpchar' THEN regexp_replace(
                        pg_catalog.format_type(attribute.atttypid, attribute.atttypmod),
                        '^character',
                        'char'
                    )
                    ELSE pg_catalog.format_type(attribute.atttypid, attribute.atttypmod)
                END AS data_type,
                NOT attribute.attnotnull AS nullable,
                CASE
                    WHEN attribute.attcollation <> 0
                     AND attribute.attcollation <> typ.typcollation
                    THEN pg_catalog.format('%I.%I', collation_namespace.nspname, collation_catalog.collname)
                END AS collation,
                pg_catalog.pg_get_expr(default_value.adbin, default_value.adrelid) AS expression,
                attribute.attidentity::text,
                attribute.attgenerated::text,
                pg_catalog.col_description(attribute.attrelid, attribute.attnum)
            FROM pg_catalog.pg_attribute AS attribute
            JOIN pg_catalog.pg_type AS typ ON typ.oid = attribute.atttypid
            LEFT JOIN pg_catalog.pg_attrdef AS default_value
                ON default_value.adrelid = attribute.attrelid
               AND default_value.adnum = attribute.attnum
            LEFT JOIN pg_catalog.pg_collation AS collation_catalog
                ON collation_catalog.oid = attribute.attcollation
            LEFT JOIN pg_catalog.pg_namespace AS collation_namespace
                ON collation_namespace.oid = collation_catalog.collnamespace
            WHERE attribute.attrelid = $1
              AND attribute.attnum > 0
              AND NOT attribute.attisdropped
            ORDER BY attribute.attnum
        ";
        let column_rows = self
            .client
            .query(columns_query, &[&relation_oid])
            .await
            .map_err(|error| format!("Failed to inspect relation columns: {error}"))?;
        let mut columns = column_rows
            .iter()
            .map(|row| {
                let identity = identity_from_catalog(&row.get::<_, String>(6));
                let generated = generated_from_catalog(&row.get::<_, String>(7));
                let expression: Option<String> = row.get(5);

                ColumnInfo {
                    name: row.get(1),
                    ordinal_position: row.get(0),
                    data_type: row.get(2),
                    nullable: row.get(3),
                    collation: row.get(4),
                    default_expression: if identity.is_none() && generated.is_none() {
                        expression.clone()
                    } else {
                        None
                    },
                    identity,
                    generated,
                    generation_expression: generated.and(expression),
                    comment: row.get(8),
                    constraints: Vec::new(),
                }
            })
            .collect::<Vec<_>>();

        let constraints_query = "
            SELECT
                constraint_catalog.conname,
                constraint_catalog.contype::text,
                pg_catalog.pg_get_constraintdef(constraint_catalog.oid, false),
                COALESCE(source.columns, ARRAY[]::text[]),
                target_namespace.nspname,
                target_relation.relname,
                COALESCE(target.columns, ARRAY[]::text[]),
                constraint_catalog.confupdtype::text,
                constraint_catalog.confdeltype::text
            FROM pg_catalog.pg_constraint AS constraint_catalog
            LEFT JOIN LATERAL (
                SELECT array_agg(attribute.attname ORDER BY source_column.position) AS columns
                FROM (
                    SELECT key_column.attnum, key_column.position::integer
                    FROM unnest(COALESCE(constraint_catalog.conkey, ARRAY[]::smallint[]))
                        WITH ORDINALITY AS key_column(attnum, position)
                    WHERE constraint_catalog.contype <> 'c'

                    UNION ALL

                    SELECT
                        dependency.refobjsubid::smallint,
                        dependency.refobjsubid::integer
                    FROM pg_catalog.pg_depend AS dependency
                    WHERE constraint_catalog.contype = 'c'
                      AND dependency.classid = 'pg_catalog.pg_constraint'::pg_catalog.regclass
                      AND dependency.objid = constraint_catalog.oid
                      AND dependency.refclassid = 'pg_catalog.pg_class'::pg_catalog.regclass
                      AND dependency.refobjid = constraint_catalog.conrelid
                      AND dependency.refobjsubid > 0
                    GROUP BY dependency.refobjsubid
                ) AS source_column
                JOIN pg_catalog.pg_attribute AS attribute
                    ON attribute.attrelid = constraint_catalog.conrelid
                   AND attribute.attnum = source_column.attnum
            ) AS source ON true
            LEFT JOIN pg_catalog.pg_class AS target_relation
                ON target_relation.oid = constraint_catalog.confrelid
            LEFT JOIN pg_catalog.pg_namespace AS target_namespace
                ON target_namespace.oid = target_relation.relnamespace
            LEFT JOIN LATERAL (
                SELECT array_agg(attribute.attname ORDER BY key_column.position) AS columns
                FROM unnest(constraint_catalog.confkey::smallint[]) WITH ORDINALITY
                    AS key_column(attnum, position)
                JOIN pg_catalog.pg_attribute AS attribute
                    ON attribute.attrelid = constraint_catalog.confrelid
                   AND attribute.attnum = key_column.attnum
            ) AS target ON true
            WHERE constraint_catalog.conrelid = $1
              AND constraint_catalog.contype IN ('p', 'u', 'f', 'c')
            ORDER BY constraint_catalog.conname
        ";
        let constraint_rows = self
            .client
            .query(constraints_query, &[&relation_oid])
            .await
            .map_err(|error| format!("Failed to inspect relation constraints: {error}"))?;

        for row in &constraint_rows {
            let Some(kind) = constraint_kind_from_catalog(&row.get::<_, String>(1)) else {
                continue;
            };
            let source_columns: Vec<String> = row.get(3);
            let is_foreign_key = kind == ConstraintKind::ForeignKey;
            let has_column_position = matches!(
                kind,
                ConstraintKind::PrimaryKey | ConstraintKind::Unique | ConstraintKind::ForeignKey
            );
            let constraint = ColumnConstraint {
                name: row.get(0),
                kind,
                definition: row.get(2),
                column_position: None,
                source_columns: source_columns.clone(),
                target_schema: if is_foreign_key { row.get(4) } else { None },
                target_relation: if is_foreign_key { row.get(5) } else { None },
                target_columns: if is_foreign_key {
                    row.get(6)
                } else {
                    Vec::new()
                },
                on_update: if is_foreign_key {
                    foreign_key_action_from_catalog(&row.get::<_, String>(7))
                } else {
                    None
                },
                on_delete: if is_foreign_key {
                    foreign_key_action_from_catalog(&row.get::<_, String>(8))
                } else {
                    None
                },
            };

            for (index, source_column) in source_columns.iter().enumerate() {
                if let Some(column) = columns
                    .iter_mut()
                    .find(|column| column.name == *source_column)
                {
                    let mut constraint = constraint.clone();
                    if has_column_position {
                        constraint.column_position = i16::try_from(index + 1).ok();
                    }
                    column.constraints.push(constraint);
                }
            }
        }

        Ok(RelationStructure {
            schema: relation_row.get(1),
            relation: relation_row.get(2),
            relation_kind,
            columns,
        })
    }
}
fn relation_kind_from_catalog(value: &str) -> Option<RelationKind> {
    match value {
        "r" => Some(RelationKind::Table),
        "p" => Some(RelationKind::PartitionedTable),
        "v" => Some(RelationKind::View),
        "m" => Some(RelationKind::MaterializedView),
        "f" => Some(RelationKind::ForeignTable),
        _ => None,
    }
}

fn identity_from_catalog(value: &str) -> Option<IdentityKind> {
    match value {
        "a" => Some(IdentityKind::Always),
        "d" => Some(IdentityKind::ByDefault),
        _ => None,
    }
}

fn generated_from_catalog(value: &str) -> Option<GeneratedKind> {
    match value {
        "s" => Some(GeneratedKind::Stored),
        "v" => Some(GeneratedKind::Virtual),
        _ => None,
    }
}

fn constraint_kind_from_catalog(value: &str) -> Option<ConstraintKind> {
    match value {
        "p" => Some(ConstraintKind::PrimaryKey),
        "u" => Some(ConstraintKind::Unique),
        "f" => Some(ConstraintKind::ForeignKey),
        "c" => Some(ConstraintKind::Check),
        _ => None,
    }
}

fn foreign_key_action_from_catalog(value: &str) -> Option<ForeignKeyAction> {
    match value {
        "a" => Some(ForeignKeyAction::NoAction),
        "r" => Some(ForeignKeyAction::Restrict),
        "c" => Some(ForeignKeyAction::Cascade),
        "n" => Some(ForeignKeyAction::SetNull),
        "d" => Some(ForeignKeyAction::SetDefault),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ConnectionConfig;

    fn test_config() -> ConnectionConfig {
        let var =
            |name: &str, default: &str| std::env::var(name).unwrap_or_else(|_| default.to_string());
        ConnectionConfig {
            host: var("VIDERE_TEST_PG_HOST", "localhost"),
            port: var("VIDERE_TEST_PG_PORT", "5432"),
            database: var("VIDERE_TEST_PG_DB", "videre_test"),
            username: var("VIDERE_TEST_PG_USER", "videre"),
            password: var("VIDERE_TEST_PG_PASSWORD", "videre"),
        }
    }

    async fn connect() -> Connection {
        Connection::connect(&test_config())
            .await
            .expect("test database unreachable — is `docker compose up -d` running?")
    }

    #[test]
    fn maps_supported_relation_kinds() {
        assert_eq!(relation_kind_from_catalog("r"), Some(RelationKind::Table));
        assert_eq!(
            relation_kind_from_catalog("p"),
            Some(RelationKind::PartitionedTable)
        );
        assert_eq!(relation_kind_from_catalog("v"), Some(RelationKind::View));
        assert_eq!(
            relation_kind_from_catalog("m"),
            Some(RelationKind::MaterializedView)
        );
        assert_eq!(
            relation_kind_from_catalog("f"),
            Some(RelationKind::ForeignTable)
        );
        assert_eq!(relation_kind_from_catalog("S"), None);
    }

    #[test]
    fn maps_identity_and_generated_states() {
        assert_eq!(identity_from_catalog("a"), Some(IdentityKind::Always));
        assert_eq!(identity_from_catalog("d"), Some(IdentityKind::ByDefault));
        assert_eq!(identity_from_catalog(""), None);
        assert_eq!(generated_from_catalog("s"), Some(GeneratedKind::Stored));
        assert_eq!(generated_from_catalog("v"), Some(GeneratedKind::Virtual));
        assert_eq!(generated_from_catalog(""), None);
    }

    #[test]
    fn maps_supported_constraint_kinds() {
        assert_eq!(
            constraint_kind_from_catalog("p"),
            Some(ConstraintKind::PrimaryKey)
        );
        assert_eq!(
            constraint_kind_from_catalog("u"),
            Some(ConstraintKind::Unique)
        );
        assert_eq!(
            constraint_kind_from_catalog("f"),
            Some(ConstraintKind::ForeignKey)
        );
        assert_eq!(
            constraint_kind_from_catalog("c"),
            Some(ConstraintKind::Check)
        );
        assert_eq!(constraint_kind_from_catalog("x"), None);
    }

    #[test]
    fn maps_foreign_key_actions() {
        assert_eq!(
            foreign_key_action_from_catalog("a"),
            Some(ForeignKeyAction::NoAction)
        );
        assert_eq!(
            foreign_key_action_from_catalog("r"),
            Some(ForeignKeyAction::Restrict)
        );
        assert_eq!(
            foreign_key_action_from_catalog("c"),
            Some(ForeignKeyAction::Cascade)
        );
        assert_eq!(
            foreign_key_action_from_catalog("n"),
            Some(ForeignKeyAction::SetNull)
        );
        assert_eq!(
            foreign_key_action_from_catalog("d"),
            Some(ForeignKeyAction::SetDefault)
        );
        assert_eq!(foreign_key_action_from_catalog("x"), None);
    }

    #[tokio::test]
    #[ignore]
    async fn relation_structure_reads_real_catalog_metadata() {
        let conn = connect().await;
        conn.client
            .batch_execute(
                "DROP SCHEMA IF EXISTS videre_structure_test CASCADE;
                 CREATE SCHEMA videre_structure_test;
                 CREATE TYPE videre_structure_test.order_status AS ENUM ('open', 'closed');
                 CREATE DOMAIN videre_structure_test.amount_domain AS numeric(12,2);
                 CREATE TABLE videre_structure_test.target (
                     \"Target ID\" integer NOT NULL,
                     tenant integer NOT NULL,
                     PRIMARY KEY (\"Target ID\", tenant)
                 );
                 CREATE TABLE videre_structure_test.\"Quoted Source\" (
                     \"Order ID\" integer GENERATED ALWAYS AS IDENTITY,
                     tenant integer NOT NULL,
                     label varchar(100) COLLATE \"C\" DEFAULT 'new',
                     tags text[] NOT NULL,
                     aliases varchar(40)[],
                     amount numeric(12,2) DEFAULT 0,
                     state videre_structure_test.order_status DEFAULT 'open',
                     domain_amount videre_structure_test.amount_domain,
                     computed integer GENERATED ALWAYS AS (tenant + amount::integer) STORED,
                     parent_id integer,
                     CONSTRAINT quoted_source_pk PRIMARY KEY (\"Order ID\", tenant),
                     CONSTRAINT quoted_source_unique UNIQUE (label, tags),
                     CONSTRAINT quoted_source_fk FOREIGN KEY (parent_id, tenant)
                         REFERENCES videre_structure_test.target (\"Target ID\", tenant)
                         ON UPDATE CASCADE ON DELETE SET NULL,
                     CONSTRAINT quoted_source_check CHECK (amount >= 0 AND tenant > 0)
                 );
                 COMMENT ON COLUMN videre_structure_test.\"Quoted Source\".label IS 'A comment\nwith two lines';
                 CREATE VIEW videre_structure_test.\"Quoted View\" AS
                     SELECT \"Order ID\", label FROM videre_structure_test.\"Quoted Source\";
                 CREATE MATERIALIZED VIEW videre_structure_test.\"Quoted Materialized View\" AS
                     SELECT \"Order ID\", label FROM videre_structure_test.\"Quoted Source\";",
            )
            .await
            .expect("structure fixture DDL should apply");

        let structure = conn
            .relation_structure("videre_structure_test", "Quoted Source")
            .await
            .expect("relation structure should be readable");
        assert_eq!(structure.relation_kind, RelationKind::Table);
        assert_eq!(structure.columns.len(), 10);

        let column = |name: &str| {
            structure
                .columns
                .iter()
                .find(|column| column.name == name)
                .unwrap_or_else(|| panic!("{name} column should be present"))
        };
        let order_id = column("Order ID");
        assert_eq!(order_id.ordinal_position, 1);
        assert_eq!(order_id.identity, Some(IdentityKind::Always));
        assert!(order_id.default_expression.is_none());
        let primary_key = order_id
            .constraints
            .iter()
            .find(|constraint| constraint.kind == ConstraintKind::PrimaryKey)
            .expect("composite primary key should be attached to every member");
        assert_eq!(primary_key.source_columns, ["Order ID", "tenant"]);
        assert_eq!(primary_key.column_position, Some(1));

        let label = column("label");
        assert_eq!(label.data_type, "varchar(100)");
        assert_eq!(
            label.default_expression.as_deref(),
            Some("'new'::character varying")
        );
        assert!(label.collation.is_some());
        assert_eq!(label.comment.as_deref(), Some("A comment\nwith two lines"));
        assert!(label
            .constraints
            .iter()
            .any(|constraint| constraint.kind == ConstraintKind::Unique
                && constraint.source_columns == ["label", "tags"]));

        assert_eq!(column("tags").data_type, "text[]");
        assert_eq!(column("aliases").data_type, "varchar(40)[]");
        assert_eq!(column("amount").data_type, "numeric(12,2)");
        assert_eq!(
            column("state").data_type,
            "videre_structure_test.order_status"
        );
        assert_eq!(
            column("domain_amount").data_type,
            "videre_structure_test.amount_domain"
        );
        let computed = column("computed");
        assert_eq!(computed.generated, Some(GeneratedKind::Stored));
        assert!(computed.default_expression.is_none());
        assert!(computed.generation_expression.is_some());

        let parent_id = column("parent_id");
        let foreign_key = parent_id
            .constraints
            .iter()
            .find(|constraint| constraint.kind == ConstraintKind::ForeignKey)
            .expect("composite foreign key should be attached to every source member");
        assert_eq!(foreign_key.source_columns, ["parent_id", "tenant"]);
        assert_eq!(
            foreign_key.target_schema.as_deref(),
            Some("videre_structure_test")
        );
        assert_eq!(foreign_key.target_relation.as_deref(), Some("target"));
        assert_eq!(foreign_key.target_columns, ["Target ID", "tenant"]);
        assert_eq!(foreign_key.on_update, Some(ForeignKeyAction::Cascade));
        assert_eq!(foreign_key.on_delete, Some(ForeignKeyAction::SetNull));

        let check = column("amount")
            .constraints
            .iter()
            .find(|constraint| constraint.kind == ConstraintKind::Check)
            .expect("check constraint should be attached to referenced columns");
        assert_eq!(check.source_columns, ["tenant", "amount"]);
        assert_eq!(check.column_position, None);
        assert!(check.definition.starts_with("CHECK"));

        for relation in ["Quoted View", "Quoted Materialized View"] {
            let structure = conn
                .relation_structure("videre_structure_test", relation)
                .await
                .expect("view structure should be readable");
            assert_eq!(structure.columns.len(), 2);
        }

        conn.client
            .batch_execute("DROP SCHEMA videre_structure_test CASCADE;")
            .await
            .expect("structure fixture should be removed");
    }
}
