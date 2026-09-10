use std::collections::HashMap;

use leptos::prelude::*;

use crate::types::{ForeignKeyInfo, SortDirection, SortSpec};

fn next_sort(current: Option<&SortSpec>, column: &str) -> Option<SortSpec> {
    match current {
        Some(sort) if sort.column == column && sort.direction == SortDirection::Asc => {
            Some(SortSpec {
                column: column.to_string(),
                direction: SortDirection::Desc,
            })
        }
        Some(sort) if sort.column == column => None,
        _ => Some(SortSpec {
            column: column.to_string(),
            direction: SortDirection::Asc,
        }),
    }
}

// ---------------------------------------------------------------------------
// Format a single cell value
// ---------------------------------------------------------------------------

fn format_value(val: &serde_json::Value) -> String {
    match val {
        serde_json::Value::Null => String::new(), // handled specially in view
        serde_json::Value::String(s) => s.clone(),
        serde_json::Value::Number(n) => n.to_string(),
        serde_json::Value::Bool(b) => b.to_string(),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn sort_cycle_is_ascending_descending_then_default() {
        let ascending = next_sort(None, "name").unwrap();
        assert_eq!(ascending.column, "name");
        assert_eq!(ascending.direction, SortDirection::Asc);

        let descending = next_sort(Some(&ascending), "name").unwrap();
        assert_eq!(descending.direction, SortDirection::Desc);
        assert_eq!(next_sort(Some(&descending), "name"), None);
    }

    #[test]
    fn selecting_another_column_starts_ascending() {
        let current = SortSpec {
            column: "name".into(),
            direction: SortDirection::Desc,
        };

        assert_eq!(
            next_sort(Some(&current), "created_at"),
            Some(SortSpec {
                column: "created_at".into(),
                direction: SortDirection::Asc,
            })
        );
    }

    // -- format_value -------------------------------------------------------

    #[test]
    fn format_null_returns_empty() {
        assert_eq!(format_value(&json!(null)), "");
    }

    #[test]
    fn format_string() {
        assert_eq!(format_value(&json!("hello")), "hello");
    }

    #[test]
    fn format_number() {
        assert_eq!(format_value(&json!(42)), "42");
        assert_eq!(format_value(&json!(3.14)), "3.14");
    }

    #[test]
    fn format_bool() {
        assert_eq!(format_value(&json!(true)), "true");
        assert_eq!(format_value(&json!(false)), "false");
    }

    #[test]
    fn format_array_uses_to_string() {
        let val = json!([1, 2, 3]);
        let formatted = format_value(&val);
        assert_eq!(formatted, val.to_string());
    }

    #[test]
    fn format_object_uses_to_string() {
        let val = json!({"key": "value"});
        let formatted = format_value(&val);
        assert_eq!(formatted, val.to_string());
    }

    /// The frontend half of the type guarantee.
    ///
    /// These are the exact values the backend emits for each Postgres type — the
    /// same strings pinned by `cells_show_the_exact_expected_text` in
    /// `src-tauri/src/pg/data.rs`. Both ends assert the same list, so a change to
    /// either one shows up here rather than as a blank cell in the app.
    #[test]
    fn backend_values_reach_the_cell_intact() {
        let cases = [
            // Natively decoded: numbers and booleans stay typed.
            (json!(9223372036854775807i64), "9223372036854775807"),
            (json!(32767), "32767"),
            (json!(0.1), "0.1"),
            (json!(2.5), "2.5"),
            (json!(true), "true"),
            // Server-rendered, carried as strings.
            (
                json!("12345678901234567890.1234567890"),
                "12345678901234567890.1234567890",
            ),
            (json!("2024-01-15 10:30:00+00"), "2024-01-15 10:30:00+00"),
            (json!("2024-01-15"), "2024-01-15"),
            (json!("3 days 04:00:00"), "3 days 04:00:00"),
            (json!("$1,234.56"), "$1,234.56"),
            (json!("\\x48656c6c6f"), "\\x48656c6c6f"),
            (json!("{\"b\": [1, 2]}"), "{\"b\": [1, 2]}"),
            (json!("{alpha,beta}"), "{alpha,beta}"),
            (json!("{{1,2},{3,4}}"), "{{1,2},{3,4}}"),
            (json!("{[1,5),[10,20)}"), "{[1,5),[10,20)}"),
            (json!("'a' 'cat' 'fat'"), "'a' 'cat' 'fat'"),
            (
                json!("0b7f2c1e-4a5d-4f8e-9c3a-1d2e3f4a5b6c"),
                "0b7f2c1e-4a5d-4f8e-9c3a-1d2e3f4a5b6c",
            ),
            (json!("<r><a>1</a></r>"), "<r><a>1</a></r>"),
            (json!("(3,4),(1,2)"), "(3,4),(1,2)"),
            // `character(n)` keeps its padding.
            (json!("ab      "), "ab      "),
        ];

        for (value, expected) in cases {
            assert_eq!(format_value(&value), expected, "for {value:?}");
            // Nothing here is NULL, so none may take the NULL-marker branch.
            assert!(!value.is_null(), "for {value:?}");
        }
    }

    /// A blank cell is only ever an empty string, never a lost value — and it
    /// stays distinguishable from a real NULL, which the view renders as a
    /// "NULL" marker instead of empty text.
    #[test]
    fn empty_string_and_null_stay_distinguishable() {
        let empty = json!("");
        let null = json!(null);

        assert_eq!(format_value(&empty), "");
        assert_eq!(format_value(&null), "");
        // Same rendered text, different branch in the view.
        assert!(!empty.is_null());
        assert!(null.is_null());
    }

    /// The decoder's error marker is deliberately visible rather than silent, so
    /// it must survive to the cell if it ever appears.
    #[test]
    fn unreadable_marker_is_not_swallowed() {
        let val = json!("<unreadable: some driver error>");
        assert_eq!(format_value(&val), "<unreadable: some driver error>");
    }
}

// ---------------------------------------------------------------------------
// DataTable component
// ---------------------------------------------------------------------------

/// Generic data table whose header clicks request server-side sorting.
///
/// Renders a `<table>` with a sticky header row. Click column headers to
/// cycle through ascending / descending / the backend's default order.
///
/// Optional FK support: pass `fk_columns` (column-name -> ForeignKeyInfo)
/// and `fk_click` signal. When a user clicks an FK cell the signal is set
/// so the parent can open a detail panel.
#[component]
pub fn DataTable(
    columns: Vec<String>,
    rows: Vec<Vec<serde_json::Value>>,
    sort: RwSignal<Option<SortSpec>>,
    on_sort: Callback<Option<SortSpec>>,
    #[prop(optional)] fk_columns: Option<HashMap<String, ForeignKeyInfo>>,
    #[prop(optional)] fk_click: Option<RwSignal<Option<(ForeignKeyInfo, serde_json::Value)>>>,
) -> impl IntoView {
    let fk_map = fk_columns.unwrap_or_default();

    let col_count = columns.len();

    // Pre-compute which columns are FK columns (by index)
    let fk_by_idx: HashMap<usize, ForeignKeyInfo> = columns
        .iter()
        .enumerate()
        .filter_map(|(i, name)| fk_map.get(name).cloned().map(|fk| (i, fk)))
        .collect();

    // Store rows in a signal so sorting can produce a new view
    let source_rows = StoredValue::new(rows);

    view! {
        <div class="data-table-wrap">
            <table class="data-table">
                <thead>
                    <tr>
                        {columns
                            .iter()
                            .enumerate()
                            .map(|(idx, col_name)| {
                                let name = col_name.clone();
                                let clicked_name = name.clone();
                                let indicator_name = name.clone();
                                let is_fk = fk_by_idx.contains_key(&idx);
                                view! {
                                    <th
                                        class="data-table-th"
                                        on:click=move |_| {
                                            let current = sort.get_untracked();
                                            on_sort.run(next_sort(current.as_ref(), &clicked_name));
                                        }
                                    >
                                        <span class="data-table-th-inner">
                                            <span>{name}</span>
                                            {if is_fk {
                                                Some(view! {
                                                    <span class="fk-badge" title="Foreign key">"FK"</span>
                                                })
                                            } else {
                                                None
                                            }}
                                            {move || {
                                                let active_sort = sort.get();
                                                if let Some(active) = active_sort
                                                    .filter(|active| active.column == indicator_name)
                                                {
                                                    let arrow = if active.direction == SortDirection::Asc {
                                                        "\u{25B2}" // up triangle
                                                    } else {
                                                        "\u{25BC}" // down triangle
                                                    };
                                                    view! { <span class="sort-indicator">{arrow}</span> }.into_any()
                                                } else {
                                                    view! { <span class="sort-indicator sort-inactive">{"\u{25B2}"}</span> }.into_any()
                                                }
                                            }}
                                        </span>
                                    </th>
                                }
                            })
                            .collect::<Vec<_>>()
                        }
                    </tr>
                </thead>
                <tbody>
                    {move || {
                        let display_rows = source_rows.get_value();

                        if display_rows.is_empty() {
                            return vec![view! {
                                <tr>
                                    <td class="data-table-empty" colspan=col_count.to_string()>
                                        "No results."
                                    </td>
                                </tr>
                            }.into_any()];
                        }

                        display_rows
                            .into_iter()
                            .map(|row| {
                                let cells: Vec<_> = row
                                    .iter()
                                    .enumerate()
                                    .map(|(ci, val)| {
                                        let is_null = val.is_null();
                                        let fk_info = fk_by_idx.get(&ci).cloned();
                                        let formatted = format_value(val);
                                        let raw_val = val.clone();

                                        if is_null {
                                            view! {
                                                <td class="data-table-td">
                                                    <span class="null-value">"NULL"</span>
                                                </td>
                                            }
                                            .into_any()
                                        } else if let Some(fk) = fk_info {
                                            // FK cell — render as a clickable link
                                            let fk_click = fk_click;
                                            view! {
                                                <td class="data-table-td">
                                                    <button
                                                        class="fk-link"
                                                        title=format!("View {} record", fk.foreign_table_name)
                                                        on:click=move |_| {
                                                            if let Some(sig) = fk_click {
                                                                sig.set(Some((fk.clone(), raw_val.clone())));
                                                            }
                                                        }
                                                    >
                                                        {formatted.clone()}
                                                    </button>
                                                </td>
                                            }
                                            .into_any()
                                        } else {
                                            view! {
                                                <td class="data-table-td">{formatted}</td>
                                            }
                                            .into_any()
                                        }
                                    })
                                    .collect();
                                view! { <tr class="data-table-row">{cells}</tr> }.into_any()
                            })
                            .collect::<Vec<_>>()
                    }}
                </tbody>
            </table>
        </div>
    }
}
