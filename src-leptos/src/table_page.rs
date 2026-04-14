use std::collections::HashMap;

use leptos::prelude::*;
use leptos::task::spawn_local;

use crate::data_table::DataTable;
use crate::db_store::DbStore;
use crate::drawer::Drawer;
use crate::tauri;
use crate::types::{ForeignKeyInfo, RowData, TableData};

const PAGE_SIZE: i64 = 100;

/// Fetch a page of table data from the Tauri backend.
fn fetch_page(
    name: &str,
    schema: &str,
    page: usize,
    data: RwSignal<Option<TableData>>,
    is_loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
) {
    let name = name.to_string();
    let schema = schema.to_string();
    is_loading.set(true);
    error.set(None);

    spawn_local(async move {
        match tauri::invoke::<TableData>(
            "get_table_data",
            &serde_json::json!({
                "tableName": name,
                "schema": schema,
                "limit": PAGE_SIZE,
                "offset": (page as i64) * PAGE_SIZE,
            }),
        )
        .await
        {
            Ok(result) => {
                data.set(Some(result));
                is_loading.set(false);
            }
            Err(e) => {
                error.set(Some(format!("Failed to fetch data: {e}")));
                is_loading.set(false);
            }
        }
    });
}

/// Fetch a single row by primary key (for FK detail panel).
fn fetch_fk_row(
    fk: &ForeignKeyInfo,
    value: &serde_json::Value,
    panel_data: RwSignal<Option<RowData>>,
    panel_loading: RwSignal<bool>,
    panel_error: RwSignal<Option<String>>,
) {
    let table = fk.foreign_table_name.clone();
    let schema = fk.foreign_table_schema.clone();
    let column = fk.foreign_column_name.clone();
    let value = value.clone();
    panel_loading.set(true);
    panel_error.set(None);
    panel_data.set(None);

    spawn_local(async move {
        match tauri::invoke::<RowData>(
            "get_row_by_pk",
            &serde_json::json!({
                "tableName": table,
                "schema": schema,
                "pkColumn": column,
                "pkValue": value,
            }),
        )
        .await
        {
            Ok(row) => {
                panel_data.set(Some(row));
                panel_loading.set(false);
            }
            Err(e) => {
                panel_error.set(Some(format!("Failed to fetch row: {e}")));
                panel_loading.set(false);
            }
        }
    });
}

// ---------------------------------------------------------------------------
// TablePage component
// ---------------------------------------------------------------------------

#[component]
pub fn TablePage(name: String, schema: String) -> impl IntoView {
    let db = use_context::<DbStore>().expect("DbStore not provided");

    // ---- Table data state --------------------------------------------------
    let page = RwSignal::new(0_usize);
    let data: RwSignal<Option<TableData>> = RwSignal::new(None);
    let is_loading = RwSignal::new(true);
    let error: RwSignal<Option<String>> = RwSignal::new(None);

    // ---- FK side-panel state -----------------------------------------------
    let panel_open = RwSignal::new(false);
    let panel_title = RwSignal::new(String::new());
    let panel_data: RwSignal<Option<RowData>> = RwSignal::new(None);
    let panel_loading = RwSignal::new(false);
    let panel_error: RwSignal<Option<String>> = RwSignal::new(None);

    // Signal that DataTable writes to when an FK cell is clicked.
    let fk_click: RwSignal<Option<(ForeignKeyInfo, serde_json::Value)>> = RwSignal::new(None);

    // ---- FK column map (static for this table) -----------------------------
    let fk_list = db.get_foreign_keys_for_table(&name, &schema);
    let fk_map: HashMap<String, ForeignKeyInfo> = fk_list
        .into_iter()
        .map(|fk| (fk.column_name.clone(), fk))
        .collect();

    // ---- Initial data fetch ------------------------------------------------
    fetch_page(&name, &schema, 0, data, is_loading, error);

    // ---- Handle FK clicks --------------------------------------------------
    // Watch the fk_click signal; when set, open the detail panel.
    let fk_click_effect = fk_click;
    Effect::new(move |_prev: Option<()>| {
        if let Some((ref fk, ref value)) = fk_click_effect.get() {
            panel_title.set(fk.foreign_table_name.clone());
            panel_open.set(true);
            fetch_fk_row(fk, value, panel_data, panel_loading, panel_error);
        }
    });

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
        fetch_page(&name_prev, &schema_prev, p, data, is_loading, error);
    };

    let name_next = name.clone();
    let schema_next = schema.clone();
    let on_next = move |_: web_sys::MouseEvent| {
        let p = page.get_untracked() + 1;
        page.set(p);
        fetch_page(&name_next, &schema_next, p, data, is_loading, error);
    };

    // ---- Display name ------------------------------------------------------
    let display_name = name.clone();

    // ---- View --------------------------------------------------------------
    view! {
        <div class="table-page">
            // Loading state
            {move || {
                if is_loading.get() && data.get().is_none() {
                    Some(view! {
                        <div class="table-page-loading">
                            <svg class="animate-spin" xmlns="http://www.w3.org/2000/svg"
                                 width="20" height="20" viewBox="0 0 24 24" fill="none"
                                 stroke="currentColor" stroke-width="2"
                                 stroke-linecap="round" stroke-linejoin="round">
                                <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                                <path d="M12 6l0 -3"/>
                                <path d="M16.25 7.75l2.15 -2.15"/>
                                <path d="M18 12l3 0"/>
                                <path d="M16.25 16.25l2.15 2.15"/>
                                <path d="M12 18l0 3"/>
                                <path d="M7.75 16.25l-2.15 2.15"/>
                                <path d="M6 12l-3 0"/>
                                <path d="M7.75 7.75l-2.15 -2.15"/>
                            </svg>
                            <span>"Loading data\u{2026}"</span>
                        </div>
                    })
                } else {
                    None
                }
            }}

            // Error state
            {move || {
                error.get().map(|msg| view! {
                    <div class="table-page-error">
                        <p>{msg}</p>
                    </div>
                })
            }}

            // Main content (once data is loaded)
            {move || {
                let table_data = data.get()?;
                let cols = table_data.columns.clone();
                let rows = table_data.rows.clone();
                let total = table_data.total_rows;
                let is_empty = rows.is_empty();
                let p = page.get();

                let showing_from = p as i64 * PAGE_SIZE + 1;
                let showing_to = ((p as i64 + 1) * PAGE_SIZE).min(total);

                Some(view! {
                    // Header
                    <div class="table-page-header">
                        <h2 class="table-page-title">{display_name.clone()}</h2>
                        <span class="text-muted text-sm">
                            {if is_empty {
                                format!("Empty table \u{2022} {} columns", cols.len())
                            } else {
                                format!("{total} total rows")
                            }}
                        </span>
                    </div>

                    // Table
                    <div class="table-page-body">
                        <DataTable
                            columns=cols
                            rows=rows
                            fk_columns=fk_map.clone()
                            fk_click=fk_click
                        />
                    </div>

                    // Pagination footer
                    {if !is_empty {
                        let tp = total_pages.get();
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
                                        "\u{2039} Previous"
                                    </button>
                                    <span class="text-muted text-sm">
                                        {move || format!("Page {} of {}", page.get() + 1, tp)}
                                    </span>
                                    <button
                                        class="btn btn-ghost btn-sm"
                                        disabled=move || page.get() + 1 >= tp
                                        on:click=on_next.clone()
                                    >
                                        "Next \u{203A}"
                                    </button>
                                </div>
                            </div>
                        })
                    } else {
                        None
                    }}
                })
            }}

            // FK detail side panel
            <Drawer open=panel_open title=panel_title subtitle="Referenced row details">
                {move || {
                    if panel_loading.get() {
                        return view! {
                            <div class="table-page-loading">
                                <svg class="animate-spin" xmlns="http://www.w3.org/2000/svg"
                                     width="16" height="16" viewBox="0 0 24 24" fill="none"
                                     stroke="currentColor" stroke-width="2"
                                     stroke-linecap="round" stroke-linejoin="round">
                                    <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                                    <path d="M12 6l0 -3"/>
                                    <path d="M16.25 7.75l2.15 -2.15"/>
                                    <path d="M18 12l3 0"/>
                                    <path d="M16.25 16.25l2.15 2.15"/>
                                    <path d="M12 18l0 3"/>
                                    <path d="M7.75 16.25l-2.15 2.15"/>
                                    <path d="M6 12l-3 0"/>
                                    <path d="M7.75 7.75l-2.15 -2.15"/>
                                </svg>
                                <span>"Loading\u{2026}"</span>
                            </div>
                        }.into_any();
                    }

                    if let Some(ref err) = panel_error.get() {
                        return view! {
                            <div class="table-page-error"><p>{err.clone()}</p></div>
                        }.into_any();
                    }

                    match panel_data.get() {
                        Some(row) => {
                            let pairs: Vec<_> = row.columns.iter().zip(row.values.iter())
                                .map(|(col, val)| {
                                    let display = if val.is_null() {
                                        view! { <span class="null-value">"NULL"</span> }.into_any()
                                    } else if val.is_object() || val.is_array() {
                                        let json = serde_json::to_string_pretty(val)
                                            .unwrap_or_else(|_| val.to_string());
                                        view! { <code class="json-value">{json}</code> }.into_any()
                                    } else {
                                        let s = match val {
                                            serde_json::Value::String(s) => s.clone(),
                                            other => other.to_string(),
                                        };
                                        view! { <span>{s}</span> }.into_any()
                                    };
                                    view! {
                                        <div class="row-detail-field">
                                            <span class="row-detail-label">{col.clone()}</span>
                                            <span class="row-detail-value">{display}</span>
                                        </div>
                                    }
                                })
                                .collect();
                            view! { <div>{pairs}</div> }.into_any()
                        }
                        None => view! {
                            <p class="text-muted text-sm">"No data to display"</p>
                        }.into_any(),
                    }
                }}
            </Drawer>
        </div>
    }
}
