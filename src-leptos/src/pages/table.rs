use std::collections::HashMap;

use leptos::task::spawn_local;
use leptos::{html, prelude::*};

use crate::components::data_table::{CellPosition, ColumnSort, DataTable};
use crate::components::icons;
use crate::components::relation_drawers::{ColumnTarget, RelationDrawerPanels, RelationDrawers};
use crate::stores::db_store::DbStore;
use crate::tauri;
use crate::types::{ForeignKeyInfo, SortSpec, TableData};

const PAGE_SIZE: i64 = 100;

fn page_request_args(
    name: &str,
    schema: &str,
    page: usize,
    sort: Option<&SortSpec>,
) -> serde_json::Value {
    serde_json::json!({
        "tableName": name,
        "schema": schema,
        "limit": PAGE_SIZE,
        "offset": (page as i64) * PAGE_SIZE,
        "sortColumn": sort.map(|sort| sort.column.as_str()),
        "sortDirection": sort.map(|sort| sort.direction),
    })
}

/// Fetch a page of table data from the Tauri backend.
fn fetch_page(
    name: &str,
    schema: &str,
    page: usize,
    sort: Option<SortSpec>,
    data: RwSignal<Option<TableData>>,
    request_id: RwSignal<u64>,
    is_loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    let name = name.to_string();
    let schema = schema.to_string();
    let this_request = request_id.get_untracked().wrapping_add(1);
    request_id.set(this_request);
    is_loading.set(true);
    error.set(None);

    spawn_local(async move {
        match tauri::invoke::<TableData>(
            "get_table_data",
            &page_request_args(&name, &schema, page, sort.as_ref()),
        )
        .await
        {
            Ok(result) => {
                if request_id.get_untracked() != this_request {
                    return;
                }
                data.set(Some(result));
                is_loading.set(false);
            }
            Err(e) => {
                if request_id.get_untracked() != this_request {
                    return;
                }
                error.set(Some(format!("Failed to fetch data: {e}")));
                is_loading.set(false);
            }
        }
    });
}

#[component]
pub fn TablePage(name: String, schema: String) -> impl IntoView {
    let db = use_context::<DbStore>().expect("DbStore not provided");

    // ---- Table data state --------------------------------------------------
    let page = RwSignal::new(0_usize);
    let sort: RwSignal<Option<SortSpec>> = RwSignal::new(None);
    let data: RwSignal<Option<TableData>> = RwSignal::new(None);
    let request_id = RwSignal::new(0_u64);
    let is_loading = RwSignal::new(true);
    let error: RwSignal<Option<String>> = RwSignal::new(None);
    let active_cell: RwSignal<Option<CellPosition>> = RwSignal::new(None);
    let table_page = NodeRef::<html::Div>::new();

    // ---- Drawer state ------------------------------------------------------
    let drawers = RelationDrawers::new();
    let keyboard_enabled = Signal::derive(move || !is_loading.get() && !drawers.is_open());
    let drawer_root = Callback::new(move |()| table_page.get_untracked().map(Into::into));
    let on_fk_click = Callback::new(move |(fk, value): (ForeignKeyInfo, serde_json::Value)| {
        drawers.open_foreign_key(fk, value);
    });

    // ---- FK column map (static for this table) -----------------------------
    let fk_list = db.get_foreign_keys_for_table(&name, &schema);
    let fk_map: HashMap<String, ForeignKeyInfo> = fk_list
        .into_iter()
        .map(|fk| (fk.column_name.clone(), fk))
        .collect();

    // ---- Initial data fetch ------------------------------------------------
    fetch_page(&name, &schema, 0, None, data, request_id, is_loading, error);

    // ---- Pagination helpers ------------------------------------------------
    let total_pages = Memo::new(move |_| {
        data.get()
            .map(|d| ((d.total_rows as f64) / (PAGE_SIZE as f64)).ceil() as usize)
            .unwrap_or(0)
    });

    let name_prev = name.clone();
    let schema_prev = schema.clone();
    let on_prev = move |_: web_sys::MouseEvent| {
        let p = page.get_untracked().saturating_sub(1);
        page.set(p);
        fetch_page(
            &name_prev,
            &schema_prev,
            p,
            sort.get_untracked(),
            data,
            request_id,
            is_loading,
            error,
        );
    };

    let name_next = name.clone();
    let schema_next = schema.clone();
    let on_next = move |_: web_sys::MouseEvent| {
        let p = page.get_untracked() + 1;
        page.set(p);
        fetch_page(
            &name_next,
            &schema_next,
            p,
            sort.get_untracked(),
            data,
            request_id,
            is_loading,
            error,
        );
    };

    let name_sort = name.clone();
    let schema_sort = schema.clone();
    let on_sort = Callback::new(move |next: Option<SortSpec>| {
        sort.set(next.clone());
        page.set(0);
        fetch_page(
            &name_sort,
            &schema_sort,
            0,
            next,
            data,
            request_id,
            is_loading,
            error,
        );
    });

    let name_column_info = name.clone();
    let schema_column_info = schema.clone();

    let name_refresh = name.clone();
    let schema_refresh = schema.clone();
    let refresh_page = Callback::new(move |_| {
        fetch_page(
            &name_refresh,
            &schema_refresh,
            page.get_untracked(),
            sort.get_untracked(),
            data,
            request_id,
            is_loading,
            error,
        );
    });

    view! {
        <div class="table-page" node_ref=table_page>
            <div class="table-page-header">
                <div class="table-page-heading">
                    <h2 class="table-page-title">{name.clone()}</h2>
                    <span class="text-muted text-sm">
                        {move || data.get().map(|result| {
                            if result.rows.is_empty() {
                                format!("Empty table • {} columns", result.columns.len())
                            } else {
                                format!("{} total rows", result.total_rows)
                            }
                        })}
                    </span>
                </div>
                <button
                    class="btn btn-ghost btn-sm table-page-refresh"
                    type="button"
                    title=move || if is_loading.get() { "Loading data" } else if error.get().is_some() { "Retry loading data" } else { "Refresh data" }
                    aria-label=move || if is_loading.get() { "Loading data" } else if error.get().is_some() { "Retry loading data" } else { "Refresh data" }
                    disabled=move || is_loading.get()
                    on:click=move |_| refresh_page.run(())
                >
                    {icons::icon_refresh(16, is_loading)}
                </button>
            </div>
            {move || {
                error.get().map(|msg| view! {
                    <div class="table-page-error" role="alert">
                        <p>{msg}</p>
                    </div>
                })
            }}

            {move || {
                let table_data = data.get()?;
                let cols = table_data.columns.clone();
                let rows = table_data.rows.clone();
                let total = table_data.total_rows;
                let is_empty = rows.is_empty();
                let p = page.get();

                let showing_from = p as i64 * PAGE_SIZE + 1;
                let showing_to = ((p as i64 + 1) * PAGE_SIZE).min(total);

                // The grid sorts by position; requests name the column.
                let fk_columns: HashMap<usize, ForeignKeyInfo> = cols
                    .iter()
                    .enumerate()
                    .filter_map(|(index, column)| fk_map.get(column).cloned().map(|fk| (index, fk)))
                    .collect();
                let sort_names = cols.clone();
                let grid_sort = Signal::derive(move || {
                    sort.with(|sort| {
                        let sort = sort.as_ref()?;
                        let column = sort_names.iter().position(|name| *name == sort.column)?;
                        Some(ColumnSort { column, direction: sort.direction })
                    })
                });
                let sort_columns = cols.clone();
                let on_grid_sort = Callback::new(move |next: Option<ColumnSort>| {
                    on_sort.run(next.and_then(|next| {
                        Some(SortSpec {
                            column: sort_columns.get(next.column)?.clone(),
                            direction: next.direction,
                        })
                    }));
                });
                let info_columns = cols.clone();
                let info_name = name_column_info.clone();
                let info_schema = schema_column_info.clone();
                let on_column_info = Callback::new(move |position: usize| {
                    if let Some(column) = info_columns.get(position) {
                        drawers.open_column(ColumnTarget {
                            position,
                            schema: info_schema.clone(),
                            relation: info_name.clone(),
                            column: column.clone(),
                        });
                    }
                });

                Some(view! {
                    <div class="table-page-body">
                        <DataTable
                            columns=cols
                            rows=rows
                            sort=grid_sort
                            on_sort=on_grid_sort
                            fk_columns=fk_columns
                            fk_click=on_fk_click
                            column_info_click=on_column_info
                            active_cell=active_cell
                            keyboard_enabled=keyboard_enabled
                        />
                    </div>

                    {if !is_empty {
                        let tp = total_pages.get();
                        let is_last_page = move || page.get() + 1 >= tp;
                        Some(view! {
                            <div class="table-page-footer">
                                <span class="text-muted text-sm">
                                    {format!("Showing {showing_from} to {showing_to} of {total} rows")}
                                </span>
                                <div class="pagination">
                                    <button
                                        class="btn btn-ghost btn-sm"
                                        disabled=move || page.get() == 0
                                        on:click=on_prev.clone()
                                    >
                                        "‹ Previous"
                                    </button>
                                    <span class="text-muted text-sm">
                                        {move || format!("Page {} of {}", page.get() + 1, tp)}
                                    </span>
                                    <button
                                        class="btn btn-ghost btn-sm"
                                        disabled=is_last_page
                                        on:click=on_next.clone()
                                    >
                                        "Next ›"
                                    </button>
                                </div>
                            </div>
                        })
                    } else {
                        None
                    }}
                })
            }}

            <RelationDrawerPanels drawers=drawers root=drawer_root active_cell=active_cell />
        </div>
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::SortDirection;

    #[test]
    fn page_request_keeps_relation_page_and_sort() {
        let sort = SortSpec {
            column: "MiXeD Name".to_string(),
            direction: SortDirection::Desc,
        };
        assert_eq!(
            page_request_args("Orders", "archive", 2, Some(&sort)),
            serde_json::json!({
                "tableName": "Orders",
                "schema": "archive",
                "limit": 100,
                "offset": 200,
                "sortColumn": "MiXeD Name",
                "sortDirection": "desc",
            })
        );
        assert_eq!(
            page_request_args("Orders", "public", 0, None),
            serde_json::json!({
                "tableName": "Orders",
                "schema": "public",
                "limit": 100,
                "offset": 0,
                "sortColumn": null,
                "sortDirection": null,
            })
        );
    }
}
