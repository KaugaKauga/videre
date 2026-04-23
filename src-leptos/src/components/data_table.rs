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
                                                    <span class="fk-badge" title="Foreign key">"FK"</span>
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
