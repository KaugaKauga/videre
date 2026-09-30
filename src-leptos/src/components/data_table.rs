use std::collections::HashMap;

use leptos::{html, prelude::*};
use wasm_bindgen::JsCast;
use web_sys::{Element, HtmlElement};

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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct CellPosition {
    pub row: usize,
    pub column: usize,
}

fn clamped_cell(
    position: Option<CellPosition>,
    row_count: usize,
    column_count: usize,
) -> Option<CellPosition> {
    if row_count == 0 || column_count == 0 {
        return None;
    }
    let position = position.unwrap_or(CellPosition { row: 0, column: 0 });
    Some(CellPosition {
        row: position.row.min(row_count - 1),
        column: position.column.min(column_count - 1),
    })
}

fn next_cell(
    key: &str,
    current: CellPosition,
    row_count: usize,
    column_count: usize,
) -> Option<CellPosition> {
    let mut cell = clamped_cell(Some(current), row_count, column_count)?;
    match key {
        "ArrowDown" | "j" => cell.row = (cell.row + 1).min(row_count - 1),
        "ArrowUp" | "k" => cell.row = cell.row.saturating_sub(1),
        "ArrowRight" => cell.column = (cell.column + 1).min(column_count - 1),
        "ArrowLeft" => cell.column = cell.column.saturating_sub(1),
        _ => return None,
    }
    Some(cell)
}

fn copy_value(value: &serde_json::Value) -> String {
    if value.is_null() {
        "NULL".into()
    } else {
        format_value(value)
    }
}

fn cell_copy_text(
    rows: &[Vec<serde_json::Value>],
    position: CellPosition,
    has_selection: bool,
) -> Option<String> {
    if has_selection {
        return None;
    }
    rows.get(position.row)?.get(position.column).map(copy_value)
}

fn cell_for_target(table: &Element, target: &Element) -> Option<(HtmlElement, CellPosition)> {
    let cell = target.closest("td[data-row][data-column]").ok()??;
    // Only the cell and its FK button own navigation, never an embedded editor.
    if !table.contains(Some(&cell))
        || !(target.is_same_node(Some(&cell)) || target.class_list().contains("fk-link"))
    {
        return None;
    }
    let position = CellPosition {
        row: cell.get_attribute("data-row")?.parse().ok()?,
        column: cell.get_attribute("data-column")?.parse().ok()?,
    };
    Some((cell.dyn_into::<HtmlElement>().ok()?, position))
}

fn focused_cell(table: &Element) -> Option<(HtmlElement, CellPosition)> {
    let target = table.owner_document()?.active_element()?;
    cell_for_target(table, &target)
}

/// Focus the data cell at `position` inside `root` and scroll it into view.
pub(crate) fn focus_cell(root: &Element, position: CellPosition) {
    let selector = format!(
        "td[data-row=\"{}\"][data-column=\"{}\"]",
        position.row, position.column
    );
    let Some(cell) = root
        .query_selector(&selector)
        .ok()
        .flatten()
        .and_then(|cell| cell.dyn_into::<HtmlElement>().ok())
    else {
        return;
    };
    let focus = web_sys::FocusOptions::new();
    focus.set_prevent_scroll(true);
    let _ = cell.focus_with_options(&focus);
    let scroll = web_sys::ScrollIntoViewOptions::new();
    scroll.set_block(web_sys::ScrollLogicalPosition::Nearest);
    scroll.set_inline(web_sys::ScrollLogicalPosition::Nearest);
    cell.scroll_into_view_with_scroll_into_view_options(&scroll);
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn cell_movement_uses_arrows_and_vim_vertical_keys() {
        let cell = CellPosition { row: 1, column: 1 };
        for key in ["ArrowDown", "j"] {
            assert_eq!(
                next_cell(key, cell, 3, 4),
                Some(CellPosition { row: 2, column: 1 })
            );
        }
        for key in ["ArrowUp", "k"] {
            assert_eq!(
                next_cell(key, cell, 3, 4),
                Some(CellPosition { row: 0, column: 1 })
            );
        }
        assert_eq!(
            next_cell("ArrowLeft", cell, 3, 4),
            Some(CellPosition { row: 1, column: 0 })
        );
        assert_eq!(
            next_cell("ArrowRight", cell, 3, 4),
            Some(CellPosition { row: 1, column: 2 })
        );
    }

    #[test]
    fn cell_movement_stops_at_page_and_column_boundaries() {
        let first = CellPosition { row: 0, column: 0 };
        let last = CellPosition { row: 2, column: 3 };
        for key in ["ArrowUp", "k", "ArrowLeft"] {
            assert_eq!(next_cell(key, first, 3, 4), Some(first));
        }
        for key in ["ArrowDown", "j", "ArrowRight"] {
            assert_eq!(next_cell(key, last, 3, 4), Some(last));
        }
        for key in ["ArrowDown", "j", "ArrowUp", "k", "ArrowRight", "ArrowLeft"] {
            assert_eq!(next_cell(key, first, 1, 1), Some(first));
        }
    }

    #[test]
    fn tab_copy_and_activation_are_not_cell_movement() {
        let cell = CellPosition { row: 0, column: 0 };
        for key in ["Tab", "Enter", "Escape", "c", "C", "J", "K", " "] {
            assert_eq!(next_cell(key, cell, 2, 2), None);
        }
    }

    #[test]
    fn focus_position_survives_reload_and_clamps_to_smaller_results() {
        let cell = CellPosition { row: 8, column: 3 };
        assert_eq!(clamped_cell(Some(cell), 100, 10), Some(cell));
        assert_eq!(
            clamped_cell(Some(cell), 4, 2),
            Some(CellPosition { row: 3, column: 1 })
        );
        assert_eq!(
            clamped_cell(None, 4, 2),
            Some(CellPosition { row: 0, column: 0 })
        );
        assert_eq!(clamped_cell(Some(cell), 0, 2), None);
        assert_eq!(clamped_cell(Some(cell), 4, 0), None);
        assert_eq!(next_cell("j", cell, 0, 2), None);
    }

    #[test]
    fn copied_values_are_complete_and_keep_postgres_text_and_null_distinct() {
        let long = format!("  {}\n尾  ", "value".repeat(1000));
        assert_eq!(copy_value(&json!(long)), long);
        assert_eq!(copy_value(&json!(null)), "NULL");
        assert_eq!(copy_value(&json!("")), "");
        assert_eq!(copy_value(&json!("ab      ")), "ab      ");
        assert_eq!(copy_value(&json!("{alpha,beta}")), "{alpha,beta}");
        assert_eq!(copy_value(&json!(42)), "42");
        assert_eq!(copy_value(&json!(false)), "false");
        assert_eq!(copy_value(&json!([1, 2])), "[1,2]");
        assert_eq!(copy_value(&json!({"key": "value"})), "{\"key\":\"value\"}");
    }

    #[test]
    fn selected_text_keeps_native_copy_and_stale_coordinates_do_not_copy() {
        let rows = vec![vec![json!("full cell"), json!(null), json!("")]];
        let cell = CellPosition { row: 0, column: 0 };
        assert_eq!(
            cell_copy_text(&rows, cell, false).as_deref(),
            Some("full cell")
        );
        assert_eq!(cell_copy_text(&rows, cell, true), None);
        assert_eq!(
            cell_copy_text(&rows, CellPosition { row: 1, column: 0 }, false),
            None
        );
        assert_eq!(
            cell_copy_text(&rows, CellPosition { row: 0, column: 3 }, false),
            None
        );
        assert_eq!(
            cell_copy_text(&rows, CellPosition { row: 0, column: 1 }, false).as_deref(),
            Some("NULL")
        );
        assert_eq!(
            cell_copy_text(&rows, CellPosition { row: 0, column: 2 }, false).as_deref(),
            Some("")
        );
    }

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
    /// Last focused cell. The parent owns it so position survives page loads and
    /// focus can return to it when a drawer closes.
    active_cell: RwSignal<Option<CellPosition>>,
    /// Pauses cell navigation and copy, e.g. while loading or a drawer is open.
    keyboard_enabled: Signal<bool>,
    sort: RwSignal<Option<SortSpec>>,
    on_sort: Callback<Option<SortSpec>>,
    #[prop(optional)] fk_columns: Option<HashMap<String, ForeignKeyInfo>>,
    #[prop(optional)] fk_click: Option<RwSignal<Option<(ForeignKeyInfo, serde_json::Value)>>>,
    /// Receives the selected column name for the column-information drawer.
    #[prop(optional)]
    column_info_click: Option<Callback<String>>,
) -> impl IntoView {
    let fk_map = fk_columns.unwrap_or_default();

    let col_count = columns.len();
    let row_count = rows.len();
    let table_ref = NodeRef::<html::Table>::new();
    // Clamp on read so a smaller result set still has exactly one Tab stop.
    let cell_tab_stop = StoredValue::new(Selector::new(move || {
        clamped_cell(active_cell.get(), row_count, col_count)
    }));
    let copy_error = RwSignal::new(None::<String>);

    // Pre-compute which columns are FK columns (by index)
    let fk_by_idx: HashMap<usize, ForeignKeyInfo> = columns
        .iter()
        .enumerate()
        .filter_map(|(i, name)| fk_map.get(name).cloned().map(|fk| (i, fk)))
        .collect();

    let source_rows = StoredValue::new(rows);

    let on_focus_in = move |event: web_sys::FocusEvent| {
        let Some(table) = table_ref.get_untracked() else {
            return;
        };
        let Some((_, position)) = event
            .target()
            .and_then(|target| target.dyn_into::<Element>().ok())
            .and_then(|target| cell_for_target(&table, &target))
        else {
            return;
        };
        if active_cell.get_untracked() != Some(position) {
            active_cell.set(Some(position));
            copy_error.set(None);
        }
    };
    let on_mouse_down = move |event: web_sys::MouseEvent| {
        if event.button() != 0 {
            return;
        }
        let Some(table) = table_ref.get_untracked() else {
            return;
        };
        let Some(target) = event
            .target()
            .and_then(|target| target.dyn_into::<Element>().ok())
        else {
            return;
        };
        // Focus before the browser begins selecting text; never cancel the selection gesture.
        if target
            .closest("button, a, input, textarea, select, [contenteditable]")
            .ok()
            .flatten()
            .is_none()
        {
            if let Some(cell) = target.closest("td[data-row][data-column]").ok().flatten() {
                if table.contains(Some(&cell)) {
                    if let Ok(cell) = cell.dyn_into::<HtmlElement>() {
                        let _ = cell.focus();
                    }
                }
            }
        }
    };
    let on_key_down = move |event: web_sys::KeyboardEvent| {
        if !keyboard_enabled.get_untracked()
            || event.default_prevented()
            || event.is_composing()
            || event.ctrl_key()
            || event.meta_key()
            || event.alt_key()
            || event.shift_key()
        {
            return;
        }
        let Some(table) = table_ref.get_untracked() else {
            return;
        };
        let Some((cell, position)) = focused_cell(&table) else {
            return;
        };
        if let Some(next) = next_cell(&event.key(), position, row_count, col_count) {
            event.prevent_default();
            // A keyboard move starts a new cell interaction, not a stale text selection.
            if let Some(document) = table.owner_document() {
                if let Ok(Some(selection)) = document.get_selection() {
                    let _ = selection.remove_all_ranges();
                }
            }
            focus_cell(&table, next);
        } else if event.key() == "Enter" {
            if let Some(button) = cell
                .query_selector(".fk-link")
                .ok()
                .flatten()
                .and_then(|button| button.dyn_into::<web_sys::HtmlButtonElement>().ok())
            {
                event.prevent_default();
                button.click();
            }
        }
    };
    let on_copy = move |event: web_sys::Event| {
        let Ok(event) = event.dyn_into::<web_sys::ClipboardEvent>() else {
            return;
        };
        if !keyboard_enabled.get_untracked() || event.default_prevented() {
            return;
        }
        let Some(table) = table_ref.get_untracked() else {
            return;
        };
        let Some((_, position)) = focused_cell(&table) else {
            return;
        };
        let Some(document) = table.owner_document() else {
            return;
        };
        let has_selection = match document.get_selection() {
            Ok(Some(selection)) => !selection.is_collapsed(),
            Ok(None) => false,
            Err(_) => return,
        };
        let text = source_rows.with_value(|rows| cell_copy_text(rows, position, has_selection));
        let Some(text) = text else {
            return;
        };
        let Some(clipboard) = event.clipboard_data() else {
            copy_error.set(Some(
                "Could not access the clipboard. Select the value and copy it instead.".into(),
            ));
            return;
        };
        match clipboard.set_data("text/plain", &text) {
            Ok(()) => {
                event.prevent_default();
                copy_error.set(None);
            }
            Err(_) => copy_error.set(Some(
                "Could not copy the cell. Select the value and copy it instead.".into(),
            )),
        }
    };

    view! {
        <div class="data-table-wrap">
            {move || copy_error.get().map(|message| view! {
                <div class="table-page-error" role="alert">{message}</div>
            })}
            <table class="data-table"
                node_ref=table_ref
                aria-label="Table data"
                on:focusin=on_focus_in
                on:mousedown=on_mouse_down
                on:keydown=on_key_down
                on:copy=on_copy
            >
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
                                let info_column = name.clone();
                                let column_info_click = column_info_click;
                                let info_label = format!("Column information for {name}");
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
                                            <button
                                                class="column-info-button"
                                                type="button"
                                                title=info_label.clone()
                                                aria-label=info_label
                                                data-column-info-for=info_column.clone()
                                                on:click=move |ev: web_sys::MouseEvent| {
                                                    ev.stop_propagation();
                                                    if let Some(callback) = column_info_click {
                                                        callback.run(info_column.clone());
                                                    }
                                                }
                                            >
                                                <span aria-hidden="true">"ⓘ"</span>
                                            </button>
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
                            .enumerate()
                            .map(|(ri, row)| {
                                let cells: Vec<_> = row
                                    .iter()
                                    .enumerate()
                                    .map(|(ci, val)| {
                                        let is_null = val.is_null();
                                        let fk_info = fk_by_idx.get(&ci).cloned();
                                        let formatted = format_value(val);
                                        let raw_val = val.clone();
                                        let position = CellPosition { row: ri, column: ci };
                                        let tabindex = move || {
                                            if cell_tab_stop.with_value(|stop| stop.selected(Some(position))) {
                                                0
                                            } else {
                                                -1
                                            }
                                        };

                                        if is_null {
                                            view! {
                                                <td class="data-table-td"
                                                    data-row=ri.to_string()
                                                    data-column=ci.to_string()
                                                    tabindex=tabindex
                                                >
                                                    <span class="null-value">"NULL"</span>
                                                </td>
                                            }
                                            .into_any()
                                        } else if let Some(fk) = fk_info {
                                            // FK cell — render as a clickable link
                                            let fk_click = fk_click;
                                            view! {
                                                <td class="data-table-td"
                                                    data-row=ri.to_string()
                                                    data-column=ci.to_string()
                                                    tabindex=tabindex
                                                >
                                                    <button
                                                        class="fk-link"
                                                        type="button"
                                                        tabindex="-1"
                                                        title=format!("View {} record", fk.foreign_table_name)
                                                        on:click=move |_| {
                                                            // WebKit does not necessarily focus a mouse-clicked button.
                                                            active_cell.set(Some(position));
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
                                                <td class="data-table-td"
                                                    data-row=ri.to_string()
                                                    data-column=ci.to_string()
                                                    tabindex=tabindex
                                                >{formatted}</td>
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
