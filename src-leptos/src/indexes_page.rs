use leptos::prelude::*;

use crate::db_store::DbStore;
use crate::types::IndexInfo;

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn format_bytes(bytes: i64) -> String {
    if bytes == 0 {
        return "0 B".to_string();
    }
    const K: f64 = 1024.0;
    const SIZES: &[&str] = &["B", "KB", "MB", "GB"];
    let i = (bytes as f64).ln() / K.ln();
    let i = i.floor() as usize;
    let i = i.min(SIZES.len() - 1);
    let val = bytes as f64 / K.powi(i as i32);
    format!("{:.1} {}", val, SIZES[i])
}

/// Flatten all per-table index maps into a single sorted Vec.
fn collect_indexes(db: &DbStore) -> Vec<IndexInfo> {
    let tables = db.tables.get_untracked();
    let index_map = db.indexes.get_untracked();

    let mut all: Vec<IndexInfo> = tables
        .iter()
        .flat_map(|t| {
            let key = format!("{}.{}", t.schema, t.name);
            index_map.get(&key).cloned().unwrap_or_default()
        })
        .collect();

    all.sort_by(|a, b| {
        a.table_schema
            .cmp(&b.table_schema)
            .then_with(|| a.table_name.cmp(&b.table_name))
            .then_with(|| a.index_name.cmp(&b.index_name))
    });

    all
}

// ---------------------------------------------------------------------------
// Component
// ---------------------------------------------------------------------------

#[component]
pub fn IndexesPage() -> impl IntoView {
    let db = use_context::<DbStore>().expect("DbStore not provided");

    let all_indexes = collect_indexes(&db);
    let table_count = db.tables.get_untracked().len();

    if all_indexes.is_empty() {
        return view! {
            <div class="empty-state">
                <div class="empty-state-inner">
                    <svg class="empty-state-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"
                         fill="none" stroke="currentColor" stroke-width="1.5"
                         stroke-linecap="round" stroke-linejoin="round">
                        <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                        <path d="M9 6l11 0"/>
                        <path d="M9 12l11 0"/>
                        <path d="M9 18l11 0"/>
                        <path d="M5 6l0 .01"/>
                        <path d="M5 12l0 .01"/>
                        <path d="M5 18l0 .01"/>
                    </svg>
                    <h3>"No indexes found"</h3>
                    <p>"Connect to a database to view indexes"</p>
                </div>
            </div>
        }
        .into_any();
    }

    let index_count = all_indexes.len();

    let rows: Vec<_> = all_indexes
        .into_iter()
        .map(|idx| {
            let cols_display = idx.columns.join(", ");
            let size = format_bytes(idx.size_bytes);
            let unique_class = if idx.is_unique { "bool-yes" } else { "bool-no" };
            let unique_text = if idx.is_unique { "\u{2713}" } else { "\u{2014}" };
            let primary_class = if idx.is_primary {
                "bool-yes bool-primary"
            } else {
                "bool-no"
            };
            let primary_text = if idx.is_primary { "\u{2713}" } else { "\u{2014}" };

            view! {
                <tr class="data-table-row">
                    <td class="data-table-td mono">{idx.index_name}</td>
                    <td class="data-table-td">
                        <span class="text-muted">{idx.table_schema}"."</span>
                        {idx.table_name}
                    </td>
                    <td class="data-table-td mono">{cols_display}</td>
                    <td class="data-table-td">
                        <span class="badge">{idx.index_type}</span>
                    </td>
                    <td class="data-table-td cell-center">
                        <span class=unique_class>{unique_text}</span>
                    </td>
                    <td class="data-table-td cell-center">
                        <span class=primary_class>{primary_text}</span>
                    </td>
                    <td class="data-table-td cell-right mono">{size}</td>
                </tr>
            }
        })
        .collect();

    view! {
        <div class="table-page">
            <div class="table-page-header">
                <h2 class="table-page-title">"Indexes"</h2>
                <span class="text-muted text-sm">
                    {format!(
                        "{index_count} index{} across {table_count} table{}",
                        if index_count != 1 { "es" } else { "" },
                        if table_count != 1 { "s" } else { "" },
                    )}
                </span>
            </div>
            <div class="table-page-body">
                <div class="data-table-wrap">
                    <table class="data-table">
                        <thead>
                            <tr>
                                <th class="data-table-th">"Index Name"</th>
                                <th class="data-table-th">"Table"</th>
                                <th class="data-table-th">"Columns"</th>
                                <th class="data-table-th">"Type"</th>
                                <th class="data-table-th cell-center">"Unique"</th>
                                <th class="data-table-th cell-center">"Primary"</th>
                                <th class="data-table-th cell-right">"Size"</th>
                            </tr>
                        </thead>
                        <tbody>
                            {rows}
                        </tbody>
                    </table>
                </div>
            </div>
        </div>
    }
    .into_any()
}
