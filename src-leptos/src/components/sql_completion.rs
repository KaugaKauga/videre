//! Schema-only completion for a single, flat SELECT statement. No values or keywords.
//!
//! Sources are discovered from the entire current statement, including after the caret.
//! CTEs, subqueries, set operations, derived tables and table functions are deliberately
//! unsupported: suppress completion rather than infer a scope or expose outer columns.
//! The editor must guard IME composition and translate replacement bytes to DOM UTF-16.

use std::{collections::HashMap, ops::Range};

use super::sql_highlight::{
    dollar_delimiter_end, is_identifier_continue, is_identifier_start, is_keyword, next_char,
    skip_block_comment, skip_quoted,
};
use crate::types::{RelationStructure, RelationStructureKey, TableInfo};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionContext {
    /// UTF-8 bytes covering the entire identifier, including any suffix after the caret.
    pub replacement: Range<usize>,
    /// Decoded identifier text before the caret (without surrounding SQL quotes).
    pub prefix: String,
    kind: CompletionKind,
    quoted: bool,
    qualifier: Vec<Identifier>,
    sources: Vec<Source>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CompletionCandidate {
    pub label: String,
    pub insertion: String,
    pub detail: String,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum CompletionKind {
    Tables,
    Columns,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Identifier {
    // Unquoted names use PostgreSQL's ASCII folding; quoted names retain their case.
    name: String,
    quoted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct Source {
    schema: Option<Identifier>,
    table: Identifier,
    alias: Option<Identifier>,
}

#[derive(Debug)]
struct Token {
    range: Range<usize>,
    kind: TokenKind,
}

#[derive(Debug)]
enum TokenKind {
    Identifier(Identifier),
    Symbol(char),
    Number,
    Opaque,
}

impl Token {
    fn identifier(&self) -> Option<&Identifier> {
        match &self.kind {
            TokenKind::Identifier(identifier) => Some(identifier),
            _ => None,
        }
    }

    fn keyword(&self, keyword: &str) -> bool {
        self.identifier()
            .is_some_and(|id| !id.quoted && id.name.eq_ignore_ascii_case(keyword))
    }

    fn symbol(&self, symbol: char) -> bool {
        matches!(self.kind, TokenKind::Symbol(ch) if ch == symbol)
    }

    fn any_keyword(&self, keywords: &[&str]) -> bool {
        keywords.iter().any(|keyword| self.keyword(keyword))
    }

    fn source_boundary(&self) -> bool {
        // Highlighted types and nonreserved words can still be relation/alias names.
        self.any_keyword(FROM_LIST_ENDS)
            || self.any_keyword(CLAUSE_ENDS)
            || self.any_keyword(&[
                "as",
                "from",
                "join",
                "by",
                "left",
                "right",
                "full",
                "inner",
                "outer",
                "cross",
                "natural",
                "tablesample",
                "only",
                "lateral",
            ])
    }
}

/// Clauses after a FROM list that start a column context of their own.
const FROM_LIST_ENDS: &[&str] = &["where", "on", "group", "order", "having"];
/// Clauses that end both a FROM list and any column context.
const CLAUSE_ENDS: &[&str] = &["limit", "offset", "fetch", "for", "window", "using"];

struct Region {
    range: Range<usize>,
    closed: bool,
}

/// Clamps out-of-range bytes and rounds a byte inside a UTF-8 character down.
pub fn byte_to_utf16(sql: &str, byte: usize) -> u32 {
    let mut byte = byte.min(sql.len());
    while !sql.is_char_boundary(byte) {
        byte -= 1;
    }
    sql[..byte].chars().fold(0u32, |offset, ch| {
        offset.saturating_add(ch.len_utf16() as u32)
    })
}

/// Clamps out-of-range offsets; an offset inside a surrogate pair rounds down.
pub fn utf16_to_byte(sql: &str, caret_utf16: u32) -> usize {
    let mut offset = 0u32;
    for (byte, ch) in sql.char_indices() {
        let next = offset.saturating_add(ch.len_utf16() as u32);
        if next > caret_utf16 {
            return byte;
        }
        offset = next;
    }
    sql.len()
}

/// Suppresses completion on and at either boundary of unquoted clause/control keywords.
pub fn completion_context(sql: &str, caret_utf16: u32) -> Option<CompletionContext> {
    let caret = utf16_to_byte(sql, caret_utf16);
    let (tokens, regions) = scan(sql);
    if regions.iter().any(|region| {
        region.range.start <= caret
            && (caret < region.range.end || (caret == region.range.end && !region.closed))
    }) {
        return None;
    }

    let start = tokens
        .iter()
        .rposition(|token| token.symbol(';') && token.range.end <= caret)
        .map_or(0, |index| index + 1);
    let end = tokens[start..]
        .iter()
        .position(|token| token.symbol(';') && token.range.start >= caret)
        .map_or(tokens.len(), |index| start + index);
    let tokens = &tokens[start..end];
    if !tokens.first()?.keyword("select")
        || tokens.iter().enumerate().any(|(index, token)| {
            (index != 0 && token.keyword("select"))
                || ["with", "union", "intersect", "except", "into", "exists"]
                    .iter()
                    .any(|keyword| token.keyword(keyword))
        })
    {
        return None;
    }
    let sources = discover_sources(tokens)?;

    let active = tokens.iter().position(|token| {
        token.identifier().is_some() && token.range.start <= caret && caret <= token.range.end
    });
    let (replacement, prefix, quoted, before) = if let Some(index) = active {
        let token = &tokens[index];
        // Keyword boundaries are not identifier edits; exact-range replacement would
        // remove the clause, and zero-width insertion would need spacing rules.
        if token.source_boundary()
            || [
                "select", "distinct", "all", "and", "or", "not", "when", "then", "else", "between",
            ]
            .iter()
            .any(|keyword| token.keyword(keyword))
        {
            return None;
        }
        let identifier = token.identifier()?;
        // Do not replace a function name or the earlier part of a dotted reference.
        if tokens
            .get(index + 1)
            .is_some_and(|next| next.symbol('.') || next.symbol('('))
        {
            return None;
        }
        let prefix = if identifier.quoted {
            let end = caret.min(token.range.end - 1).max(token.range.start + 1);
            sql[token.range.start + 1..end].replace("\"\"", "\"")
        } else {
            sql[token.range.start..caret].to_string()
        };
        (token.range.clone(), prefix, identifier.quoted, index)
    } else {
        if tokens.iter().any(|token| {
            token.range.start <= caret
                && (caret < token.range.end
                    || (caret == token.range.end && matches!(token.kind, TokenKind::Number)))
        }) {
            return None;
        }
        (
            caret..caret,
            String::new(),
            false,
            tokens.partition_point(|token| token.range.end <= caret),
        )
    };

    let mut base = before;
    let mut qualifier = Vec::new();
    while base > 0 && tokens[base - 1].symbol('.') {
        let identifier = tokens.get(base.checked_sub(2)?)?.identifier()?.clone();
        qualifier.push(identifier);
        base -= 2;
    }
    qualifier.reverse();
    if qualifier.len() > 2 {
        return None;
    }

    let kind = clause_at(&tokens[..base])?;
    let previous = tokens.get(base.checked_sub(1)?)?;
    match kind {
        CompletionKind::Tables => {
            if qualifier.len() > 1
                || !(previous.keyword("from") || previous.keyword("join") || previous.symbol(','))
            {
                return None;
            }
        }
        CompletionKind::Columns => {
            // In a column context FROM is `IS DISTINCT FROM` or `extract(x FROM ...)`.
            let expression_start = [
                "select", "where", "on", "having", "by", "distinct", "all", "and", "or", "not",
                "when", "then", "else", "between", "from",
            ]
            .iter()
            .any(|keyword| previous.keyword(keyword))
                || previous.symbol(',')
                || previous.symbol('(');
            let comparison = (!prefix.is_empty() || !qualifier.is_empty() || quoted)
                && comparison_before(sql, tokens, base);
            if !expression_start && !comparison {
                return None;
            }
        }
    }

    Some(CompletionContext {
        replacement,
        prefix,
        kind,
        quoted,
        qualifier,
        sources,
    })
}

/// The unique catalog relations needed for columns, in stable schema/name order.
/// Ambiguous unqualified table names do not resolve, regardless of a likely search_path.
pub fn required_relations(
    ctx: &CompletionContext,
    tables: &[TableInfo],
) -> Vec<RelationStructureKey> {
    let mut keys: Vec<_> = selected_sources(ctx, tables)
        .into_iter()
        .map(|(_, key)| key)
        .collect();
    keys.sort_by(|a, b| (&a.schema, &a.relation).cmp(&(&b.schema, &b.relation)));
    keys.dedup();
    keys
}

/// At most twelve schema candidates. Missing structures yield no columns for that source.
pub fn suggestions(
    ctx: &CompletionContext,
    tables: &[TableInfo],
    structures: &HashMap<RelationStructureKey, RelationStructure>,
) -> Vec<CompletionCandidate> {
    let mut candidates = Vec::new();
    match ctx.kind {
        CompletionKind::Tables => {
            for table in tables {
                if !matches_prefix(&table.name, ctx)
                    || ctx
                        .qualifier
                        .first()
                        .is_some_and(|schema| schema.name != table.schema)
                {
                    continue;
                }
                let insertion = if ctx.qualifier.is_empty() {
                    format!(
                        "{}.{}",
                        quote_identifier(&table.schema),
                        quote_identifier(&table.name)
                    )
                } else {
                    quote_identifier(&table.name)
                };
                candidates.push(CompletionCandidate {
                    label: format!("{}.{}", table.schema, table.name),
                    insertion,
                    detail: "table".into(),
                });
            }
        }
        CompletionKind::Columns => {
            let selected = selected_sources(ctx, tables);
            for (source, key) in &selected {
                let Some(structure) = structures.get(key) else {
                    continue;
                };
                let qualify = ctx.qualifier.is_empty() && ctx.sources.len() > 1;
                let reference = if let Some(alias) = &source.alias {
                    quote_identifier(&alias.name)
                } else if ctx
                    .sources
                    .iter()
                    .filter(|other| other.table.name == key.relation)
                    .count()
                    > 1
                {
                    format!(
                        "{}.{}",
                        quote_identifier(&key.schema),
                        quote_identifier(&key.relation)
                    )
                } else {
                    quote_identifier(&key.relation)
                };
                // Duplicate aliases or an unaliased self-join cannot be named uniquely.
                if qualify
                    && ctx
                        .sources
                        .iter()
                        .filter(|other| match (&source.alias, &other.alias) {
                            (Some(alias), Some(other_alias)) => alias.name == other_alias.name,
                            (None, None) => {
                                resolve_source(other, tables).as_ref() == Some(key)
                                    || (other.schema.is_none() && other.table.name == key.relation)
                            }
                            (Some(alias), None) => alias.name == other.table.name,
                            (None, Some(alias)) => key.relation == alias.name,
                        })
                        .count()
                        > 1
                {
                    continue;
                }
                for column in &structure.columns {
                    if !matches_prefix(&column.name, ctx) {
                        continue;
                    }
                    let insertion = if qualify {
                        format!("{reference}.{}", quote_identifier(&column.name))
                    } else {
                        quote_identifier(&column.name)
                    };
                    candidates.push(CompletionCandidate {
                        label: if qualify {
                            insertion.clone()
                        } else {
                            column.name.clone()
                        },
                        insertion,
                        detail: format!("{}.{} · {}", key.schema, key.relation, column.data_type),
                    });
                }
            }
        }
    }
    candidates.sort_by_cached_key(|candidate| {
        (
            candidate.label.to_ascii_lowercase(),
            candidate.label.clone(),
            candidate.insertion.clone(),
            candidate.detail.clone(),
        )
    });
    // Sorting includes all fields so catalog and HashMap iteration order cannot affect ties.
    candidates.dedup_by(|a, b| a.insertion == b.insertion);
    candidates.truncate(12);
    candidates
}

fn matches_prefix(name: &str, ctx: &CompletionContext) -> bool {
    if ctx.quoted {
        name.starts_with(&ctx.prefix)
    } else {
        name.to_ascii_lowercase()
            .starts_with(&ctx.prefix.to_ascii_lowercase())
    }
}

fn quote_identifier(name: &str) -> String {
    let plain = name.chars().next().is_some_and(is_identifier_start)
        && name.chars().all(is_identifier_continue)
        && !name.chars().any(char::is_uppercase)
        && !is_keyword(name)
        // PostgreSQL reserved names not included in the highlighter's common keyword list.
        && !matches!(name, "analyse" | "binary" | "system_user");
    if plain {
        name.into()
    } else {
        format!("\"{}\"", name.replace('"', "\"\""))
    }
}

fn resolve_source(source: &Source, tables: &[TableInfo]) -> Option<RelationStructureKey> {
    let mut matches = tables.iter().filter(|table| {
        source.table.name == table.name
            && source
                .schema
                .as_ref()
                .is_none_or(|schema| schema.name == table.schema)
    });
    let first = matches.next()?;
    if matches.any(|other| other.schema != first.schema || other.name != first.name) {
        return None;
    }
    Some(RelationStructureKey::new(&first.schema, &first.name))
}

fn selected_sources<'a>(
    ctx: &'a CompletionContext,
    tables: &[TableInfo],
) -> Vec<(&'a Source, RelationStructureKey)> {
    if ctx.kind != CompletionKind::Columns {
        return Vec::new();
    }
    let matching: Vec<_> = ctx
        .sources
        .iter()
        .filter(|source| match ctx.qualifier.as_slice() {
            [] => true,
            [reference] => source.alias.as_ref().map_or_else(
                || reference.name == source.table.name,
                |alias| reference.name == alias.name,
            ),
            [schema, relation] => {
                source.alias.is_none()
                    && relation.name == source.table.name
                    && source.schema.as_ref().map_or_else(
                        || {
                            resolve_source(source, tables)
                                .is_some_and(|key| key.schema == schema.name)
                        },
                        |source_schema| source_schema.name == schema.name,
                    )
            }
            _ => false,
        })
        .collect();
    // Count syntax-level matches before resolving the catalog: an unknown or
    // ambiguous relation must not make a duplicate alias appear unambiguous.
    if !ctx.qualifier.is_empty() && matching.len() != 1 {
        return Vec::new();
    }
    matching
        .into_iter()
        .filter_map(|source| resolve_source(source, tables).map(|key| (source, key)))
        .collect()
}

fn comparison_before(sql: &str, tokens: &[Token], before: usize) -> bool {
    let last = &tokens[before - 1];
    let mut start = last.range.start;
    for token in tokens[..before - 1].iter().rev() {
        if token.range.end != start
            || !matches!(
                token.kind,
                TokenKind::Symbol(
                    '+' | '-'
                        | '*'
                        | '/'
                        | '<'
                        | '>'
                        | '='
                        | '~'
                        | '!'
                        | '@'
                        | '#'
                        | '%'
                        | '^'
                        | '&'
                        | '|'
                        | '?'
                        | ':'
                )
            )
        {
            break;
        }
        start = token.range.start;
    }
    matches!(
        &sql[start..last.range.end],
        "=" | "<" | ">" | "<=" | ">=" | "<>" | "!="
    )
}

/// A FROM that starts a FROM list, not the operator `IS [NOT] DISTINCT FROM`.
fn starts_from_list(tokens: &[Token], index: usize) -> bool {
    tokens[index].keyword("from")
        && !(index >= 2
            && tokens[index - 1].keyword("distinct")
            && tokens[index - 2].any_keyword(&["is", "not"]))
}

fn clause_at(tokens: &[Token]) -> Option<CompletionKind> {
    let mut clause = None;
    let mut depth = 0usize;
    let mut awaiting_by = false;
    for (index, token) in tokens.iter().enumerate() {
        if token.symbol('(') {
            depth += 1;
        } else if token.symbol(')') {
            depth = depth.checked_sub(1)?;
        } else if depth == 0 {
            if token.keyword("select")
                || token.keyword("where")
                || token.keyword("on")
                || token.keyword("having")
            {
                clause = Some(CompletionKind::Columns);
                awaiting_by = false;
            } else if starts_from_list(tokens, index) || token.keyword("join") {
                clause = Some(CompletionKind::Tables);
                awaiting_by = false;
            } else if token.keyword("order") || token.keyword("group") {
                clause = None;
                awaiting_by = true;
            } else if token.keyword("by") && awaiting_by {
                clause = Some(CompletionKind::Columns);
                awaiting_by = false;
            } else if token.any_keyword(CLAUSE_ENDS) {
                clause = None;
                awaiting_by = false;
            }
        }
    }
    clause
}

fn discover_sources(tokens: &[Token]) -> Option<Vec<Source>> {
    let mut sources = Vec::new();
    let mut index = 0;
    let mut depth = 0usize;
    let mut in_from = false;
    while index < tokens.len() {
        let token = &tokens[index];
        if token.symbol('(') {
            depth += 1;
        } else if token.symbol(')') {
            depth = depth.checked_sub(1)?;
        } else if depth == 0 {
            if starts_from_list(tokens, index)
                || token.keyword("join")
                || (in_from && token.symbol(','))
            {
                in_from = true;
                index += 1;
                let Some(table) = tokens.get(index) else {
                    break;
                };
                if table.symbol('(') || table.keyword("only") || table.keyword("lateral") {
                    return None;
                }
                let identifier = table.identifier()?;
                if table.source_boundary() {
                    continue;
                }
                let mut source = Source {
                    schema: None,
                    table: identifier.clone(),
                    alias: None,
                };
                index += 1;
                if tokens.get(index).is_some_and(|token| token.symbol('.')) {
                    index += 1;
                    let Some(relation) = tokens.get(index).and_then(Token::identifier) else {
                        continue;
                    };
                    source.schema = Some(source.table);
                    source.table = relation.clone();
                    index += 1;
                }
                if tokens
                    .get(index)
                    .is_some_and(|token| token.symbol('(') || token.symbol('.'))
                {
                    return None;
                }
                if tokens.get(index).is_some_and(|token| token.keyword("as")) {
                    index += 1;
                }
                if let Some(token) = tokens.get(index).filter(|token| !token.source_boundary()) {
                    if let Some(alias) = token.identifier() {
                        source.alias = Some(alias.clone());
                        index += 1;
                    }
                }
                sources.push(source);
                continue;
            }
            if token.any_keyword(FROM_LIST_ENDS) || token.any_keyword(CLAUSE_ENDS) {
                in_from = false;
            }
        }
        index += 1;
    }
    Some(sources)
}

fn scan(sql: &str) -> (Vec<Token>, Vec<Region>) {
    let mut tokens = Vec::new();
    let mut regions = Vec::new();
    // The existing skip helpers return input length for an open region. A sentinel
    // distinguishes that result from a closing delimiter exactly at the real EOF,
    // without duplicating their quote escaping or nested-comment logic.
    let terminated = format!("{sql}\0");
    let mut cursor = 0;
    while cursor < sql.len() {
        let start = cursor;
        let ch = next_char(sql, cursor);
        if ch.is_ascii_whitespace() {
            cursor += ch.len_utf8();
            continue;
        }
        if sql[cursor..].starts_with("--") {
            cursor += 2;
            while cursor < sql.len() && !matches!(next_char(sql, cursor), '\n' | '\r') {
                cursor += next_char(sql, cursor).len_utf8();
            }
            // Inserting exactly before the newline still extends the line comment.
            regions.push(Region {
                range: start..cursor,
                closed: false,
            });
            continue;
        }
        if sql[cursor..].starts_with("/*") {
            let end = skip_block_comment(&terminated, cursor);
            cursor = end.min(sql.len());
            regions.push(Region {
                range: start..cursor,
                closed: end <= sql.len(),
            });
            continue;
        }
        if ch == '$' {
            if let Some(delimiter_end) = dollar_delimiter_end(sql, cursor) {
                let delimiter = &sql[cursor..delimiter_end];
                let closing = sql[delimiter_end..].find(delimiter);
                cursor =
                    closing.map_or(sql.len(), |offset| delimiter_end + offset + delimiter.len());
                regions.push(Region {
                    range: start..cursor,
                    closed: closing.is_some(),
                });
                tokens.push(Token {
                    range: start..cursor,
                    kind: TokenKind::Opaque,
                });
                continue;
            }
        }
        let kind = match ch {
            '\'' | '"' => {
                let end = skip_quoted(&terminated, cursor, ch, false);
                cursor = end.min(sql.len());
                if ch == '"' && end <= sql.len() {
                    TokenKind::Identifier(Identifier {
                        name: sql[start + 1..cursor - 1].replace("\"\"", "\""),
                        quoted: true,
                    })
                } else {
                    regions.push(Region {
                        range: start..cursor,
                        closed: end <= sql.len(),
                    });
                    TokenKind::Opaque
                }
            }
            _ if is_identifier_continue(ch) => {
                cursor += ch.len_utf8();
                while cursor < sql.len() && is_identifier_continue(next_char(sql, cursor)) {
                    cursor += next_char(sql, cursor).len_utf8();
                }
                let word = &sql[start..cursor];
                let escape = word.eq_ignore_ascii_case("e") && sql[cursor..].starts_with('\'');
                let unicode = word.eq_ignore_ascii_case("u")
                    && (sql[cursor..].starts_with("&'") || sql[cursor..].starts_with("&\""));
                if escape || unicode {
                    if unicode {
                        cursor += 1;
                    }
                    let end = skip_quoted(&terminated, cursor, next_char(sql, cursor), escape);
                    cursor = end.min(sql.len());
                    // U& identifiers need Unicode escape decoding, intentionally unsupported.
                    regions.push(Region {
                        range: start..cursor,
                        closed: end <= sql.len(),
                    });
                    TokenKind::Opaque
                } else if is_identifier_start(ch) {
                    TokenKind::Identifier(Identifier {
                        name: word.to_ascii_lowercase(),
                        quoted: false,
                    })
                } else {
                    TokenKind::Number
                }
            }
            _ => {
                cursor += ch.len_utf8();
                TokenKind::Symbol(ch)
            }
        };
        tokens.push(Token {
            range: start..cursor,
            kind,
        });
    }
    (tokens, regions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::ColumnInfo;

    fn marked(marked: &str) -> (String, u32) {
        let caret = marked.find('|').expect("caret marker");
        let sql = marked.replacen('|', "", 1);
        let utf16 = byte_to_utf16(&sql, caret);
        (sql, utf16)
    }

    fn context(marked_sql: &str) -> CompletionContext {
        let (sql, caret) = marked(marked_sql);
        completion_context(&sql, caret).unwrap_or_else(|| panic!("no context: {marked_sql}"))
    }

    fn catalog() -> (
        Vec<TableInfo>,
        HashMap<RelationStructureKey, RelationStructure>,
    ) {
        let tables = vec![
            TableInfo {
                schema: "public".into(),
                name: "creatures".into(),
            },
            TableInfo {
                schema: "public".into(),
                name: "owners".into(),
            },
            TableInfo {
                schema: "Fancy Schema".into(),
                name: "Zoo".into(),
            },
        ];
        let mut structures = HashMap::new();
        for (table, columns) in tables.iter().zip([
            vec![
                "id", "type", "title", "health", "Odd Name", "MiXeD", "select", "say\"hi", "🐾name",
            ],
            vec!["id", "name"],
            vec!["Case", "case", "quote\"name"],
        ]) {
            let key = RelationStructureKey::new(&table.schema, &table.name);
            structures.insert(
                key,
                RelationStructure {
                    schema: table.schema.clone(),
                    relation: table.name.clone(),
                    relation_kind: "table".into(),
                    columns: columns
                        .into_iter()
                        .enumerate()
                        .map(|(index, name)| ColumnInfo {
                            name: name.into(),
                            ordinal_position: index as i32 + 1,
                            data_type: "text".into(),
                            nullable: true,
                            collation: None,
                            default_expression: None,
                            identity: None,
                            generated: None,
                            generation_expression: None,
                            comment: None,
                            constraints: vec![],
                        })
                        .collect(),
                },
            );
        }
        (tables, structures)
    }

    fn complete(sql: &str) -> Vec<CompletionCandidate> {
        let (tables, structures) = catalog();
        let (sql, caret) = marked(sql);
        completion_context(&sql, caret)
            .map_or_else(Vec::new, |ctx| suggestions(&ctx, &tables, &structures))
    }

    fn insertions(sql: &str) -> Vec<String> {
        complete(sql)
            .into_iter()
            .map(|candidate| candidate.insertion)
            .collect()
    }

    #[test]
    fn tables_after_from_join_and_comma() {
        for sql in [
            "SELECT * FROM cr|",
            "select * from CR|",
            "SELECT * FROM owners JOIN cr|",
            "SELECT * FROM owners, cr|",
            "SELECT * FROM /* comment */ cr|",
        ] {
            assert_eq!(insertions(sql), ["public.creatures"], "{sql}");
            assert!(required_relations(&context(sql), &catalog().0).is_empty());
        }
        assert_eq!(complete("SELECT * FROM |").len(), 3);
        assert_eq!(insertions("SELECT * FROM public.cr|"), ["creatures"]);
        assert_eq!(
            insertions("SELECT * FROM public.|"),
            ["creatures", "owners"]
        );
        assert!(complete("SELECT * FROM missing.cr|").is_empty());
    }

    #[test]
    fn discovers_from_after_caret_and_supported_column_clauses() {
        for sql in [
            "SELECT ty| FROM creatures",
            "SELECT TY| FROM creatures",
            "SELECT id, ty| FROM creatures",
            "SELECT DISTINCT ty| FROM creatures",
            "SELECT * FROM creatures WHERE ty|",
            "SELECT * FROM creatures ORDER BY ty|",
            "SELECT * FROM creatures GROUP BY ty|",
            "SELECT * FROM creatures HAVING ty|",
            "SELECT * FROM creatures c JOIN owners o ON c.ty| = o.name",
            "SELECT * FROM creatures WHERE id > 1 AND ty| = 'x'",
            "SELECT coalesce(ty|, 'x') FROM creatures",
            "SELECT (ty|) FROM creatures",
        ] {
            assert_eq!(insertions(sql), ["type"], "{sql}");
        }
        assert!(!complete("SELECT * FROM creatures WHERE |").is_empty());
        assert!(!complete("SELECT | FROM creatures").is_empty());
        assert!(!complete("SELECT * FROM creatures ORDER BY |").is_empty());
    }

    #[test]
    fn distinct_from_operator_is_not_a_from_list() {
        for sql in [
            "SELECT ty| FROM creatures WHERE id IS DISTINCT FROM 5",
            "SELECT ty| FROM creatures WHERE id IS NOT DISTINCT FROM 5",
            "SELECT ty| FROM creatures WHERE id IS DISTINCT FROM health",
            "SELECT * FROM creatures WHERE id IS DISTINCT FROM ty|",
        ] {
            assert_eq!(insertions(sql), ["type"], "{sql}");
        }
    }

    #[test]
    fn clause_keywords_are_never_replacement_targets() {
        let cases: &[(&str, &[&str])] = &[
            ("SELECT FROM creatures", &["SELECT", "FROM"]),
            (
                "SELECT DISTINCT type FROM creatures WHERE health > 0 GROUP BY type HAVING type != '' ORDER BY type LIMIT 12 OFFSET 0",
                &["DISTINCT", "FROM", "WHERE", "GROUP", "BY", "HAVING", "ORDER", "LIMIT", "OFFSET"],
            ),
            ("SELECT ALL type FROM creatures", &["ALL"]),
            (
                "SELECT * FROM creatures c LEFT JOIN owners o ON c.id = o.id",
                &["LEFT", "JOIN", "ON"],
            ),
            (
                "SELECT * FROM creatures WHERE health > 0 AND id > 0 OR NOT id = 0",
                &["AND", "OR", "NOT"],
            ),
            ("select type fRoM creatures", &["fRoM"]),
            ("SELECT * FROM creatures c JOIN owners o USING (id)", &["USING"]),
        ];
        for &(sql, keywords) in cases {
            for &keyword in keywords {
                for (start, _) in sql.match_indices(keyword) {
                    for byte in start..=start + keyword.len() {
                        assert!(
                            completion_context(sql, byte_to_utf16(sql, byte)).is_none(),
                            "keyword {keyword} at byte {byte}: {sql}",
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn completing_before_from_preserves_keyword_with_exact_range_replacement() {
        let (tables, structures) = catalog();
        for marked_sql in ["SELECT h|ealth FROM creatures", "SELECT | FROM creatures"] {
            let (mut sql, caret) = marked(marked_sql);
            let ctx = completion_context(&sql, caret).unwrap();
            let candidate = suggestions(&ctx, &tables, &structures)
                .into_iter()
                .find(|candidate| candidate.insertion == "health")
                .unwrap();
            sql.replace_range(ctx.replacement, &candidate.insertion);
            assert_eq!(sql, "SELECT health FROM creatures");
        }
    }

    #[test]
    fn quoted_clause_names_remain_completable_identifiers() {
        let (tables, mut structures) = catalog();
        let structure = structures
            .get_mut(&RelationStructureKey::new("public", "creatures"))
            .unwrap();
        let mut column = structure.columns[0].clone();
        column.name = "from".into();
        structure.columns.push(column);
        for marked_sql in [
            "SELECT \"fr|om\" FROM creatures",
            "SELECT \"from|\" FROM creatures",
        ] {
            let (mut sql, caret) = marked(marked_sql);
            let ctx = completion_context(&sql, caret).unwrap();
            let candidate = suggestions(&ctx, &tables, &structures)
                .into_iter()
                .next()
                .unwrap();
            assert_eq!(candidate.insertion, "\"from\"");
            sql.replace_range(ctx.replacement, &candidate.insertion);
            assert_eq!(sql, "SELECT \"from\" FROM creatures");
        }
    }

    #[test]
    fn aliases_restrict_requests_and_replace_only_column() {
        let ctx = context("SELECT c.ty| FROM creatures AS c JOIN owners o ON c.id = o.id");
        assert_eq!(
            required_relations(&ctx, &catalog().0),
            [RelationStructureKey::new("public", "creatures")]
        );
        assert_eq!(
            insertions("SELECT c.ty| FROM creatures c JOIN owners o ON c.id = o.id"),
            ["type"]
        );
        assert_eq!(
            insertions("SELECT o.na| FROM creatures c JOIN owners o ON c.id = o.id"),
            ["name"]
        );
        assert!(!complete("SELECT c.| FROM creatures c").is_empty());
        assert!(!complete("SELECT * FROM creatures c JOIN owners o ON o.id = c.|").is_empty());
        assert!(complete("SELECT creatures.ty| FROM creatures c").is_empty());
        assert!(complete("SELECT missing.ty| FROM creatures c").is_empty());
        let (sql, _) = marked("SELECT c.ty|pe FROM creatures c");
        assert_eq!(&sql[ctx.replacement], "ty");
        let middle = context("SELECT c.ty|pe FROM creatures c");
        assert_eq!(&sql[middle.replacement], "type");
        assert_eq!(middle.prefix, "ty");
    }

    #[test]
    fn joined_columns_are_qualified_and_cache_keys_deduplicated() {
        assert_eq!(
            insertions("SELECT i| FROM creatures c JOIN owners o ON c.id = o.id"),
            ["c.id", "o.id"]
        );
        assert_eq!(
            insertions("SELECT i| FROM creatures JOIN owners ON creatures.id = owners.id"),
            ["creatures.id", "owners.id"]
        );
        assert_eq!(
            insertions("SELECT i| FROM creatures a, creatures b"),
            ["a.id", "b.id"]
        );
        // Locking clause lists are not FROM lists.
        for sql in [
            "SELECT i| FROM creatures FOR UPDATE OF creatures, owners",
            "SELECT i| FROM creatures FOR SHARE OF creatures, owners NOWAIT",
        ] {
            assert_eq!(
                insertions(sql),
                insertions("SELECT i| FROM creatures"),
                "{sql}"
            );
        }
        let keys = required_relations(
            &context("SELECT i| FROM creatures a JOIN creatures b ON a.id=b.id"),
            &catalog().0,
        );
        assert_eq!(keys, [RelationStructureKey::new("public", "creatures")]);
        assert!(complete("SELECT i| FROM creatures c JOIN owners c ON c.id=c.id").is_empty());
        assert!(complete("SELECT c.i| FROM creatures c JOIN owners c ON c.id=c.id").is_empty());
        assert!(
            complete("SELECT i| FROM creatures JOIN creatures ON creatures.id=creatures.id")
                .is_empty()
        );
    }

    #[test]
    fn unqualified_catalog_ambiguity_never_guesses_search_path() {
        let (mut tables, mut structures) = catalog();
        tables.push(TableInfo {
            schema: "archive".into(),
            name: "creatures".into(),
        });
        let mut other = structures[&RelationStructureKey::new("public", "creatures")].clone();
        other.schema = "archive".into();
        structures.insert(RelationStructureKey::new("archive", "creatures"), other);
        for sql in ["SELECT ty| FROM creatures", "SELECT c.ty| FROM creatures c"] {
            let ctx = context(sql);
            assert!(required_relations(&ctx, &tables).is_empty());
            assert!(suggestions(&ctx, &tables, &structures).is_empty());
        }
        let ctx = context("SELECT ty| FROM public.creatures");
        assert_eq!(suggestions(&ctx, &tables, &structures)[0].insertion, "type");
        let ctx = context("SELECT i| FROM public.creatures JOIN archive.creatures ON true");
        let result: Vec<_> = suggestions(&ctx, &tables, &structures)
            .into_iter()
            .map(|c| c.insertion)
            .collect();
        assert_eq!(result, ["archive.creatures.id", "public.creatures.id"]);
        assert!(suggestions(
            &context("SELECT creatures.i| FROM public.creatures, archive.creatures"),
            &tables,
            &structures
        )
        .is_empty());
        assert_eq!(
            suggestions(
                &context("SELECT public.creatures.ty| FROM public.creatures, archive.creatures"),
                &tables,
                &structures
            )[0]
            .insertion,
            "type"
        );
    }

    #[test]
    fn quoted_identifiers_and_aliases_preserve_actual_names() {
        assert_eq!(
            insertions("SELECT * FROM \"Fancy Schema\".\"Z|oo\""),
            ["\"Zoo\""]
        );
        assert_eq!(
            insertions("SELECT * FROM \"Fancy Schema\".\"z|oo\""),
            Vec::<String>::new()
        );
        assert_eq!(
            insertions("SELECT z.\"C|ase\" FROM \"Fancy Schema\".\"Zoo\" z"),
            ["\"Case\""]
        );
        assert_eq!(
            insertions("SELECT \"C\".ty| FROM creatures \"C\""),
            ["type"]
        );
        assert!(complete("SELECT c.ty| FROM creatures \"C\"").is_empty());
        assert_eq!(insertions("SELECT C.ty| FROM creatures C"), ["type"]);
        assert_eq!(
            insertions("SELECT \"a\"\"b\".ty| FROM creatures \"a\"\"b\""),
            ["type"]
        );
        assert_eq!(
            insertions("SELECT \"Fancy Schema\".\"Zoo\".\"C|ase\" FROM \"Fancy Schema\".\"Zoo\""),
            ["\"Case\""]
        );
        let ctx = context("SELECT \"say\"\"h|i\" FROM creatures");
        assert_eq!(ctx.prefix, "say\"h");
        assert_eq!(
            suggestions(&ctx, &catalog().0, &catalog().1)[0].insertion,
            "\"say\"\"hi\""
        );
    }

    #[test]
    fn insertion_quotes_case_spaces_embedded_quotes_and_reserved_names() {
        for (prefix, expected) in [
            ("mi", "\"MiXeD\""),
            ("odd", "\"Odd Name\""),
            ("sel", "\"select\""),
            ("say", "\"say\"\"hi\""),
            ("🐾", "🐾name"),
        ] {
            assert_eq!(
                insertions(&format!("SELECT {prefix}| FROM creatures")),
                [expected]
            );
        }
        assert!(insertions("SELECT * FROM |").contains(&"\"Fancy Schema\".\"Zoo\"".into()));
        for name in [
            "analyse",
            "binary",
            "system_user",
            "order",
            "1start",
            "",
            "has space",
            "a\"b",
            "MiXeD",
        ] {
            assert!(quote_identifier(name).starts_with('"'), "{name}");
        }
        assert_eq!(quote_identifier("lower_$9"), "lower_$9");
    }

    #[test]
    fn ignores_strings_comments_and_all_open_regions() {
        for sql in [
            "SELECT 'ty|' FROM creatures",
            "SELECT E'ty\\'| FROM creatures'",
            "SELECT 'ty''|' FROM creatures",
            "SELECT $$ty|$$ FROM creatures",
            "SELECT $tag$ty|$tag$ FROM creatures",
            "SELECT $标签$ty|$标签$ FROM creatures",
            "SELECT U&'ty|' FROM creatures",
            "SELECT U&\"ty|\" FROM creatures",
            "SELECT ty -- |\nFROM creatures",
            "SELECT ty -- |\rFROM creatures",
            "SELECT -- comment|\nFROM creatures",
            "SELECT -- comment|\r\nFROM creatures",
            "SELECT /* | */ ty FROM creatures",
            "SELECT /* outer /* | */ inner */ ty FROM creatures",
            "SELECT 'unterminated|",
            "SELECT 'doubled''|",
            "SELECT E'escaped\\'|",
            "SELECT \"unterminated|",
            "SELECT $$unterminated|",
            "SELECT $tag$unterminated|",
            "SELECT /* outer /* inner */|",
            "SELECT -- unfinished|",
        ] {
            let (text, caret) = marked(sql);
            assert!(completion_context(&text, caret).is_none(), "{sql}");
        }
        assert_eq!(
            insertions("SELECT ty| FROM creatures WHERE title = E'\\🐾\\'FROM owners'"),
            ["type"]
        );
        assert_eq!(
            insertions("SELECT ty| FROM creatures WHERE title = 'x\\'"),
            ["type"]
        );
        assert_eq!(
            insertions("SELECT ty| FROM creatures /* FROM owners */"),
            ["type"]
        );
        assert_eq!(
            insertions("SELECT ty| FROM creatures -- FROM owners\n"),
            ["type"]
        );
        assert_eq!(
            insertions("SELECT ty| FROM creatures WHERE title = $Tag$FROM owners $tag$ $Tag$"),
            ["type"]
        );
        assert_eq!(
            insertions("SELECT ty| FROM creatures WHERE title = 'FROM owners'"),
            ["type"]
        );
    }

    #[test]
    fn statement_boundaries_ignore_opaque_semicolons() {
        assert_eq!(
            insertions("SELECT * FROM owners; SELECT ty| FROM creatures; SELECT * FROM owners"),
            ["type"]
        );
        assert_eq!(
            insertions("SELECT ty| FROM creatures; SELECT * FROM owners"),
            ["type"]
        );
        assert!(complete("SELECT * FROM creatures; SELECT ty|").is_empty());
        assert_eq!(
            insertions("SELECT ty| FROM creatures WHERE title='; FROM owners' /* ; FROM owners */"),
            ["type"]
        );
        assert_eq!(
            insertions("SELECT ty| FROM creatures WHERE title=$$; FROM owners$$"),
            ["type"]
        );
        assert_eq!(
            insertions("SELECT * FROM creatures; SELECT na| FROM owners"),
            ["name"]
        );
        assert!(complete("SELECT * FROM creatures;|").is_empty());
        assert!(complete("SELECT * FROM creatures; -- |comment").is_empty());
    }

    #[test]
    fn suppresses_unsupported_scopes_instead_of_leaking_sources() {
        for sql in [
            "WITH c AS (SELECT * FROM creatures) SELECT ty| FROM c",
            "SELECT ty| FROM (SELECT * FROM creatures) c",
            "SELECT ty| FROM creatures WHERE EXISTS (SELECT * FROM owners)",
            "SELECT * FROM creatures WHERE EXISTS (SELECT na| FROM owners)",
            "SELECT * FROM creatures WHERE EXISTS (ty|",
            "SELECT ty| FROM creatures UNION SELECT name FROM owners",
            "SELECT ty| FROM creatures INTERSECT SELECT name FROM owners",
            "SELECT ty| FROM creatures EXCEPT SELECT name FROM owners",
            "SELECT ty| FROM generate_series(1, 10) c",
            "SELECT ty| FROM (creatures c)",
            "SELECT ty| FROM ONLY creatures",
            "SELECT ty| FROM creatures JOIN LATERAL owners o ON true",
            "SELECT ty| INTO result FROM creatures",
            "UPDATE creatures SET ty| = 1",
        ] {
            let (text, caret) = marked(sql);
            assert!(completion_context(&text, caret).is_none(), "{sql}");
        }
    }

    #[test]
    fn numeric_path_operators_alias_and_noncolumn_contexts_are_suppressed() {
        for sql in [
            "SELECT 12| FROM creatures",
            "SELECT 12.| FROM creatures",
            "SELECT /tmp/ty| FROM creatures",
            "SELECT +| FROM creatures",
            "SELECT * FROM creatures WHERE id = |",
            "SELECT * FROM creatures WHERE id = 42|",
            "SELECT * FROM creatures WHERE title = 'x' |",
            "SELECT id::te| FROM creatures",
            "SELECT id ->> ty| FROM creatures",
            "SELECT id #>> ty| FROM creatures",
            "SELECT id == ty| FROM creatures",
            "SELECT id @> ty| FROM creatures",
            "SELECT ty()| FROM creatures",
            "SELECT id AS ty| FROM creatures",
            "SELECT * FROM creatures c|",
            "SELECT * FROM creatures ORDER |",
            "SELECT * FROM creatures ORDER BY id DESC |",
            "SELECT * FROM creatures LIMIT ty|",
            "SELECT * FROM creatures OFFSET |",
            "SELECT * FROM creatures JOIN owners USING (i|)",
            "SELECT foo..ty| FROM creatures",
            "SELECT a.b.c.ty| FROM creatures",
            "SELECT * FROM pub|lic.creatures",
        ] {
            assert!(complete(sql).is_empty(), "{sql}");
        }
        for operator in ["=", "<", ">", "<=", ">=", "<>", "!="] {
            assert_eq!(
                insertions(&format!("SELECT * FROM creatures WHERE id {operator} ty|")),
                ["type"]
            );
        }
    }

    #[test]
    fn replacement_covers_suffix_quotes_and_utf8_while_prefix_stops_at_caret() {
        for (marked_sql, expected, prefix) in [
            ("SELECT ty|pe FROM creatures", "type", "ty"),
            ("SELECT * FROM public.cr|eatures", "creatures", "cr"),
            ("SELECT 🐾na|me FROM creatures", "🐾name", "🐾na"),
            (
                "SELECT \"Odd |Name\" FROM creatures",
                "\"Odd Name\"",
                "Odd ",
            ),
            ("SELECT |type FROM creatures", "type", ""),
        ] {
            let (sql, caret) = marked(marked_sql);
            let ctx = completion_context(&sql, caret).unwrap();
            assert_eq!(&sql[ctx.replacement], expected);
            assert_eq!(ctx.prefix, prefix);
        }
        let (sql, caret) = marked("SELECT '🐾'; SELECT ty|pe FROM creatures");
        let ctx = completion_context(&sql, caret).unwrap();
        assert_eq!(&sql[ctx.replacement], "type");
        let blank = context("SELECT c.| FROM creatures c");
        assert!(blank.replacement.is_empty());
    }

    #[test]
    fn utf16_round_trips_and_clamps_surrogates_and_partial_bytes() {
        let sql = "aé🐾e\u{301}中";
        for byte in (0..=sql.len()).filter(|byte| sql.is_char_boundary(*byte)) {
            assert_eq!(utf16_to_byte(sql, byte_to_utf16(sql, byte)), byte);
        }
        assert_eq!(utf16_to_byte(sql, 3), 3);
        assert_eq!(utf16_to_byte(sql, 4), 7);
        assert_eq!(byte_to_utf16(sql, 2), 1);
        assert_eq!(byte_to_utf16(sql, 6), 2);
        assert_eq!(utf16_to_byte(sql, u32::MAX), sql.len());
        assert_eq!(byte_to_utf16(sql, usize::MAX), 7);
        assert_eq!(utf16_to_byte("", 99), 0);
        assert_eq!(byte_to_utf16("", 99), 0);
    }

    #[test]
    fn stable_deduplicated_prefix_filtered_limit_and_missing_structures() {
        let (mut tables, mut structures) = catalog();
        let ctx = context("SELECT * FROM |");
        tables.push(tables[0].clone());
        for i in 0..20 {
            tables.push(TableInfo {
                schema: "public".into(),
                name: format!("t{i:02}"),
            });
        }
        let first = suggestions(&ctx, &tables, &structures);
        assert_eq!(first.len(), 12);
        tables.reverse();
        assert_eq!(first, suggestions(&ctx, &tables, &structures));
        let ctx = context("SELECT * FROM t1|");
        assert_eq!(suggestions(&ctx, &tables, &structures).len(), 10);
        let ctx = context("SELECT ty| FROM creatures");
        structures.clear();
        assert!(suggestions(&ctx, &tables, &structures).is_empty());
        assert_eq!(
            required_relations(&ctx, &tables),
            [RelationStructureKey::new("public", "creatures")]
        );
    }

    #[test]
    fn unresolved_sources_do_not_hide_duplicate_aliases() {
        for sql in [
            "SELECT c.i| FROM creatures c JOIN missing c ON true",
            "SELECT i| FROM creatures c JOIN missing c ON true",
            "SELECT i| FROM creatures JOIN missing creatures ON true",
        ] {
            assert!(complete(sql).is_empty(), "{sql}");
        }
        assert_eq!(
            insertions("SELECT i| FROM creatures c JOIN missing m ON true"),
            ["c.id"]
        );
        let (mut tables, structures) = catalog();
        tables.push(TableInfo {
            schema: "archive".into(),
            name: "creatures".into(),
        });
        let ctx = context("SELECT c.i| FROM creatures c JOIN owners c ON true");
        assert!(required_relations(&ctx, &tables).is_empty());
        assert!(suggestions(&ctx, &tables, &structures).is_empty());
    }

    #[test]
    fn comments_and_strings_do_not_hide_real_eof_delimiters_or_catalog_sources() {
        for sql in [
            "SELECT ty| FROM creatures /* closed at EOF */",
            "SELECT ty| FROM \"creatures\"",
            "SELECT ty| FROM PUBLIC.CREATURES",
            "SELECT ty| FROM creatures WHERE title = E'foo\\\\'",
            "SELECT ty| FROM creatures WHERE title = $1$ AND title = $$closed$$",
            "SELECT ty| FROM creatures WHERE title = 'closed''quote'",
            "SELECT ty| FROM creatures -- ; FROM owners\r\nWHERE id > 0",
        ] {
            assert_eq!(insertions(sql), ["type"], "{sql}");
        }
        assert!(complete("SELECT Ca| FROM Zoo").is_empty());
        assert!(complete("SELECT ty| FROM \"CREATURES\"").is_empty());
    }

    #[test]
    fn table_schema_and_alias_embedded_quotes_round_trip() {
        let (mut tables, mut structures) = catalog();
        let table = TableInfo {
            schema: "strange\"schema".into(),
            name: "Mixed\" Table".into(),
        };
        let mut structure = structures[&RelationStructureKey::new("public", "creatures")].clone();
        structure.schema = table.schema.clone();
        structure.relation = table.name.clone();
        structures.insert(
            RelationStructureKey::new(&table.schema, &table.name),
            structure,
        );
        tables.push(table);
        let ctx = context("SELECT * FROM mi|");
        assert_eq!(
            suggestions(&ctx, &tables, &structures)[0].insertion,
            "\"strange\"\"schema\".\"Mixed\"\" Table\""
        );
        let ctx = context("SELECT sa| FROM \"strange\"\"schema\".\"Mixed\"\" Table\" \"a\"\"b\"");
        assert_eq!(
            suggestions(&ctx, &tables, &structures)[0].insertion,
            "\"say\"\"hi\""
        );
        let ctx = context("SELECT i| FROM \"strange\"\"schema\".\"Mixed\"\" Table\" AS \"a\"\"b\" JOIN owners o ON true");
        let candidates: Vec<_> = suggestions(&ctx, &tables, &structures)
            .into_iter()
            .map(|c| c.insertion)
            .collect();
        assert_eq!(candidates, ["\"a\"\"b\".id", "o.id"]);
    }

    #[test]
    fn columns_are_also_stable_deduplicated_and_capped() {
        let (tables, mut structures) = catalog();
        let structure = structures
            .get_mut(&RelationStructureKey::new("public", "creatures"))
            .unwrap();
        let column = structure.columns[0].clone();
        structure.columns = (0..20)
            .map(|index| {
                let mut column = column.clone();
                column.name = format!("col{index:02}");
                column
            })
            .collect();
        structure.columns.push(structure.columns[0].clone());
        let ctx = context("SELECT co| FROM creatures");
        let first = suggestions(&ctx, &tables, &structures);
        assert_eq!(first.len(), 12);
        assert_eq!(first[0].insertion, "col00");
        structures
            .get_mut(&RelationStructureKey::new("public", "creatures"))
            .unwrap()
            .columns
            .reverse();
        assert_eq!(suggestions(&ctx, &tables, &structures), first);
    }

    #[test]
    fn nonreserved_highlighter_keywords_can_be_relation_and_alias_names() {
        let (mut tables, mut structures) = catalog();
        tables.push(TableInfo {
            schema: "public".into(),
            name: "text".into(),
        });
        let mut structure = structures[&RelationStructureKey::new("public", "creatures")].clone();
        structure.relation = "text".into();
        structures.insert(RelationStructureKey::new("public", "text"), structure);
        for sql in [
            "SELECT ty| FROM TEXT",
            "SELECT public.text.ty| FROM public.TEXT",
            "SELECT text.ty| FROM creatures text",
            "SELECT text.ty| FROM creatures AS text",
        ] {
            let ctx = context(sql);
            assert_eq!(
                suggestions(&ctx, &tables, &structures)[0].insertion,
                "type",
                "{sql}"
            );
        }
    }

    #[test]
    fn every_utf8_prefix_and_every_utf16_caret_is_safe() {
        for text in [
            "SELECT 🐾name FROM \"Fancy Schema\".\"Zoo\" z WHERE z.\"quote\"\"name\" = E'\\🐾' /* nested /* x */ */;",
            "SELECT c.title FROM creatures c WHERE c.title = $标签$🐾'$标签$ -- comment\r\n",
            "SELECT U&\"a\\0062\" FROM creatures; SELECT E'\\'tail' FROM owners",
        ] {
            for byte in (0..=text.len()).filter(|byte| text.is_char_boundary(*byte)) {
                let sql = &text[..byte];
                for caret in 0..=byte_to_utf16(sql, sql.len()) + 1 {
                    if let Some(ctx) = completion_context(sql, caret) {
                        assert!(sql.is_char_boundary(ctx.replacement.start));
                        assert!(sql.is_char_boundary(ctx.replacement.end));
                        assert!(ctx.replacement.start <= ctx.replacement.end);
                        assert!(ctx.replacement.end <= sql.len());
                    }
                }
            }
        }
    }
}
