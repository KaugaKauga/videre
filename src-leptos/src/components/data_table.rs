use std::collections::HashMap;

use leptos::prelude::*;

use crate::types::ForeignKeyInfo;

// ---------------------------------------------------------------------------
// Sorting helpers
// ---------------------------------------------------------------------------

#[derive(Clone, Copy, PartialEq)]
enum SortDir {
    Asc,
    Desc,
}

fn compare_values(a: &serde_json::Value, b: &serde_json::Value) -> std::cmp::Ordering {
    use serde_json::Value::*;
    use std::cmp::Ordering::*;

    match (a, b) {
        (Null, Null) => Equal,
        (Null, _) => Greater, // nulls sort last
        (_, Null) => Less,
        (Number(a), Number(b)) => {
            let a = a.as_f64().unwrap_or(0.0);
            let b = b.as_f64().unwrap_or(0.0);
            a.partial_cmp(&b).unwrap_or(Equal)
        }
        (String(a), String(b)) => a.cmp(b),
        (Bool(a), Bool(b)) => a.cmp(b),
        (a, b) => a.to_string().cmp(&b.to_string()),
    }
}

fn sort_rows(
    rows: &[Vec<serde_json::Value>],
    col_idx: usize,
    dir: SortDir,
) -> Vec<Vec<serde_json::Value>> {
    let mut sorted = rows.to_vec();
    sorted.sort_by(|a, b| {
        let cmp = compare_values(&a[col_idx], &b[col_idx]);
        if dir == SortDir::Desc {
            cmp.reverse()
        } else {
            cmp
        }
    });
    sorted
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
    use std::cmp::Ordering;

    // -- compare_values -----------------------------------------------------

    #[test]
    fn compare_null_with_null_is_equal() {
        assert_eq!(compare_values(&json!(null), &json!(null)), Ordering::Equal);
    }

    #[test]
    fn compare_null_sorts_last() {
        // null > non-null
        assert_eq!(compare_values(&json!(null), &json!(1)), Ordering::Greater);
        assert_eq!(compare_values(&json!(1), &json!(null)), Ordering::Less);
        assert_eq!(compare_values(&json!(null), &json!("a")), Ordering::Greater);
    }

    #[test]
    fn compare_numbers() {
        assert_eq!(compare_values(&json!(1), &json!(2)), Ordering::Less);
        assert_eq!(compare_values(&json!(2), &json!(1)), Ordering::Greater);
        assert_eq!(compare_values(&json!(42), &json!(42)), Ordering::Equal);
    }

    #[test]
    fn compare_floats() {
        assert_eq!(compare_values(&json!(1.5), &json!(2.5)), Ordering::Less);
        assert_eq!(compare_values(&json!(3.14), &json!(3.14)), Ordering::Equal);
    }

    #[test]
    fn compare_strings() {
        assert_eq!(compare_values(&json!("alpha"), &json!("beta")), Ordering::Less);
        assert_eq!(compare_values(&json!("z"), &json!("a")), Ordering::Greater);
        assert_eq!(compare_values(&json!("same"), &json!("same")), Ordering::Equal);
    }

    #[test]
    fn compare_bools() {
        assert_eq!(compare_values(&json!(false), &json!(true)), Ordering::Less);
        assert_eq!(compare_values(&json!(true), &json!(false)), Ordering::Greater);
        assert_eq!(compare_values(&json!(true), &json!(true)), Ordering::Equal);
    }

    #[test]
    fn compare_mixed_types_uses_to_string() {
        // array vs array falls through to to_string comparison
        let a = json!([1, 2]);
        let b = json!([3, 4]);
        let result = compare_values(&a, &b);
        assert_eq!(result, a.to_string().cmp(&b.to_string()));
    }

    // -- sort_rows ----------------------------------------------------------

    #[test]
    fn sort_rows_ascending() {
        let rows = vec![
            vec![json!(3), json!("c")],
            vec![json!(1), json!("a")],
            vec![json!(2), json!("b")],
        ];
        let sorted = sort_rows(&rows, 0, SortDir::Asc);
        assert_eq!(sorted[0][0], json!(1));
        assert_eq!(sorted[1][0], json!(2));
        assert_eq!(sorted[2][0], json!(3));
    }

    #[test]
    fn sort_rows_descending() {
        let rows = vec![
            vec![json!(1), json!("a")],
            vec![json!(3), json!("c")],
            vec![json!(2), json!("b")],
        ];
        let sorted = sort_rows(&rows, 0, SortDir::Desc);
        assert_eq!(sorted[0][0], json!(3));
        assert_eq!(sorted[1][0], json!(2));
        assert_eq!(sorted[2][0], json!(1));
    }

    #[test]
    fn sort_rows_by_string_column() {
        let rows = vec![
            vec![json!(1), json!("Zeus")],
            vec![json!(2), json!("Athena")],
            vec![json!(3), json!("Hera")],
        ];
        let sorted = sort_rows(&rows, 1, SortDir::Asc);
        assert_eq!(sorted[0][1], json!("Athena"));
        assert_eq!(sorted[1][1], json!("Hera"));
        assert_eq!(sorted[2][1], json!("Zeus"));
    }

    #[test]
    fn sort_rows_nulls_sort_last_ascending() {
        let rows = vec![
            vec![json!(null)],
            vec![json!(1)],
            vec![json!(2)],
        ];
        let sorted = sort_rows(&rows, 0, SortDir::Asc);
        assert_eq!(sorted[0][0], json!(1));
        assert_eq!(sorted[1][0], json!(2));
        assert!(sorted[2][0].is_null());
    }

    #[test]
    fn sort_rows_empty_input() {
        let rows: Vec<Vec<serde_json::Value>> = vec![];
        let sorted = sort_rows(&rows, 0, SortDir::Asc);
        assert!(sorted.is_empty());
    }

    #[test]
    fn sort_rows_does_not_mutate_original() {
        let rows = vec![
            vec![json!(2)],
            vec![json!(1)],
        ];
        let _ = sort_rows(&rows, 0, SortDir::Asc);
        assert_eq!(rows[0][0], json!(2));
        assert_eq!(rows[1][0], json!(1));
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
}

// ---------------------------------------------------------------------------
// DataTable component
// ---------------------------------------------------------------------------

/// Generic sortable data table.
///
/// Renders a `<table>` with a sticky header row. Click column headers to
/// cycle through ascending / descending / unsorted.
///
/// Optional FK support: pass `fk_columns` (column-name -> ForeignKeyInfo)
/// and `fk_click` signal. When a user clicks an FK cell the signal is set
/// so the parent can open a detail panel.
#[component]
pub fn DataTable(
    columns: Vec<String>,
    rows: Vec<Vec<serde_json::Value>>,
    #[prop(optional)] fk_columns: Option<HashMap<String, ForeignKeyInfo>>,
    #[prop(optional)] fk_click: Option<RwSignal<Option<(ForeignKeyInfo, serde_json::Value)>>>,
) -> impl IntoView {
    let sort_col = RwSignal::new(None::<usize>);
    let sort_dir = RwSignal::new(SortDir::Asc);

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
                                let is_fk = fk_by_idx.contains_key(&idx);
                                view! {
                                    <th
                                        class="data-table-th"
                                        on:click=move |_| {
                                            let current = sort_col.get_untracked();
                                            if current == Some(idx) {
                                                // Same column: cycle Asc -> Desc -> None
                                                if sort_dir.get_untracked() == SortDir::Asc {
                                                    sort_dir.set(SortDir::Desc);
                                                } else {
                                                    sort_col.set(None);
                                                }
                                            } else {
                                                sort_col.set(Some(idx));
                                                sort_dir.set(SortDir::Asc);
                                            }
                                        }
                                    >
                                        <span class="data-table-th-inner">
                                            <span>{name}</span>
                                            {if is_fk {
                                                Some(view! {
                                                    <span class="fk-badge" title="Foreign key">{"\u{1F517}"}</span>
                                                })
                                            } else {
                                                None
                                            }}
                                            {move || {
                                                let sc = sort_col.get();
                                                if sc == Some(idx) {
                                                    let arrow = if sort_dir.get() == SortDir::Asc {
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
                        let sc = sort_col.get();
                        let sd = sort_dir.get();

                        let display_rows = source_rows.with_value(|rows| {
                            match sc {
                                Some(idx) => sort_rows(rows, idx, sd),
                                None => rows.clone(),
                            }
                        });

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
