use std::collections::HashMap;

use leptos::prelude::*;

use crate::db_store::DbStore;
use crate::drawer::Drawer;
use crate::types::{RoleInfo, TablePrivilege};

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

fn format_conn_limit(limit: i32) -> String {
    if limit == -1 {
        "unlimited".to_string()
    } else {
        limit.to_string()
    }
}

fn format_valid_until(val: &Option<String>) -> String {
    match val {
        Some(s) if !s.is_empty() => s.clone(),
        _ => "\u{2014}".to_string(), // em-dash
    }
}

#[derive(Clone)]
struct PermSummary {
    label: String,
    kind: &'static str, // "superuser" | "read-write" | "read-only" | "mixed" | "none"
}

fn permission_summary(privileges: &[TablePrivilege], is_superuser: bool) -> PermSummary {
    if is_superuser {
        return PermSummary {
            label: "superuser".into(),
            kind: "superuser",
        };
    }
    if privileges.is_empty() {
        return PermSummary {
            label: "no table access".into(),
            kind: "none",
        };
    }

    let mut has_select = false;
    let mut has_write = false;
    for p in privileges {
        if p.privileges.iter().any(|s| s == "SELECT") {
            has_select = true;
        }
        if p.privileges
            .iter()
            .any(|s| s == "INSERT" || s == "UPDATE" || s == "DELETE")
        {
            has_write = true;
        }
    }

    let n = privileges.len();
    let tables_text = format!("{n} table{}", if n != 1 { "s" } else { "" });

    match (has_select, has_write) {
        (true, true) => PermSummary {
            label: format!("read-write ({tables_text})"),
            kind: "read-write",
        },
        (true, false) => PermSummary {
            label: format!("read-only ({tables_text})"),
            kind: "read-only",
        },
        (false, true) => PermSummary {
            label: format!("write-only ({tables_text})"),
            kind: "mixed",
        },
        _ => PermSummary {
            label: format!("mixed ({tables_text})"),
            kind: "mixed",
        },
    }
}

fn perm_badge_class(kind: &str) -> &'static str {
    match kind {
        "superuser" => "badge badge-danger",
        "read-write" => "badge badge-success",
        "read-only" => "badge badge-info",
        "none" => "badge",
        _ => "badge badge-warn",
    }
}

fn priv_badge_class(priv_name: &str) -> &'static str {
    match priv_name {
        "SELECT" => "badge badge-success",
        "INSERT" => "badge badge-info",
        "UPDATE" => "badge badge-warn",
        "DELETE" => "badge badge-danger",
        _ => "badge",
    }
}

// ---------------------------------------------------------------------------
// Role detail side panel
// ---------------------------------------------------------------------------

#[component]
fn RoleDetailPanel(
    role: RoleInfo,
    privileges: Vec<TablePrivilege>,
) -> impl IntoView {
    // Group privileges by schema.table
    let grouped: HashMap<String, Vec<String>> = {
        let mut map: HashMap<String, Vec<String>> = HashMap::new();
        for p in &privileges {
            let key = format!("{}.{}", p.table_schema, p.table_name);
            map.entry(key).or_default().extend(p.privileges.clone());
        }
        map
    };

    view! {
        <div class="role-detail-sections">
            // Properties
            <div class="role-detail-section">
                <h4 class="role-detail-heading">"Role Properties"</h4>
                <div class="role-prop-list">
                    <div class="role-prop"><span class="text-muted">"Can Login"</span><span>{if role.can_login { "Yes" } else { "No" }}</span></div>
                    <div class="role-prop"><span class="text-muted">"Superuser"</span><span class={if role.is_superuser { "text-danger" } else { "" }}>{if role.is_superuser { "Yes" } else { "No" }}</span></div>
                    <div class="role-prop"><span class="text-muted">"Create Database"</span><span>{if role.can_create_db { "Yes" } else { "No" }}</span></div>
                    <div class="role-prop"><span class="text-muted">"Create Role"</span><span>{if role.can_create_role { "Yes" } else { "No" }}</span></div>
                    <div class="role-prop"><span class="text-muted">"Connection Limit"</span><span class="mono">{format_conn_limit(role.connection_limit)}</span></div>
                    <div class="role-prop"><span class="text-muted">"Valid Until"</span><span>{format_valid_until(&role.valid_until)}</span></div>
                </div>
            </div>

            // Member of
            {if !role.member_of.is_empty() {
                let badges: Vec<_> = role.member_of.iter().map(|g| {
                    view! { <span class="badge badge-info">{g.clone()}</span> }
                }).collect();
                Some(view! {
                    <div class="role-detail-section">
                        <h4 class="role-detail-heading">"Member Of"</h4>
                        <div class="badge-row">{badges}</div>
                    </div>
                })
            } else {
                None
            }}

            // Table permissions
            <div class="role-detail-section">
                <h4 class="role-detail-heading">
                    "Table Permissions"
                    {if !privileges.is_empty() {
                        let n = privileges.len();
                        Some(view! {
                            <span class="text-muted" style="font-weight:400">
                                {format!(" ({n} table{})", if n != 1 { "s" } else { "" })}
                            </span>
                        })
                    } else {
                        None
                    }}
                </h4>
                {if role.is_superuser {
                    view! { <p class="text-muted text-sm" style="font-style:italic">"Superuser has full access to all tables"</p> }.into_any()
                } else if privileges.is_empty() {
                    view! { <p class="text-muted text-sm" style="font-style:italic">"No direct table permissions"</p> }.into_any()
                } else {
                    let entries: Vec<_> = grouped.into_iter().map(|(table, privs)| {
                        let priv_badges: Vec<_> = privs.iter().map(|p| {
                            let cls = priv_badge_class(p);
                            view! { <span class=cls>{p.clone()}</span> }
                        }).collect();
                        view! {
                            <div class="priv-row">
                                <span class="mono text-sm">{table}</span>
                                <div class="badge-row">{priv_badges}</div>
                            </div>
                        }
                    }).collect();
                    view! { <div class="priv-list">{entries}</div> }.into_any()
                }}
            </div>
        </div>
    }
}

// ---------------------------------------------------------------------------
// Roles table renderer (shared between Users and Groups sections)
// ---------------------------------------------------------------------------

fn render_role_row(
    role: &RoleInfo,
    summary: &PermSummary,
    show_extras: bool,
    on_click: impl Fn() + 'static,
) -> impl IntoView {
    let name = role.role_name.clone();
    let is_su = role.is_superuser;
    let badge_cls = perm_badge_class(summary.kind);
    let summary_label = summary.label.clone();

    let member_badges: Vec<_> = role
        .member_of
        .iter()
        .map(|g| view! { <span class="badge">{g.clone()}</span> })
        .collect();
    let has_members = !member_badges.is_empty();

    let conn_limit = format_conn_limit(role.connection_limit);
    let valid = format_valid_until(&role.valid_until);

    view! {
        <tr class="data-table-row">
            <td class="data-table-td">
                <button class="fk-link mono" on:click=move |_| on_click()>
                    {name}
                </button>
                {if is_su {
                    Some(view! { <span class="badge badge-danger" style="margin-left:0.5rem">"admin"</span> })
                } else {
                    None
                }}
            </td>
            <td class="data-table-td">
                <span class=badge_cls>{summary_label}</span>
            </td>
            {if show_extras {
                Some(view! {
                    <td class="data-table-td">
                        {if has_members {
                            view! { <div class="badge-row">{member_badges}</div> }.into_any()
                        } else {
                            view! { <span class="text-muted">{"\u{2014}"}</span> }.into_any()
                        }}
                    </td>
                    <td class="data-table-td cell-right mono">{conn_limit}</td>
                    <td class="data-table-td cell-right">{valid}</td>
                })
            } else {
                None
            }}
        </tr>
    }
}

// ---------------------------------------------------------------------------
// RolesPage component
// ---------------------------------------------------------------------------

#[component]
pub fn RolesPage() -> impl IntoView {
    let db = use_context::<DbStore>().expect("DbStore not provided");

    let roles = db.roles.get_untracked();

    // Side panel state
    let panel_open = RwSignal::new(false);
    let panel_title = RwSignal::new(String::new());
    let selected_role = RwSignal::new(None::<String>);

    if roles.is_empty() {
        return view! {
            <div class="empty-state">
                <div class="empty-state-inner">
                    <svg class="empty-state-icon" xmlns="http://www.w3.org/2000/svg" viewBox="0 0 24 24"
                         fill="none" stroke="currentColor" stroke-width="1.5"
                         stroke-linecap="round" stroke-linejoin="round">
                        <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                        <circle cx="9" cy="7" r="4"/>
                        <path d="M3 21v-2a4 4 0 0 1 4 -4h4a4 4 0 0 1 4 4v2"/>
                        <path d="M16 3.13a4 4 0 0 1 0 7.75"/>
                        <path d="M21 21v-2a4 4 0 0 0 -3 -3.85"/>
                    </svg>
                    <h3>"No roles found"</h3>
                    <p>"Connect to a database to view roles"</p>
                </div>
            </div>
        }
        .into_any();
    }

    // Split into users (can_login) and groups
    let users: Vec<RoleInfo> = roles.iter().filter(|r| r.can_login).cloned().collect();
    let groups: Vec<RoleInfo> = roles.iter().filter(|r| !r.can_login).cloned().collect();
    let user_count = users.len();
    let group_count = groups.len();

    // Build permission summaries
    let summaries: HashMap<String, PermSummary> = roles
        .iter()
        .map(|r| {
            let privs = db.get_privileges_for_role(&r.role_name);
            let s = permission_summary(&privs, r.is_superuser);
            (r.role_name.clone(), s)
        })
        .collect();

    // Build user rows
    let user_rows: Vec<_> = users
        .iter()
        .map(|role| {
            let summary = summaries.get(&role.role_name).cloned().unwrap_or(PermSummary {
                label: "unknown".into(),
                kind: "none",
            });
            let rn = role.role_name.clone();
            render_role_row(role, &summary, true, move || {
                panel_title.set(rn.clone());
                selected_role.set(Some(rn.clone()));
                panel_open.set(true);
            })
        })
        .collect();

    // Build group rows
    let group_rows: Vec<_> = groups
        .iter()
        .map(|role| {
            let summary = summaries.get(&role.role_name).cloned().unwrap_or(PermSummary {
                label: "unknown".into(),
                kind: "none",
            });
            let rn = role.role_name.clone();
            render_role_row(role, &summary, false, move || {
                panel_title.set(rn.clone());
                selected_role.set(Some(rn.clone()));
                panel_open.set(true);
            })
        })
        .collect();

    // Clone roles for the detail panel
    let all_roles = roles.clone();

    view! {
        <div class="table-page">
            // Header
            <div class="table-page-header">
                <h2 class="table-page-title">"Roles"</h2>
                <span class="text-muted text-sm">
                    {format!(
                        "{user_count} user{}, {group_count} group{}",
                        if user_count != 1 { "s" } else { "" },
                        if group_count != 1 { "s" } else { "" },
                    )}
                </span>
            </div>

            // Scrollable body
            <div class="table-page-body roles-body">
                // Users section
                <div class="roles-section">
                    <h3 class="roles-section-title">
                        "Users "
                        <span class="text-muted" style="font-weight:400">"(can login)"</span>
                    </h3>
                    {if user_rows.is_empty() {
                        view! { <p class="text-muted text-sm" style="font-style:italic">"No login users found"</p> }.into_any()
                    } else {
                        view! {
                            <div class="data-table-wrap">
                                <table class="data-table">
                                    <thead>
                                        <tr>
                                            <th class="data-table-th">"User Name"</th>
                                            <th class="data-table-th">"Permissions"</th>
                                            <th class="data-table-th">"Member Of"</th>
                                            <th class="data-table-th cell-right">"Conn Limit"</th>
                                            <th class="data-table-th cell-right">"Valid Until"</th>
                                        </tr>
                                    </thead>
                                    <tbody>{user_rows}</tbody>
                                </table>
                            </div>
                        }.into_any()
                    }}
                </div>

                // Groups section
                <div class="roles-section">
                    <h3 class="roles-section-title">
                        "Groups "
                        <span class="text-muted" style="font-weight:400">"(cannot login)"</span>
                    </h3>
                    {if group_rows.is_empty() {
                        view! { <p class="text-muted text-sm" style="font-style:italic">"No group roles found"</p> }.into_any()
                    } else {
                        view! {
                            <div class="data-table-wrap">
                                <table class="data-table">
                                    <thead>
                                        <tr>
                                            <th class="data-table-th">"Group Name"</th>
                                            <th class="data-table-th">"Permissions"</th>
                                        </tr>
                                    </thead>
                                    <tbody>{group_rows}</tbody>
                                </table>
                            </div>
                        }.into_any()
                    }}
                </div>
            </div>

            // Role detail side panel
            <Drawer open=panel_open title=panel_title subtitle="Role details and permissions">
                {move || {
                    let sel = selected_role.get()?;
                    let role = all_roles.iter().find(|r| r.role_name == sel)?.clone();
                    let privs = db.get_privileges_for_role(&sel);
                    Some(view! { <RoleDetailPanel role=role privileges=privs /> })
                }}
            </Drawer>
        </div>
    }
    .into_any()
}
