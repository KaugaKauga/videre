use serde_json::Value;
use tokio_postgres::types::{FromSql, ToSql, Type};
use tokio_postgres::{Column, Row};
use uuid::Uuid;

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

struct PlannedColumn {
    name: String,
    ty: Type,
    decoder: Decoder,
}

pub struct ColumnPlan {
    columns: Vec<PlannedColumn>,
}

impl ColumnPlan {
    pub fn from_columns(columns: &[Column]) -> Self {
        Self {
            columns: columns
                .iter()
                .map(|c| PlannedColumn {
                    name: c.name().to_string(),
                    ty: c.type_().clone(),
                    decoder: Decoder::for_type(c.type_()),
                })
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
        self.columns.iter().map(|c| c.name.clone()).collect()
    }

    /// How to compare `column` against a bound key value, or `None` if the
    /// relation has no column by that name.
    pub fn key_predicate(&self, column: &str) -> Option<KeyPredicate> {
        self.columns
            .iter()
            .find(|c| c.name == column)
            .map(|c| KeyPredicate::for_type(&c.ty))
    }

    /// The SELECT list for the real query: a bare identifier where we decode
    /// natively, `"col"::text` where Postgres should do the formatting.
    pub fn select_list(&self) -> Result<String, String> {
        let mut parts = Vec::with_capacity(self.columns.len());
        for column in &self.columns {
            let ident = quote_ident(&column.name)?;
            parts.push(if column.decoder.needs_cast() {
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
            .map(|(idx, column)| decode_cell(row, idx, column.decoder))
            .collect()
    }
}

/// How a key column is compared against a bound parameter.
///
/// Postgres infers `$1`'s type from the comparison, so `"id" = $1` on a `bigint`
/// column means the driver must send an `int8` — sending anything else is a
/// serialization error, not a coercion. The parameter type therefore has to come
/// from the column, never from the shape of the incoming JSON.
pub enum KeyPredicate {
    /// Bind a parameter of the column's own type. Uses the index.
    Typed(Type),
    /// Compare the column's text form. Correct for any type at all, but it
    /// cannot use the index, so it is reserved for types we can't bind natively
    /// (`numeric`, dates, enums, domains). Both directions go through the
    /// server's text form, so values round-trip exactly.
    AsText,
}

impl KeyPredicate {
    fn for_type(ty: &Type) -> Self {
        if [
            Type::BOOL,
            Type::INT2,
            Type::INT4,
            Type::INT8,
            Type::FLOAT4,
            Type::FLOAT8,
            Type::TEXT,
            Type::VARCHAR,
            Type::BPCHAR,
            Type::NAME,
            Type::UUID,
        ]
        .contains(ty)
        {
            Self::Typed(ty.clone())
        } else {
            Self::AsText
        }
    }

    /// The left-hand side of the `= $1` comparison.
    pub fn lhs(&self, quoted_ident: &str) -> String {
        match self {
            Self::Typed(_) => quoted_ident.to_string(),
            Self::AsText => format!("{quoted_ident}::text"),
        }
    }
}

/// Turn a JSON key value from the frontend into a parameter the column accepts.
///
/// `Send` as well as `Sync` because the boxed parameter is held across the query
/// `await`, inside a future Tauri requires to be `Send`.
pub fn bind_key(
    value: &Value,
    predicate: &KeyPredicate,
) -> Result<Box<dyn ToSql + Sync + Send>, String> {
    if value.is_null() {
        return Err("Key value cannot be null".to_string());
    }

    let ty = match predicate {
        KeyPredicate::AsText => return Ok(Box::new(as_text(value))),
        KeyPredicate::Typed(ty) => ty,
    };

    if ty == &Type::BOOL {
        Ok(Box::new(as_bool(value)?))
    } else if ty == &Type::INT2 {
        Ok(Box::new(
            as_int(value, i16::MIN.into(), i16::MAX.into())? as i16
        ))
    } else if ty == &Type::INT4 {
        Ok(Box::new(
            as_int(value, i32::MIN.into(), i32::MAX.into())? as i32
        ))
    } else if ty == &Type::INT8 {
        Ok(Box::new(as_int(value, i64::MIN, i64::MAX)?))
    } else if ty == &Type::FLOAT4 {
        Ok(Box::new(as_float(value)? as f32))
    } else if ty == &Type::FLOAT8 {
        Ok(Box::new(as_float(value)?))
    } else if ty == &Type::UUID {
        Ok(Box::new(as_uuid(value)?))
    } else if [Type::TEXT, Type::VARCHAR, Type::BPCHAR, Type::NAME].contains(ty) {
        Ok(Box::new(as_string(value)?))
    } else {
        // Unreachable: `for_type` only returns Typed for the types above.
        Err(format!("Cannot bind a {ty} key value"))
    }
}

/// Accepts a JSON number or a numeric string. The string form matters because
/// JSON numbers are `f64` in the webview, so a `bigint` beyond 2^53 can only
/// reach us losslessly as text.
fn as_int(value: &Value, min: i64, max: i64) -> Result<i64, String> {
    let n = match value {
        Value::Number(n) => n
            .as_i64()
            .ok_or_else(|| format!("Key value {n} is not a whole number"))?,
        Value::String(s) => s
            .trim()
            .parse::<i64>()
            .map_err(|_| format!("Key value {s:?} is not a whole number"))?,
        other => return Err(format!("Expected an integer key value, got {other}")),
    };
    // Reported rather than truncated with `as` — a silently wrapped key would
    // fetch the wrong row.
    if n < min || n > max {
        return Err(format!("Key value {n} is out of range for this column"));
    }
    Ok(n)
}

fn as_float(value: &Value) -> Result<f64, String> {
    match value {
        Value::Number(n) => n
            .as_f64()
            .ok_or_else(|| format!("Key value {n} is not a number")),
        Value::String(s) => s
            .trim()
            .parse()
            .map_err(|_| format!("Key value {s:?} is not a number")),
        other => Err(format!("Expected a numeric key value, got {other}")),
    }
}

fn as_bool(value: &Value) -> Result<bool, String> {
    match value {
        Value::Bool(b) => Ok(*b),
        // `t`/`f` is what the server's text form produces.
        Value::String(s) => match s.trim() {
            "true" | "t" => Ok(true),
            "false" | "f" => Ok(false),
            _ => Err(format!("Key value {s:?} is not a boolean")),
        },
        other => Err(format!("Expected a boolean key value, got {other}")),
    }
}

fn as_uuid(value: &Value) -> Result<Uuid, String> {
    match value {
        Value::String(s) => s
            .trim()
            .parse()
            .map_err(|_| format!("Key value {s:?} is not a UUID")),
        other => Err(format!("Expected a UUID key value, got {other}")),
    }
}

fn as_string(value: &Value) -> Result<String, String> {
    match value {
        Value::String(s) => Ok(s.clone()),
        other => Err(format!("Expected a text key value, got {other}")),
    }
}

/// The `::text` comparison path: whatever the read side rendered, verbatim.
fn as_text(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        other => other.to_string(),
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
        fn from_parts(parts: &[(&str, Type)]) -> Self {
            Self {
                columns: parts
                    .iter()
                    .map(|(name, ty)| PlannedColumn {
                        name: name.to_string(),
                        ty: ty.clone(),
                        decoder: Decoder::for_type(ty),
                    })
                    .collect(),
            }
        }
    }

    fn typed(ty: Type) -> KeyPredicate {
        KeyPredicate::for_type(&ty)
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
            ("id", Type::INT4),
            ("name", Type::TEXT),
            ("created_at", Type::TIMESTAMPTZ),
        ]);
        assert_eq!(
            plan.select_list().unwrap(),
            "\"id\", \"name\", \"created_at\"::text"
        );
        assert_eq!(plan.names(), vec!["id", "name", "created_at"]);
    }

    #[test]
    fn select_list_escapes_column_names() {
        let plan = ColumnPlan::from_parts(&[("we\"ird", Type::TIMESTAMPTZ)]);
        assert_eq!(plan.select_list().unwrap(), "\"we\"\"ird\"::text");
    }

    #[test]
    fn empty_relation_is_detectable() {
        assert!(ColumnPlan::from_parts(&[]).is_empty());
        assert!(!ColumnPlan::from_parts(&[("id", Type::INT4)]).is_empty());
    }

    #[test]
    fn key_predicate_comes_from_the_column_not_the_value() {
        let plan = ColumnPlan::from_parts(&[("id", Type::INT8), ("code", Type::NUMERIC)]);

        // Indexable types compare directly...
        assert!(matches!(
            plan.key_predicate("id"),
            Some(KeyPredicate::Typed(_))
        ));
        // ...anything else falls back to the text form.
        assert!(matches!(
            plan.key_predicate("code"),
            Some(KeyPredicate::AsText)
        ));
        // An unknown column is caught before it reaches SQL.
        assert!(plan.key_predicate("nope").is_none());
    }

    #[test]
    fn predicate_lhs_only_casts_when_it_must() {
        assert_eq!(typed(Type::INT8).lhs("\"id\""), "\"id\"");
        assert_eq!(typed(Type::UUID).lhs("\"id\""), "\"id\"");
        assert_eq!(typed(Type::NUMERIC).lhs("\"id\""), "\"id\"::text");
        assert_eq!(typed(Type::DATE).lhs("\"d\""), "\"d\"::text");
    }

    #[test]
    fn every_integer_width_binds() {
        // The regression: a bigint key used to be cast to i32 and rejected by the
        // driver as `error serializing parameter 0`, whatever its value.
        for ty in [Type::INT2, Type::INT4, Type::INT8] {
            assert!(
                bind_key(&Value::from(42), &typed(ty.clone())).is_ok(),
                "{ty} should bind"
            );
        }
        assert!(bind_key(
            &Value::from(9_223_372_036_854_775_807i64),
            &typed(Type::INT8)
        )
        .is_ok());
    }

    #[test]
    fn out_of_range_keys_are_reported_not_truncated() {
        // 2^31 would wrap to a negative i32 under `as i32` and fetch a wrong row.
        let err = bind_key(&Value::from(2_147_483_648i64), &typed(Type::INT4)).unwrap_err();
        assert!(err.contains("out of range"), "{err}");

        let err = bind_key(&Value::from(40_000), &typed(Type::INT2)).unwrap_err();
        assert!(err.contains("out of range"), "{err}");
    }

    #[test]
    fn integers_also_accept_their_string_form() {
        // JSON numbers are f64 in the webview, so a bigint past 2^53 can only
        // arrive losslessly as text.
        assert!(bind_key(&Value::from("9007199254740993"), &typed(Type::INT8)).is_ok());
        assert!(bind_key(&Value::from("not a number"), &typed(Type::INT8)).is_err());
    }

    #[test]
    fn uuid_shaped_text_binds_as_text_when_the_column_is_text() {
        let uuid = Value::from("0b7f2c1e-4a5d-4f8e-9c3a-1d2e3f4a5b6c");
        // A `text` column holding UUID-shaped strings used to be bound as a
        // `uuid` parameter and rejected.
        assert!(bind_key(&uuid, &typed(Type::TEXT)).is_ok());
        // A real uuid column still binds as a uuid.
        assert!(bind_key(&uuid, &typed(Type::UUID)).is_ok());
        assert!(bind_key(&Value::from("nope"), &typed(Type::UUID)).is_err());
    }

    #[test]
    fn mismatched_and_null_keys_are_rejected_clearly() {
        assert!(bind_key(&Value::Null, &typed(Type::INT4))
            .unwrap_err()
            .contains("cannot be null"));
        assert!(bind_key(&Value::from(true), &typed(Type::INT4)).is_err());
        assert!(bind_key(&Value::from(1.5), &typed(Type::INT8)).is_err());
        assert!(bind_key(&Value::from(7), &typed(Type::TEXT)).is_err());
    }

    #[test]
    fn text_fallback_carries_the_servers_own_rendering() {
        // Reads go through `::text`, so the value coming back is already the
        // server's form and must be compared verbatim.
        assert!(bind_key(&Value::from("12345.6700"), &KeyPredicate::AsText).is_ok());
        assert!(bind_key(&Value::from("2024-01-15"), &KeyPredicate::AsText).is_ok());
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
