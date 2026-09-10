use std::collections::HashMap;

use leptos::task::spawn_local;
use leptos::{ev, leptos_dom::helpers::window_event_listener, prelude::*};
use wasm_bindgen::JsCast;

use crate::components::data_table::DataTable;
use crate::components::drawer::Drawer;
use crate::components::icons;
use crate::stores::db_store::DbStore;
use crate::tauri;
use crate::types::{
    ColumnConstraint, ColumnInfo, ForeignKeyInfo, RelationStructure, RowData, SortSpec, TableData,
};

const PAGE_SIZE: i64 = 100;

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
        let sort_column = sort.as_ref().map(|sort| sort.column.as_str());
        let sort_direction = sort.as_ref().map(|sort| sort.direction);
        match tauri::invoke::<TableData>(
            "get_table_data",
            &serde_json::json!({
                "tableName": name,
                "schema": schema,
                "limit": PAGE_SIZE,
                "offset": (page as i64) * PAGE_SIZE,
                "sortColumn": sort_column,
                "sortDirection": sort_direction,
            }),
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

fn relation_context(structure: &RelationStructure) -> String {
    format!(
        "{}.{} · {}",
        structure.schema, structure.relation, structure.relation_kind
    )
}

fn metadata_value(value: Option<&String>) -> String {
    value
        .filter(|value| !value.is_empty())
        .cloned()
        .unwrap_or_else(|| "-".to_string())
}

fn detail_field(label: &str, value: impl Into<String>) -> AnyView {
    let label = label.to_string();
    let value = value.into();
    view! {
        <div class="row-detail-field column-info-field">
            <span class="row-detail-label">{label}</span>
            <span class="row-detail-value">{value}</span>
        </div>
    }
    .into_any()
}

fn detail_code_field(label: &str, value: String) -> AnyView {
    let label = label.to_string();
    view! {
        <div class="row-detail-field column-info-field">
            <span class="row-detail-label">{label}</span>
            <code class="row-detail-value column-info-code">{value}</code>
        </div>
    }
    .into_any()
}

fn detail_section(title: &str, fields: Vec<AnyView>) -> AnyView {
    let title = title.to_string();
    view! {
        <section class="column-info-section">
            <h4 class="column-info-section-title">{title}</h4>
            <div class="column-info-fields">{fields}</div>
        </section>
    }
    .into_any()
}

fn constraint_view(constraint: ColumnConstraint) -> AnyView {
    let kind = constraint.kind;
    let definition = constraint.definition;
    let mut fields = vec![detail_code_field("name", constraint.name)];

    if let Some(position) = constraint.column_position {
        fields.push(detail_code_field("position", position.to_string()));
    }
    fields.push(detail_code_field(
        "columns",
        if constraint.source_columns.is_empty() {
            "-".to_string()
        } else {
            constraint.source_columns.join(", ")
        },
    ));
    if let (Some(schema), Some(relation)) = (constraint.target_schema, constraint.target_relation) {
        fields.push(detail_code_field(
            "referenced relation",
            format!("{schema}.{relation}"),
        ));
        fields.push(detail_code_field(
            "referenced columns",
            if constraint.target_columns.is_empty() {
                "-".to_string()
            } else {
                constraint.target_columns.join(", ")
            },
        ));
        fields.push(detail_code_field(
            "on update",
            metadata_value(constraint.on_update.as_ref()),
        ));
        fields.push(detail_code_field(
            "on delete",
            metadata_value(constraint.on_delete.as_ref()),
        ));
    }

    view! {
        <article class="column-constraint">
            <code class="column-constraint-kind">{kind}</code>
            <div class="column-info-fields">{fields}</div>
            <code class="column-info-code column-constraint-definition">{definition}</code>
        </article>
    }
    .into_any()
}

fn column_details(structure: RelationStructure, column: ColumnInfo) -> AnyView {
    let mut overview = vec![
        detail_field("Position", column.ordinal_position.to_string()),
        detail_field(
            "Relation",
            format!("{}.{}", structure.schema, structure.relation),
        ),
        detail_code_field("relation kind", structure.relation_kind),
        detail_code_field("Type", column.data_type),
        detail_code_field("nullable", column.nullable.to_string()),
    ];
    if let Some(collation) = column.collation.filter(|collation| !collation.is_empty()) {
        overview.push(detail_field("Collation", collation));
    }

    let generation = vec![
        detail_code_field("identity", metadata_value(column.identity.as_ref())),
        detail_code_field("generated", metadata_value(column.generated.as_ref())),
        detail_code_field(
            "default",
            metadata_value(column.default_expression.as_ref()),
        ),
        detail_code_field(
            "generation expression",
            metadata_value(column.generation_expression.as_ref()),
        ),
    ];

    let comment = column.comment.filter(|comment| !comment.is_empty());
    let constraints = if column.constraints.is_empty() {
        view! { <code class="column-info-code">"-"</code> }.into_any()
    } else {
        let constraints = column
            .constraints
            .into_iter()
            .map(constraint_view)
            .collect::<Vec<_>>();
        view! { <div class="column-constraints">{constraints}</div> }.into_any()
    };

    view! {
        <div class="column-info-details">
            {detail_section("overview", overview)}
            {detail_section("value generation", generation)}
            {comment.map(|comment| view! {
                <section class="column-info-section">
                    <h4 class="column-info-section-title">"comment"</h4>
                    <pre class="column-info-comment">{comment}</pre>
                </section>
            })}
            <section class="column-info-section">
                <h4 class="column-info-section-title">"constraints"</h4>
                {constraints}
            </section>
        </div>
    }
    .into_any()
}

fn column_panel_content(
    selected_column: Option<String>,
    structure: Option<RelationStructure>,
    loading: bool,
    error: Option<String>,
    retry: Callback<()>,
) -> AnyView {
    if loading {
        return view! {
            <div class="table-page-loading column-info-loading">
                {icons::icon_spinner(16)}
                <span>"Loading column information…"</span>
            </div>
        }
        .into_any();
    }

    if let Some(error) = error {
        return view! {
            <div class="column-info-error">
                <p>{error}</p>
                <button class="btn btn-secondary btn-sm" type="button" on:click=move |_| retry.run(())>
                    "Retry"
                </button>
            </div>
        }
        .into_any();
    }

    let Some(column_name) = selected_column else {
        return view! { <p class="text-muted text-sm">"No column selected."</p> }.into_any();
    };
    let Some(structure) = structure else {
        return view! {
            <p class="text-muted text-sm">"Column information is not available."</p>
        }
        .into_any();
    };
    let Some(column) = structure
        .columns
        .iter()
        .find(|column| column.name == column_name)
        .cloned()
    else {
        return view! {
            <p class="text-muted text-sm">"This column was not returned by the relation metadata."</p>
        }
        .into_any();
    };

    column_details(structure, column)
}

fn focus_column_info_button(column_name: &str) {
    let Some(document) = web_sys::window().and_then(|window| window.document()) else {
        return;
    };
    let Ok(buttons) = document.query_selector_all("[data-column-info-for]") else {
        return;
    };

    for index in 0..buttons.length() {
        let Some(element) = buttons.item(index) else {
            continue;
        };
        if let Ok(button) = element.dyn_into::<web_sys::HtmlButtonElement>() {
            if button.get_attribute("data-column-info-for").as_deref() == Some(column_name) {
                let _ = button.focus();
                return;
            }
        }
    }
}

fn fetch_relation_structure(
    name: String,
    schema: String,
    db: DbStore,
    request_id: RwSignal<u64>,
    loading: RwSignal<bool>,
    error: RwSignal<Option<String>>,
    context: RwSignal<String>,
) {
    let this_request = request_id.get_untracked().wrapping_add(1);
    request_id.set(this_request);
    loading.set(true);
    error.set(None);

    spawn_local(async move {
        match tauri::get_relation_structure(&name, &schema).await {
            Ok(structure) => {
                if request_id.get_untracked() != this_request {
                    return;
                }
                context.set(relation_context(&structure));
                db.cache_relation_structure(structure);
                loading.set(false);
            }
            Err(error_message) => {
                if request_id.get_untracked() != this_request {
                    return;
                }
                context.set(format!("{schema}.{name}"));
                error.set(Some(format!(
                    "Failed to fetch column information: {error_message}"
                )));
                loading.set(false);
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

    // ---- Drawer state ------------------------------------------------------
    let panel_open = RwSignal::new(false);
    let panel_title = RwSignal::new(String::new());
    let panel_data: RwSignal<Option<RowData>> = RwSignal::new(None);
    let panel_loading = RwSignal::new(false);
    let panel_error: RwSignal<Option<String>> = RwSignal::new(None);

    let column_drawer_open = RwSignal::new(false);
    let column_drawer_title = RwSignal::new(String::new());
    let column_drawer_context = RwSignal::new(String::new());
    let selected_column: RwSignal<Option<String>> = RwSignal::new(None);
    let column_loading = RwSignal::new(false);
    let column_error: RwSignal<Option<String>> = RwSignal::new(None);
    let column_request_id = RwSignal::new(0_u64);
    let column_info_opener: RwSignal<Option<String>> = RwSignal::new(None);

    let close_column_drawer = Callback::new(move |_| {
        column_drawer_open.set(false);
        column_info_opener.update(|opener| {
            if let Some(column_name) = opener.take() {
                focus_column_info_button(&column_name);
            }
        });
    });

    let escape_close_column_drawer = close_column_drawer;
    let table_drawer_keydown = window_event_listener(ev::keydown, move |event| {
        if event.key() != "Escape" {
            return;
        }
        if column_drawer_open.get_untracked() {
            event.prevent_default();
            escape_close_column_drawer.run(());
        } else if panel_open.get_untracked() {
            event.prevent_default();
            panel_open.set(false);
        }
    });
    on_cleanup(move || table_drawer_keydown.remove());

    // Signals that DataTable writes to when FK cells or information buttons are clicked.
    let fk_click: RwSignal<Option<(ForeignKeyInfo, serde_json::Value)>> = RwSignal::new(None);

    // ---- FK column map (static for this table) -----------------------------
    let fk_list = db.get_foreign_keys_for_table(&name, &schema);
    let fk_map: HashMap<String, ForeignKeyInfo> = fk_list
        .into_iter()
        .map(|fk| (fk.column_name.clone(), fk))
        .collect();

    // ---- Initial data fetch ------------------------------------------------
    fetch_page(&name, &schema, 0, None, data, request_id, is_loading, error);

    // ---- Drawer mode transitions -------------------------------------------
    let fk_click_effect = fk_click;
    let close_column_drawer_for_fk = close_column_drawer;
    Effect::new(move |_prev: Option<()>| {
        if let Some((ref fk, ref value)) = fk_click_effect.get() {
            close_column_drawer_for_fk.run(());
            panel_title.set(fk.foreign_table_name.clone());
            panel_open.set(true);
            fetch_fk_row(fk, value, panel_data, panel_loading, panel_error);
        }
    });

    let name_column_info = name.clone();
    let schema_column_info = schema.clone();
    let on_column_info = Callback::new(move |column: String| {
        panel_open.set(false);
        selected_column.set(Some(column.clone()));
        column_info_opener.set(Some(column.clone()));
        column_drawer_title.set(column);
        column_drawer_open.set(true);

        if let Some(structure) = db.get_relation_structure(&schema_column_info, &name_column_info) {
            column_drawer_context.set(relation_context(&structure));
            column_error.set(None);
            column_loading.set(false);
        } else if !column_loading.get_untracked() {
            column_drawer_context.set(format!(
                "{}.{} · Loading relation…",
                schema_column_info, name_column_info
            ));
            fetch_relation_structure(
                name_column_info.clone(),
                schema_column_info.clone(),
                db,
                column_request_id,
                column_loading,
                column_error,
                column_drawer_context,
            );
        }
    });

    let name_retry = name.clone();
    let schema_retry = schema.clone();
    let retry_column_info = Callback::new(move |_| {
        fetch_relation_structure(
            name_retry.clone(),
            schema_retry.clone(),
            db,
            column_request_id,
            column_loading,
            column_error,
            column_drawer_context,
        );
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

    let display_name = name.clone();

    view! {
        <div class="table-page">
            {move || {
                if is_loading.get() && data.get().is_none() {
                    Some(view! {
                        <div class="table-page-loading">
                            {icons::icon_spinner(20)}
                            <span>"Loading data…"</span>
                        </div>
                    })
                } else {
                    None
                }
            }}

            {move || {
                error.get().map(|msg| view! {
                    <div class="table-page-error">
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

                Some(view! {
                    <div class="table-page-header">
                        <h2 class="table-page-title">{display_name.clone()}</h2>
                        <span class="text-muted text-sm">
                            {if is_empty {
                                format!("Empty table • {} columns", cols.len())
                            } else {
                                format!("{total} total rows")
                            }}
                        </span>
                    </div>

                    <div class="table-page-body">
                        <DataTable
                            columns=cols
                            rows=rows
                            sort=sort
                            on_sort=on_sort
                            fk_columns=fk_map.clone()
                            fk_click=fk_click
                            column_info_click=on_column_info
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

            <Drawer
                open=panel_open
                title=panel_title
                subtitle="Referenced row details"
                close_on_escape=false
            >
                {move || {
                    if panel_loading.get() {
                        return view! {
                            <div class="table-page-loading">
                                {icons::icon_spinner(16)}
                                <span>"Loading…"</span>
                            </div>
                        }.into_any();
                    }

                    if let Some(err) = panel_error.get() {
                        return view! {
                            <div class="table-page-error"><p>{err}</p></div>
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
                                        let value = match val {
                                            serde_json::Value::String(value) => value.clone(),
                                            other => other.to_string(),
                                        };
                                        view! { <span>{value}</span> }.into_any()
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

            <Drawer
                open=column_drawer_open
                title=column_drawer_title
                subtitle="Column information"
                context=column_drawer_context
                wide=true
                close_on_escape=false
                on_close=close_column_drawer
            >
                {move || column_panel_content(
                    selected_column.get(),
                    db.get_relation_structure(&schema, &name),
                    column_loading.get(),
                    column_error.get(),
                    retry_column_info,
                )}
            </Drawer>
        </div>
    }
}
