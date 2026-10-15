//! Referenced-row and column-information drawers shared by every data grid.

use leptos::task::spawn_local;
use leptos::{ev, leptos_dom::helpers::window_event_listener, prelude::*};
use wasm_bindgen::JsCast;

use crate::components::data_table::{focus_cell, CellPosition};
use crate::components::drawer::Drawer;
use crate::components::icons;
use crate::stores::db_store::{DbStore, RelationStructureStatus};
use crate::tauri;
use crate::types::{ColumnConstraint, ColumnInfo, ForeignKeyInfo, RelationStructure, RowData};

/// The table column behind a grid column, and the header position that opened it.
#[derive(Clone, Debug, PartialEq)]
pub struct ColumnTarget {
    pub position: usize,
    pub schema: String,
    pub relation: String,
    pub column: String,
}

/// State for the two mutually exclusive drawers. Opening one closes the other.
#[derive(Clone, Copy)]
pub struct RelationDrawers {
    fk_open: RwSignal<bool>,
    fk_title: RwSignal<String>,
    fk_row: RwSignal<Option<RowData>>,
    fk_loading: RwSignal<bool>,
    fk_error: RwSignal<Option<String>>,
    fk_request: RwSignal<u64>,
    column_open: RwSignal<bool>,
    column_title: RwSignal<String>,
    column_target: RwSignal<Option<ColumnTarget>>,
}

impl Default for RelationDrawers {
    fn default() -> Self {
        Self::new()
    }
}

impl RelationDrawers {
    pub fn new() -> Self {
        Self {
            fk_open: RwSignal::new(false),
            fk_title: RwSignal::new(String::new()),
            fk_row: RwSignal::new(None),
            fk_loading: RwSignal::new(false),
            fk_error: RwSignal::new(None),
            fk_request: RwSignal::new(0),
            column_open: RwSignal::new(false),
            column_title: RwSignal::new(String::new()),
            column_target: RwSignal::new(None),
        }
    }

    pub fn is_open(&self) -> bool {
        self.fk_open.get() || self.column_open.get()
    }

    /// Fetch and show the row a foreign-key cell references.
    pub fn open_foreign_key(&self, fk: ForeignKeyInfo, value: serde_json::Value) {
        // Mode transitions must not restore the previous drawer's opener.
        self.column_open.set(false);
        self.fk_title.set(fk.foreign_table_name.clone());
        self.fk_open.set(true);

        let request = self.fk_request.get_untracked().wrapping_add(1);
        self.fk_request.set(request);
        self.fk_loading.set(true);
        self.fk_error.set(None);
        self.fk_row.set(None);
        let drawers = *self;
        spawn_local(async move {
            let result = tauri::invoke::<RowData>(
                "get_row_by_pk",
                &serde_json::json!({
                    "tableName": fk.foreign_table_name,
                    "schema": fk.foreign_table_schema,
                    "pkColumn": fk.foreign_column_name,
                    "pkValue": value,
                }),
            )
            .await;
            // A later click, or an unmounted page, owns the drawer now.
            if drawers.fk_request.try_get_untracked() != Some(request) {
                return;
            }
            match result {
                Ok(row) => drawers.fk_row.set(Some(row)),
                Err(e) => drawers
                    .fk_error
                    .set(Some(format!("Failed to fetch row: {e}"))),
            }
            drawers.fk_loading.set(false);
        });
    }

    /// Show cached or lazily fetched metadata for one table column.
    pub fn open_column(&self, target: ColumnTarget) {
        self.fk_open.set(false);
        self.column_title.set(target.column.clone());
        self.column_target.set(Some(target));
        self.column_open.set(true);
    }
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
    target: Option<ColumnTarget>,
    status: RelationStructureStatus,
    retry: Callback<()>,
) -> AnyView {
    let Some(target) = target else {
        return view! { <p class="text-muted text-sm">"No column selected."</p> }.into_any();
    };
    let structure = match status {
        RelationStructureStatus::Ready(structure) => structure,
        RelationStructureStatus::Loading | RelationStructureStatus::Missing => {
            return view! {
                <div class="table-page-loading column-info-loading">
                    {icons::icon_spinner(16)}
                    <span>"Loading column information…"</span>
                </div>
            }
            .into_any();
        }
        RelationStructureStatus::Failed(error) => {
            return view! {
                <div class="column-info-error">
                    <p>{format!("Failed to fetch column information: {error}")}</p>
                    <button class="btn btn-secondary btn-sm" type="button" on:click=move |_| retry.run(())>
                        "Retry"
                    </button>
                </div>
            }
            .into_any();
        }
    };
    let Some(column) = structure
        .columns
        .iter()
        .find(|column| column.name == target.column)
        .cloned()
    else {
        return view! {
            <p class="text-muted text-sm">"This column was not returned by the relation metadata."</p>
        }
        .into_any();
    };

    column_details(structure, column)
}

fn referenced_row_content(row: RowData) -> AnyView {
    let pairs: Vec<_> = row
        .columns
        .iter()
        .zip(row.values.iter())
        .map(|(col, val)| {
            let display = if val.is_null() {
                view! { <span class="null-value">"NULL"</span> }.into_any()
            } else if val.is_object() || val.is_array() {
                let json = serde_json::to_string_pretty(val).unwrap_or_else(|_| val.to_string());
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

fn focus_column_info_button(root: &web_sys::Element, position: usize) {
    let selector = format!("[data-column-info-for=\"{position}\"]");
    if let Some(button) = root
        .query_selector(&selector)
        .ok()
        .flatten()
        .and_then(|button| button.dyn_into::<web_sys::HtmlElement>().ok())
    {
        let _ = button.focus();
    }
}

/// Renders both drawers. `root` is the positioned page element that contains
/// the grid, so focus returns to the cell or header button that opened them.
#[component]
pub fn RelationDrawerPanels(
    drawers: RelationDrawers,
    root: Callback<(), Option<web_sys::Element>>,
    active_cell: RwSignal<Option<CellPosition>>,
) -> impl IntoView {
    let db = use_context::<DbStore>().expect("DbStore not provided");
    let RelationDrawers {
        fk_open,
        fk_title,
        fk_row,
        fk_loading,
        fk_error,
        column_open,
        column_title,
        column_target,
        ..
    } = drawers;

    // Return focus to the cell that opened the drawer, like the column drawer does.
    let close_fk_drawer = Callback::new(move |_| {
        fk_open.set(false);
        if let (Some(root), Some(position)) = (root.run(()), active_cell.get_untracked()) {
            focus_cell(&root, position);
        }
    });
    let close_column_drawer = Callback::new(move |_| {
        column_open.set(false);
        if let (Some(root), Some(target)) = (root.run(()), column_target.get_untracked()) {
            if !fk_open.get_untracked() {
                focus_column_info_button(&root, target.position);
            }
        }
    });

    let drawer_keydown = window_event_listener(ev::keydown, move |event| {
        if event.key() != "Escape" {
            return;
        }
        if column_open.get_untracked() {
            event.prevent_default();
            close_column_drawer.run(());
        } else if fk_open.get_untracked() {
            event.prevent_default();
            close_fk_drawer.run(());
        }
    });
    on_cleanup(move || drawer_keydown.remove());

    let status = Memo::new(move |_| {
        column_target.with(|target| {
            target
                .as_ref()
                .map(|target| db.relation_structure_status(&target.schema, &target.relation))
        })
    });
    // Also refetches after a reconnect clears the cache under an open drawer.
    // The store holds back a failed relation until its retry cooldown passes.
    Effect::new(move |_| {
        if column_open.get()
            && matches!(
                status.get(),
                Some(RelationStructureStatus::Missing | RelationStructureStatus::Failed(_))
            )
        {
            if let Some(target) = column_target.get_untracked() {
                db.request_relation_structure(&target.schema, &target.relation);
            }
        }
    });
    let retry = Callback::new(move |_| {
        if let Some(target) = column_target.get_untracked() {
            db.retry_relation_structure(&target.schema, &target.relation);
        }
    });
    let column_context = Signal::derive(move || {
        let Some(target) = column_target.get() else {
            return String::new();
        };
        match status.get() {
            Some(RelationStructureStatus::Ready(structure)) => relation_context(&structure),
            Some(RelationStructureStatus::Failed(_)) => {
                format!("{}.{}", target.schema, target.relation)
            }
            _ => format!("{}.{} · Loading relation…", target.schema, target.relation),
        }
    });

    view! {
        <Drawer
            open=fk_open
            title=fk_title
            subtitle="Referenced row details"
            wide=true
            close_on_escape=false
            on_close=close_fk_drawer
        >
            {move || {
                if fk_loading.get() {
                    return view! {
                        <div class="table-page-loading">
                            {icons::icon_spinner(16)}
                            <span>"Loading…"</span>
                        </div>
                    }.into_any();
                }
                if let Some(err) = fk_error.get() {
                    return view! {
                        <div class="table-page-error"><p>{err}</p></div>
                    }.into_any();
                }
                match fk_row.get() {
                    Some(row) => referenced_row_content(row),
                    None => view! {
                        <p class="text-muted text-sm">"No data to display"</p>
                    }.into_any(),
                }
            }}
        </Drawer>

        <Drawer
            open=column_open
            title=column_title
            subtitle="Column information"
            context=column_context
            wide=true
            close_on_escape=false
            on_close=close_column_drawer
        >
            {move || column_panel_content(
                column_target.get(),
                status.get().unwrap_or(RelationStructureStatus::Missing),
                retry,
            )}
        </Drawer>
    }
}
