use serde_json::Value;
use tokio_postgres::types::{FromSql, Type};
use tokio_postgres::{Column, Row};

pub fn quote_ident(ident: &str) -> Result<String, String> {
    if ident.contains('\0') {
        return Err(format!("Invalid identifier {ident:?}: contains a NUL byte"));
    }
    Ok(format!("\"{}\"", ident.replace('"', "\"\"")))
}

/// `schema.table`, both parts quoted.
pub fn qualified_name(schema: &str, table: &str) -> Result<String, String> {
    Ok(format!("{}.{}", quote_ident(schema)?, quote_ident(table)?))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Decoder {
    Bool,
    Int2,
    Int4,
    Int8,
    Float4,
    Float8,
    /// Already text on the wire, so no cast is needed.
    Text,
    /// Cast to `text` in the query and read back as a string. Postgres renders
    /// the value; we just carry it.
    AsText,
}

impl Decoder {
    fn for_type(ty: &Type) -> Self {
        if ty == &Type::BOOL {
            Self::Bool
        } else if ty == &Type::INT2 {
            Self::Int2
        } else if ty == &Type::INT4 {
            Self::Int4
        } else if ty == &Type::INT8 {
            Self::Int8
        } else if ty == &Type::FLOAT4 {
            Self::Float4
        } else if ty == &Type::FLOAT8 {
            Self::Float8
        } else if [Type::TEXT, Type::VARCHAR, Type::BPCHAR, Type::NAME].contains(ty) {
            Self::Text
        } else {
            // Timestamps, dates, times, intervals, numeric, uuid, json/jsonb,
            // arrays, bytea, enums, network types, ranges, domains, and any
            // extension type we have never seen. All of them have a text
            // representation Postgres knows how to produce.
            Self::AsText
        }
    }

    fn needs_cast(self) -> bool {
        matches!(self, Self::AsText)
    }
}

pub struct ColumnPlan {
    columns: Vec<(String, Decoder)>,
}

impl ColumnPlan {
    pub fn from_columns(columns: &[Column]) -> Self {
        Self {
            columns: columns
                .iter()
                .map(|c| (c.name().to_string(), Decoder::for_type(c.type_())))
                .collect(),
        }
    }

    /// `CREATE TABLE t ()` is legal Postgres, and an empty SELECT list is not —
    /// callers need to handle this before building a query.
    pub fn is_empty(&self) -> bool {
        self.columns.is_empty()
    }

    /// Column names in table order.
    pub fn names(&self) -> Vec<String> {
        self.columns.iter().map(|(name, _)| name.clone()).collect()
    }

    /// The SELECT list for the real query: a bare identifier where we decode
    /// natively, `"col"::text` where Postgres should do the formatting.
    pub fn select_list(&self) -> Result<String, String> {
        let mut parts = Vec::with_capacity(self.columns.len());
        for (name, decoder) in &self.columns {
            let ident = quote_ident(name)?;
            parts.push(if decoder.needs_cast() {
                format!("{ident}::text")
            } else {
                ident
            });
        }
        Ok(parts.join(", "))
    }

    /// Decode one row into JSON values, in the same order as [`names`].
    ///
    /// [`names`]: ColumnPlan::names
    pub fn decode_row(&self, row: &Row) -> Vec<Value> {
        self.columns
            .iter()
            .enumerate()
            .map(|(idx, (_, decoder))| decode_cell(row, idx, *decoder))
            .collect()
    }
}

fn decode_cell(row: &Row, idx: usize, decoder: Decoder) -> Value {
    let decoded = match decoder {
        Decoder::Bool => read::<bool>(row, idx).map(|v| v.map(Value::Bool)),
        Decoder::Int2 => read::<i16>(row, idx).map(|v| v.map(|n| Value::Number(n.into()))),
        Decoder::Int4 => read::<i32>(row, idx).map(|v| v.map(|n| Value::Number(n.into()))),
        Decoder::Int8 => read::<i64>(row, idx).map(|v| v.map(|n| Value::Number(n.into()))),
        Decoder::Float4 => read::<f32>(row, idx).map(|v| v.map(float_to_json)),
        Decoder::Float8 => read::<f64>(row, idx).map(|v| v.map(float_to_json)),
        Decoder::Text | Decoder::AsText => read::<String>(row, idx).map(|v| v.map(Value::String)),
    };

    match decoded {
        Ok(Some(value)) => value,
        // A real SQL NULL. Now the only thing that produces `null`.
        Ok(None) => Value::Null,
        // Unreachable: the decoder was picked from the type the server reported
        // for this very column. Say so in the cell anyway — falling back to
        // `null` here would recreate exactly the bug this module exists to fix.
        Err(e) => Value::String(format!("<unreadable: {e}>")),
    }
}

fn read<'a, T: FromSql<'a>>(row: &'a Row, idx: usize) -> Result<Option<T>, String> {
    row.try_get::<_, Option<T>>(idx).map_err(|e| e.to_string())
}

/// Postgres floats include `NaN`, `Infinity` and `-Infinity`, none of which JSON
/// can express as a number — emit those as text instead of collapsing them to
/// null. Finite values go through their shortest decimal form, so a `real`
/// column reads as `0.1` rather than `0.10000000149011612`.
fn float_to_json<F>(f: F) -> Value
where
    F: std::fmt::Display + Into<f64> + Copy,
{
    let text = f.to_string();
    if f.into().is_finite() {
        if let Ok(value @ Value::Number(_)) = serde_json::from_str(&text) {
            return value;
        }
    }
    Value::String(text)
}

#[cfg(test)]
mod tests {
    use super::*;

    impl ColumnPlan {
        fn from_parts(parts: &[(&str, Decoder)]) -> Self {
            Self {
                columns: parts
                    .iter()
                    .map(|(name, decoder)| (name.to_string(), *decoder))
                    .collect(),
            }
        }
    }

    #[test]
    fn quotes_plain_identifiers() {
        assert_eq!(quote_ident("created_at").unwrap(), "\"created_at\"");
        assert_eq!(quote_ident("Mixed Case").unwrap(), "\"Mixed Case\"");
    }

    #[test]
    fn doubles_embedded_quotes() {
        // Wrapping alone would let the payload escape into SQL.
        assert_eq!(quote_ident("we\"ird").unwrap(), "\"we\"\"ird\"");
        assert_eq!(
            quote_ident("x\" ; DROP TABLE t; --").unwrap(),
            "\"x\"\" ; DROP TABLE t; --\""
        );
    }

    #[test]
    fn rejects_nul_in_identifier() {
        assert!(quote_ident("bad\0name").is_err());
    }

    #[test]
    fn qualifies_and_quotes_both_parts() {
        assert_eq!(
            qualified_name("public", "gods").unwrap(),
            "\"public\".\"gods\""
        );
    }

    #[test]
    fn timestamps_are_cast_to_text_not_dropped() {
        // The regression that motivated this module: every `created_at` in
        // init-db.sql is `timestamptz`, and the old try_get chain had no arm for
        // it, so it fell through to Value::Null.
        assert_eq!(Decoder::for_type(&Type::TIMESTAMPTZ), Decoder::AsText);
        assert_eq!(Decoder::for_type(&Type::TIMESTAMP), Decoder::AsText);
        assert!(Decoder::for_type(&Type::TIMESTAMPTZ).needs_cast());
    }

    #[test]
    fn numeric_goes_to_text_to_keep_its_precision() {
        // Decoding via f64 would silently round a numeric(38,10).
        assert_eq!(Decoder::for_type(&Type::NUMERIC), Decoder::AsText);
    }

    #[test]
    fn previously_unsupported_types_all_have_a_decoder() {
        for ty in [
            Type::DATE,
            Type::TIME,
            Type::INTERVAL,
            Type::JSON,
            Type::JSONB,
            Type::BYTEA,
            Type::UUID,
            Type::INET,
            Type::MONEY,
            Type::TEXT_ARRAY,
            Type::INT4_ARRAY,
        ] {
            assert_eq!(
                Decoder::for_type(&ty),
                Decoder::AsText,
                "{ty} should be rendered by the server, not dropped"
            );
        }
    }

    #[test]
    fn native_types_are_decoded_without_a_cast() {
        for (ty, expected) in [
            (Type::BOOL, Decoder::Bool),
            (Type::INT2, Decoder::Int2),
            (Type::INT4, Decoder::Int4),
            (Type::INT8, Decoder::Int8),
            (Type::FLOAT4, Decoder::Float4),
            (Type::FLOAT8, Decoder::Float8),
            (Type::TEXT, Decoder::Text),
            (Type::VARCHAR, Decoder::Text),
            (Type::BPCHAR, Decoder::Text),
            (Type::NAME, Decoder::Text),
        ] {
            assert_eq!(Decoder::for_type(&ty), expected, "for {ty}");
            assert!(!Decoder::for_type(&ty).needs_cast(), "for {ty}");
        }
    }

    #[test]
    fn select_list_casts_only_what_needs_it() {
        let plan = ColumnPlan::from_parts(&[
            ("id", Decoder::Int4),
            ("name", Decoder::Text),
            ("created_at", Decoder::AsText),
        ]);
        assert_eq!(
            plan.select_list().unwrap(),
            "\"id\", \"name\", \"created_at\"::text"
        );
        assert_eq!(plan.names(), vec!["id", "name", "created_at"]);
    }

    #[test]
    fn select_list_escapes_column_names() {
        let plan = ColumnPlan::from_parts(&[("we\"ird", Decoder::AsText)]);
        assert_eq!(plan.select_list().unwrap(), "\"we\"\"ird\"::text");
    }

    #[test]
    fn empty_relation_is_detectable() {
        assert!(ColumnPlan::from_parts(&[]).is_empty());
        assert!(!ColumnPlan::from_parts(&[("id", Decoder::Int4)]).is_empty());
    }

    #[test]
    fn floats_keep_their_shortest_form() {
        assert_eq!(float_to_json(0.1f32), Value::from(0.1));
        assert_eq!(float_to_json(0.1f64), Value::from(0.1));
        assert_eq!(float_to_json(-2.5f64), Value::from(-2.5));
    }

    #[test]
    fn non_finite_floats_become_text_not_null() {
        assert_eq!(float_to_json(f64::NAN), Value::String("NaN".into()));
        assert_eq!(float_to_json(f64::INFINITY), Value::String("inf".into()));
        assert_eq!(
            float_to_json(f32::NEG_INFINITY),
            Value::String("-inf".into())
        );
    }
}
