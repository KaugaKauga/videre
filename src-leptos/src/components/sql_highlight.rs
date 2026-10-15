use std::ops::Range;

/// Finds PostgreSQL/common SQL keywords outside identifiers, strings, and comments.
///
/// Ranges are UTF-8 byte offsets, ordered and nonoverlapping. The source is never
/// rewritten: render its slices as escaped text, not HTML. This is a lightweight
/// highlighter, not a SQL parser; contextual keywords and common type names are
/// highlighted wherever they occur as standalone unquoted tokens.
pub fn keyword_ranges(sql: &str) -> Vec<Range<usize>> {
    let mut ranges = Vec::new();
    let mut cursor = 0;

    while cursor < sql.len() {
        let ch = next_char(sql, cursor);
        if ch == '$' {
            if let Some(end) = dollar_delimiter_end(sql, cursor) {
                let delimiter = &sql[cursor..end];
                // Search the opaque body directly; no copies or tag allocation.
                cursor = sql[end..]
                    .find(delimiter)
                    .map_or(sql.len(), |offset| end + offset + delimiter.len());
                continue;
            }
        }
        match ch {
            '\'' | '"' => cursor = skip_quoted(sql, cursor, ch, false),
            '-' if sql[cursor..].starts_with("--") => {
                cursor += 2;
                while cursor < sql.len() {
                    let ch = next_char(sql, cursor);
                    if ch == '\n' || ch == '\r' {
                        break;
                    }
                    cursor += ch.len_utf8();
                }
            }
            '/' if sql[cursor..].starts_with("/*") => {
                cursor = skip_block_comment(sql, cursor);
            }

            _ if is_identifier_continue(ch) => {
                let start = cursor;
                cursor += ch.len_utf8();
                while cursor < sql.len() {
                    let ch = next_char(sql, cursor);
                    if !is_identifier_continue(ch) {
                        break;
                    }
                    cursor += ch.len_utf8();
                }
                let word = &sql[start..cursor];
                if word.eq_ignore_ascii_case("E") && sql[cursor..].starts_with('\'') {
                    cursor = skip_quoted(sql, cursor, '\'', true);
                } else if word.eq_ignore_ascii_case("U")
                    && (sql[cursor..].starts_with("&'") || sql[cursor..].starts_with("&\""))
                {
                    cursor += 1; // U& escapes Unicode values, not quote delimiters.
                    cursor = skip_quoted(sql, cursor, next_char(sql, cursor), false);
                } else if is_keyword(word) {
                    ranges.push(start..cursor);
                }
            }
            _ => cursor += ch.len_utf8(),
        }
    }

    ranges
}

pub(super) fn is_keyword(word: &str) -> bool {
    // KEYWORDS is strictly sorted uppercase ASCII; fold only bytes being compared.
    KEYWORDS
        .binary_search_by(|keyword| {
            keyword
                .bytes()
                .cmp(word.bytes().map(|byte| byte.to_ascii_uppercase()))
        })
        .is_ok()
}

pub(super) fn next_char(sql: &str, cursor: usize) -> char {
    // Every cursor advance consumes a char or a known ASCII delimiter.
    sql[cursor..].chars().next().unwrap()
}

pub(super) fn is_identifier_start(ch: char) -> bool {
    // PostgreSQL's lexer accepts high-bit characters, including combining marks.
    ch.is_ascii_alphabetic() || ch == '_' || !ch.is_ascii()
}

pub(super) fn is_identifier_continue(ch: char) -> bool {
    is_identifier_start(ch) || ch.is_ascii_digit() || ch == '$'
}

pub(super) fn skip_quoted(sql: &str, start: usize, quote: char, backslash_escapes: bool) -> usize {
    let mut cursor = start + 1;
    while cursor < sql.len() {
        let ch = next_char(sql, cursor);
        cursor += ch.len_utf8();
        if backslash_escapes && ch == '\\' {
            if cursor < sql.len() {
                cursor += next_char(sql, cursor).len_utf8();
            }
        } else if ch == quote {
            if sql[cursor..].starts_with(quote) {
                cursor += 1;
            } else {
                return cursor;
            }
        }
    }
    sql.len()
}

pub(super) fn skip_block_comment(sql: &str, start: usize) -> usize {
    let mut cursor = start + 2;
    let mut depth = 1;
    while cursor < sql.len() {
        if sql[cursor..].starts_with("/*") {
            depth += 1;
            cursor += 2;
        } else if sql[cursor..].starts_with("*/") {
            depth -= 1;
            cursor += 2;
            if depth == 0 {
                return cursor;
            }
        } else {
            cursor += next_char(sql, cursor).len_utf8();
        }
    }
    sql.len()
}

pub(super) fn dollar_delimiter_end(sql: &str, start: usize) -> Option<usize> {
    let mut cursor = start + 1;
    let first = sql[cursor..].chars().next()?;
    if first == '$' {
        return Some(cursor + 1);
    }
    if !is_identifier_start(first) {
        return None;
    }
    cursor += first.len_utf8();
    while cursor < sql.len() {
        let ch = next_char(sql, cursor);
        if ch == '$' {
            return Some(cursor + 1);
        }
        if !is_identifier_start(ch) && !ch.is_ascii_digit() {
            return None;
        }
        cursor += ch.len_utf8();
    }
    None
}

const KEYWORDS: &[&str] = &[
    "ABORT",
    "ALL",
    "ALTER",
    "ANALYZE",
    "AND",
    "ANY",
    "ARRAY",
    "AS",
    "ASC",
    "ASYMMETRIC",
    "AT",
    "AUTHORIZATION",
    "BEGIN",
    "BETWEEN",
    "BIGINT",
    "BIGSERIAL",
    "BIT",
    "BOOL",
    "BOOLEAN",
    "BOTH",
    "BY",
    "BYTEA",
    "CALL",
    "CASCADE",
    "CASE",
    "CAST",
    "CHAR",
    "CHARACTER",
    "CHECK",
    "COALESCE",
    "COLLATE",
    "COLLATION",
    "COLUMN",
    "COMMIT",
    "CONCURRENTLY",
    "CONFLICT",
    "CONSTRAINT",
    "COPY",
    "CREATE",
    "CROSS",
    "CURRENT",
    "CURRENT_CATALOG",
    "CURRENT_DATE",
    "CURRENT_ROLE",
    "CURRENT_SCHEMA",
    "CURRENT_TIME",
    "CURRENT_TIMESTAMP",
    "CURRENT_USER",
    "DATABASE",
    "DATE",
    "DECIMAL",
    "DEFAULT",
    "DEFERRABLE",
    "DELETE",
    "DESC",
    "DISTINCT",
    "DO",
    "DOUBLE",
    "DROP",
    "ELSE",
    "END",
    "ESCAPE",
    "EXCEPT",
    "EXCLUDE",
    "EXISTS",
    "EXPLAIN",
    "EXTRACT",
    "FALSE",
    "FETCH",
    "FILTER",
    "FIRST",
    "FLOAT",
    "FLOAT4",
    "FLOAT8",
    "FOLLOWING",
    "FOR",
    "FOREIGN",
    "FREEZE",
    "FROM",
    "FULL",
    "GRANT",
    "GREATEST",
    "GROUP",
    "GROUPING",
    "GROUPS",
    "HAVING",
    "ILIKE",
    "IN",
    "INDEX",
    "INITIALLY",
    "INNER",
    "INSERT",
    "INT",
    "INT2",
    "INT4",
    "INT8",
    "INTEGER",
    "INTERSECT",
    "INTERVAL",
    "INTO",
    "IS",
    "ISNULL",
    "JOIN",
    "JSON",
    "JSONB",
    "KEY",
    "LAST",
    "LATERAL",
    "LEADING",
    "LEAST",
    "LEFT",
    "LIKE",
    "LIMIT",
    "LOCALTIME",
    "LOCALTIMESTAMP",
    "LOCK",
    "MATCHED",
    "MATERIALIZED",
    "MERGE",
    "MONEY",
    "NATURAL",
    "NEXT",
    "NO",
    "NOT",
    "NOTHING",
    "NOTNULL",
    "NULL",
    "NULLIF",
    "NULLS",
    "NUMERIC",
    "OF",
    "OFFSET",
    "ON",
    "ONLY",
    "OR",
    "ORDER",
    "OTHERS",
    "OUTER",
    "OVER",
    "OVERLAPS",
    "PARTITION",
    "PLACING",
    "PRECEDING",
    "PRECISION",
    "PRIMARY",
    "RANGE",
    "REAL",
    "RECURSIVE",
    "REFERENCES",
    "RESTRICT",
    "RETURNING",
    "REVOKE",
    "RIGHT",
    "ROLLBACK",
    "ROLLUP",
    "ROW",
    "ROWS",
    "SAVEPOINT",
    "SCHEMA",
    "SELECT",
    "SERIAL",
    "SESSION_USER",
    "SET",
    "SETOF",
    "SIMILAR",
    "SMALLINT",
    "SMALLSERIAL",
    "SOME",
    "SYMMETRIC",
    "TABLE",
    "TABLESAMPLE",
    "TEXT",
    "THEN",
    "TIES",
    "TIME",
    "TIMESTAMP",
    "TIMESTAMPTZ",
    "TIMETZ",
    "TO",
    "TRAILING",
    "TRANSACTION",
    "TRUE",
    "TRUNCATE",
    "UNBOUNDED",
    "UNION",
    "UNIQUE",
    "UPDATE",
    "USER",
    "USING",
    "UUID",
    "VACUUM",
    "VALUES",
    "VARBIT",
    "VARCHAR",
    "VARIADIC",
    "VARYING",
    "VERBOSE",
    "VIEW",
    "WHEN",
    "WHERE",
    "WINDOW",
    "WITH",
    "WITHIN",
    "XML",
    "ZONE",
];

#[cfg(test)]
mod tests {
    use super::*;

    fn assert_keywords(sql: &str, expected: &[&str]) {
        let ranges = keyword_ranges(sql);
        let mut reconstructed = String::with_capacity(sql.len());
        let mut previous_end = 0;
        let mut actual = Vec::new();
        for range in &ranges {
            assert!(range.start < range.end, "empty range: {range:?}");
            assert!(range.start >= previous_end, "unordered ranges: {ranges:?}");
            assert!(range.end <= sql.len(), "out-of-bounds range: {range:?}");
            assert!(sql.is_char_boundary(range.start));
            assert!(sql.is_char_boundary(range.end));
            reconstructed.push_str(&sql[previous_end..range.start]);
            reconstructed.push_str(&sql[range.clone()]);
            actual.push(&sql[range.clone()]);
            previous_end = range.end;
        }
        reconstructed.push_str(&sql[previous_end..]);
        assert_eq!(reconstructed, sql, "source text changed");
        assert_eq!(actual, expected, "SQL: {sql:?}");
    }

    #[test]
    fn recognizes_mixed_case_keywords_without_changing_case() {
        assert_keywords(
            "sElEcT DISTINCT name FrOm people WhErE active aNd deleted IS nOt NuLl Or FALSE",
            &[
                "sElEcT", "DISTINCT", "FrOm", "WhErE", "aNd", "IS", "nOt", "NuLl", "Or", "FALSE",
            ],
        );
    }

    #[test]
    fn does_not_match_substrings_in_identifiers() {
        assert_keywords(
            "SELECT selectivity, _SELECT, SELECT_, from_table, x$WHERE, SELECT$tail, \
             $SELECT, λSELECT, SELECT名, SELECT\u{0301}, 😀FROM, 123SELECT FROM things",
            &["SELECT", "FROM"],
        );
    }

    #[test]
    fn keywords_can_touch_punctuation() {
        assert_keywords(
            "(SELECT);FROM,WHERE=TRUE::BOOLEAN AND(NOT FALSE) UNION ALL VALUES(NULL)",
            &[
                "SELECT", "FROM", "WHERE", "TRUE", "BOOLEAN", "AND", "NOT", "FALSE", "UNION",
                "ALL", "VALUES", "NULL",
            ],
        );
    }

    #[test]
    fn skips_quoted_identifiers_and_doubled_quotes() {
        assert_keywords(
            r#"SELECT "SELECT", "a""FROM""WHERE", "😀JOIN" FROM "WHERE""GROUP";"#,
            &["SELECT", "FROM"],
        );
    }

    #[test]
    fn skips_single_quoted_strings_and_doubled_quotes() {
        assert_keywords(
            "SELECT 'FROM WHERE', 'it''s SELECT', 'GROUP ''BY'' ORDER', '😀 JOIN' \
             FROM things WHERE name = 'NULL';",
            &["SELECT", "FROM", "WHERE"],
        );
    }

    #[test]
    fn skips_escape_strings_including_escaped_multibyte_characters() {
        assert_keywords(
            r#"SELECT E'it\'s FROM', e'\\' AS label, E'WHERE\\\'JOIN', E'α\😀SELECT' FROM records"#,
            &["SELECT", "AS", "FROM"],
        );
    }

    #[test]
    fn ordinary_strings_do_not_treat_backslashes_as_quote_escapes() {
        assert_keywords(r#"SELECT 'text\' FROM records"#, &["SELECT", "FROM"]);
    }

    #[test]
    fn escape_prefix_must_be_a_standalone_token() {
        assert_keywords(r#"SELECT nameE'text\' FROM records"#, &["SELECT", "FROM"]);
    }

    #[test]
    fn skips_unicode_escape_strings_and_identifiers() {
        assert_keywords(
            r#"SELECT U&'\0053ELECT ''FROM''', u&'\+01F600WHERE' AS label FROM U&"\0054ABLE""JOIN";"#,
            &["SELECT", "AS", "FROM"],
        );
    }

    #[test]
    fn skips_dollar_quoted_bodies_including_other_delimiters() {
        assert_keywords(
            r#"SELECT $$FROM 'WHERE' "JOIN" -- SELECT /* GROUP */ 😀$$,
                $tag$ORDER $other$BY$other$ LIMIT$tag$ FROM records"#,
            &["SELECT", "FROM"],
        );
    }

    #[test]
    fn supports_unicode_dollar_tags_and_identifier_tag_characters() {
        assert_keywords(
            "$тег_1$SELECT '$$' /* FROM */$тег_1$ WHERE x = $é$JOIN$é$ \
             AND y = $a\u{0301}2$ORDER$a\u{0301}2$ OR z = $_tag9$GROUP$_tag9$",
            &["WHERE", "AND", "OR"],
        );
    }

    #[test]
    fn dollar_tags_are_case_sensitive() {
        assert_keywords(
            "SELECT $Tag$FROM $tag$ WHERE$Tag$ AS label FROM records",
            &["SELECT", "AS", "FROM"],
        );
    }

    #[test]
    fn dollar_delimiters_attached_to_identifiers_are_not_strings() {
        assert_keywords(
            "SELECT name$tag$WHERE$tag$, foo$$FROM$$, SELECT$dollar FROM records",
            &["SELECT", "FROM"],
        );
    }

    #[test]
    fn invalid_dollar_tags_do_not_hide_following_keywords() {
        assert_keywords(
            "$9$ SELECT $bad-tag$ FROM records $unfinished_tag WHERE",
            &["SELECT", "FROM", "WHERE"],
        );
    }

    #[test]
    fn skips_nested_block_comments() {
        assert_keywords(
            "SELECT /* outer WHERE /* inner JOIN */ FROM \
             /* GROUP /* 😀 ORDER */ BY */ */ FROM records;",
            &["SELECT", "FROM"],
        );
    }

    #[test]
    fn comments_ignore_strings_and_other_comment_forms() {
        assert_keywords(
            "SELECT -- FROM /* JOIN */ 'WHERE'\n\
             FROM records /* -- WHERE\n $$SELECT$$ \"GROUP\" */ WHERE true;",
            &["SELECT", "FROM", "WHERE", "true"],
        );
    }

    #[test]
    fn line_comments_end_at_lf_cr_or_crlf() {
        assert_keywords(
            "-- WHERE\rSELECT -- JOIN\r\nFROM records -- GROUP\nWHERE true -- ORDER",
            &["SELECT", "FROM", "WHERE", "true"],
        );
    }

    #[test]
    fn unfinished_strings_and_comments_hide_the_remaining_input() {
        for tail in [
            "'WHERE FROM",
            "'WHERE '' FROM",
            "\"WHERE FROM",
            "\"WHERE \"\" FROM",
            "E'WHERE\\' FROM",
            "e'WHERE\\",
            "U&'WHERE FROM",
            "u&\"WHERE FROM",
            "$$WHERE FROM",
            "$tag$WHERE FROM",
            "$名$WHERE FROM",
            "-- WHERE FROM",
            "/* WHERE FROM",
            "/* WHERE /* JOIN */ FROM",
            "/* WHERE /* JOIN */ /* FROM */",
        ] {
            assert_keywords(&format!("SELECT {tail}"), &["SELECT"]);
        }
    }

    #[test]
    fn empty_and_partial_tokens_are_safe() {
        for sql in [
            "", " \t\r\n", "'", "\"", "$", "$$", "$tag", "--", "/*", "E'", "U&\"", "123", "名",
            "😀", "SELE",
        ] {
            assert_keywords(sql, &[]);
        }
        assert_keywords(
            "-SELECT /FROM *WHERE $ AND",
            &["SELECT", "FROM", "WHERE", "AND"],
        );
    }

    #[test]
    fn returns_exact_utf8_byte_ranges_and_preserves_all_text() {
        let sql = "é SELECT 名 FROM t WHERE x = '😀';";
        assert_eq!(keyword_ranges(sql), vec![3..9, 14..18, 21..26]);
        assert_keywords(sql, &["SELECT", "FROM", "WHERE"]);
        assert_keywords(
            "SELECT '<script>FROM & WHERE</script>' AS \"<JOIN>\"; -- <ORDER>\n",
            &["SELECT", "AS"],
        );
    }

    #[test]
    fn recognizes_realistic_joins_grouping_and_pagination() {
        let sql = "WITH totals AS (\n\
            SELECT o.customer_id, COUNT(*) AS orders, SUM(o.amount) AS total\n\
            FROM orders AS o\n\
            INNER JOIN customers AS c ON c.id = o.customer_id\n\
            LEFT JOIN regions AS r USING (region_id)\n\
            WHERE o.status = 'SELECT' AND c.active IS TRUE\n\
            GROUP BY o.customer_id\n\
            HAVING COUNT(*) > 1\n\
        )\n\
        SELECT DISTINCT customer_id, total FROM totals\n\
        ORDER BY total DESC NULLS LAST LIMIT 20 OFFSET 5;";
        assert_keywords(
            sql,
            &[
                "WITH", "AS", "SELECT", "AS", "AS", "FROM", "AS", "INNER", "JOIN", "AS", "ON",
                "LEFT", "JOIN", "AS", "USING", "WHERE", "AND", "IS", "TRUE", "GROUP", "BY",
                "HAVING", "SELECT", "DISTINCT", "FROM", "ORDER", "BY", "DESC", "NULLS", "LAST",
                "LIMIT", "OFFSET",
            ],
        );
    }

    #[test]
    fn recognizes_window_and_frame_keywords() {
        assert_keywords(
            "SELECT SUM(amount) FILTER (WHERE amount IS NOT NULL) OVER (\
                PARTITION BY customer_id ORDER BY created_at \
                ROWS BETWEEN UNBOUNDED PRECEDING AND CURRENT ROW EXCLUDE TIES\
             ) AS total FROM records WINDOW recent AS (\
                ORDER BY created_at RANGE BETWEEN CURRENT ROW AND UNBOUNDED FOLLOWING\
             ), previous AS (GROUPS 1 PRECEDING EXCLUDE NO OTHERS);",
            &[
                "SELECT",
                "FILTER",
                "WHERE",
                "IS",
                "NOT",
                "NULL",
                "OVER",
                "PARTITION",
                "BY",
                "ORDER",
                "BY",
                "ROWS",
                "BETWEEN",
                "UNBOUNDED",
                "PRECEDING",
                "AND",
                "CURRENT",
                "ROW",
                "EXCLUDE",
                "TIES",
                "AS",
                "FROM",
                "WINDOW",
                "AS",
                "ORDER",
                "BY",
                "RANGE",
                "BETWEEN",
                "CURRENT",
                "ROW",
                "AND",
                "UNBOUNDED",
                "FOLLOWING",
                "AS",
                "GROUPS",
                "PRECEDING",
                "EXCLUDE",
                "NO",
                "OTHERS",
            ],
        );
    }

    #[test]
    fn recognizes_case_boolean_and_set_operators() {
        assert_keywords(
            "SELECT CASE WHEN x BETWEEN 1 AND 2 THEN TRUE ELSE FALSE END AS flag \
             FROM records WHERE x IN (1, 2) OR name ILIKE 'SELECT' \
             UNION ALL SELECT NULL INTERSECT SELECT TRUE EXCEPT SELECT FALSE;",
            &[
                "SELECT",
                "CASE",
                "WHEN",
                "BETWEEN",
                "AND",
                "THEN",
                "TRUE",
                "ELSE",
                "FALSE",
                "END",
                "AS",
                "FROM",
                "WHERE",
                "IN",
                "OR",
                "ILIKE",
                "UNION",
                "ALL",
                "SELECT",
                "NULL",
                "INTERSECT",
                "SELECT",
                "TRUE",
                "EXCEPT",
                "SELECT",
                "FALSE",
            ],
        );
    }

    #[test]
    fn recognizes_common_types_and_statement_keywords() {
        let sql = "EXPLAIN ANALYZE SELECT CAST AS BOOLEAN BOOL SMALLINT INTEGER INT BIGINT \
                   DECIMAL NUMERIC REAL DOUBLE PRECISION CHAR CHARACTER VARYING VARCHAR \
                   TEXT BYTEA DATE TIME TIMESTAMP TIMESTAMPTZ INTERVAL JSON JSONB UUID ARRAY \
                   INSERT INTO DEFAULT VALUES RETURNING UPDATE SET DELETE CREATE TABLE \
                   PRIMARY KEY REFERENCES ALTER DROP TRUNCATE BEGIN COMMIT ROLLBACK";
        assert_keywords(sql, &sql.split_ascii_whitespace().collect::<Vec<_>>());
    }

    #[test]
    fn all_utf8_prefixes_of_incomplete_input_are_safe() {
        for sql in [
            "SELECT 名, E'α\\😀 WHERE' FROM records /* JOIN /* λ */ ORDER */ WHERE TRUE",
            "SELECT $тег_1$😀 FROM $other$ WHERE$тег_1$ AS label FROM \"é\"",
            "SELECT U&'\\0053😀 ''WHERE''' FROM records -- λ WHERE\nLIMIT 1",
        ] {
            for end in (0..=sql.len()).filter(|&end| sql.is_char_boundary(end)) {
                let prefix = &sql[..end];
                let ranges = keyword_ranges(prefix);
                let expected: Vec<_> = ranges.iter().map(|range| &prefix[range.clone()]).collect();
                assert_keywords(prefix, &expected);
            }
        }
    }

    #[test]
    fn keyword_table_is_sorted_unique_ascii_and_matches_all_cases() {
        for pair in KEYWORDS.windows(2) {
            assert!(
                pair[0] < pair[1],
                "unsorted or duplicate keywords: {pair:?}"
            );
        }
        for &keyword in KEYWORDS {
            assert!(keyword.is_ascii());
            assert!(keyword
                .chars()
                .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_'));
            let lowercase = keyword.to_ascii_lowercase();
            let mixed_case: String = keyword
                .bytes()
                .enumerate()
                .map(|(index, byte)| {
                    char::from(if index % 2 == 0 {
                        byte.to_ascii_lowercase()
                    } else {
                        byte
                    })
                })
                .collect();
            assert_keywords(keyword, &[keyword]);
            assert_keywords(&lowercase, &[&lowercase]);
            assert_keywords(&mixed_case, &[&mixed_case]);
        }
    }

    #[test]
    fn binary_search_preserves_linear_lookup_for_prefixes_and_near_misses() {
        let assert_same_lookup = |word: &str| {
            assert_eq!(
                is_keyword(word),
                KEYWORDS
                    .iter()
                    .any(|keyword| word.eq_ignore_ascii_case(keyword)),
                "lookup changed for {word:?}",
            );
        };
        for &keyword in KEYWORDS {
            for end in 0..=keyword.len() {
                assert_same_lookup(&keyword[..end]);
            }
            for suffix in ["_", "$tail", "λ", "\u{0301}", "123"] {
                assert_same_lookup(&format!("{keyword}{suffix}"));
            }
        }
        for word in [
            "",
            "0",
            "!",
            "ZZZ",
            "selectivity",
            "SÉLECT",
            "λSELECT",
            "ſelect",
        ] {
            assert_same_lookup(word);
        }
    }
}
