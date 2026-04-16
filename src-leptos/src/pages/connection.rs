use leptos::prelude::*;
use leptos::task::spawn_local;
use wasm_bindgen::JsCast;

use crate::stores::connection_store::ConnectionStore;
use crate::stores::db_store::DbStore;
use crate::components::icons;
use crate::tauri;
use crate::types::{ConnectionConfig, ConnectionResult};

#[derive(Clone, Debug)]
enum Status {
    Idle,
    Testing,
    Connecting,
    Success(String),
    Error(String),
}

#[component]
pub fn ConnectionPage() -> impl IntoView {
    let conn_store = use_context::<ConnectionStore>().expect("ConnectionStore not provided");
    let db_store = use_context::<DbStore>().expect("DbStore not provided");

    let (host, set_host) = signal("localhost".to_string());
    let (port, set_port) = signal("5432".to_string());
    let (database, set_database) = signal(String::new());
    let (username, set_username) = signal(String::new());
    let (password, set_password) = signal(String::new());
    let (status, set_status) = signal(Status::Idle);

    let is_loading =
        Memo::new(move |_| matches!(status.get(), Status::Testing | Status::Connecting));

    let build_config = move || ConnectionConfig {
        host: host.get(),
        port: port.get(),
        database: database.get(),
        username: username.get(),
        password: password.get(),
    };

    let on_test = move |_| {
        let config = build_config();
        set_status.set(Status::Testing);
        spawn_local(async move {
            match tauri::invoke::<ConnectionResult>(
                "test_connection",
                &serde_json::json!({ "config": config }),
            )
            .await
            {
                Ok(result) if result.success => {
                    set_status.set(Status::Success(result.message));
                }
                Ok(result) => {
                    set_status.set(Status::Error(result.message));
                }
                Err(e) => {
                    set_status.set(Status::Error(format!("Failed to test: {}", e)));
                }
            }
        });
    };

    let on_connect = move |ev: web_sys::SubmitEvent| {
        ev.prevent_default();
        let config = build_config();
        set_status.set(Status::Connecting);
        spawn_local(async move {
            match tauri::invoke::<ConnectionResult>(
                "connect_to_db",
                &serde_json::json!({ "config": config }),
            )
            .await
            {
                Ok(result) if result.success => {
                    set_status.set(Status::Success(result.message));
                    conn_store.save_connection(
                        config.host.clone(),
                        config.port.clone(),
                        config.database.clone(),
                        config.username.clone(),
                    );
                    db_store.fetch_database_metadata();
                }
                Ok(result) => {
                    set_status.set(Status::Error(result.message));
                }
                Err(e) => {
                    set_status.set(Status::Error(format!("Failed to connect: {}", e)));
                }
            }
        });
    };

    let val = |ev: web_sys::Event| -> String {
        ev.target()
            .unwrap()
            .unchecked_into::<web_sys::HtmlInputElement>()
            .value()
    };

    view! {
        <div class="connection-page">
            <div class="connection-layout">

                /* Recents sidebar */
                {move || {
                    let conns = conn_store.connections.get();
                    let loaded = conn_store.is_loaded.get();

                    if loaded && !conns.is_empty() {
                        let items: Vec<_> = conns.iter().map(|conn| {
                            let c_host = conn.host.clone();
                            let c_port = conn.port.clone();
                            let c_db = conn.database.clone();
                            let c_user = conn.username.clone();
                            let c_id = conn.id.clone();
                            let label = conn.database.clone();
                            view! {
                                <div class="recent-item">
                                    <button class="btn btn-ghost"
                                        on:click=move |_| {
                                            set_host.set(c_host.clone());
                                            set_port.set(c_port.clone());
                                            set_database.set(c_db.clone());
                                            set_username.set(c_user.clone());
                                            set_password.set(String::new());
                                            set_status.set(Status::Idle);
                                        }
                                    >{label}</button>
                                    <button class="recent-delete" title="Remove"
                                        on:click={
                                            let id = c_id.clone();
                                            move |_| conn_store.remove_connection(id.clone())
                                        }
                                    >
                                        <svg xmlns="http://www.w3.org/2000/svg" width="14" height="14" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                            <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                                            <path d="M4 7l16 0"/>
                                            <path d="M10 11l0 6"/>
                                            <path d="M14 11l0 6"/>
                                            <path d="M5 7l1 12a2 2 0 0 0 2 2h8a2 2 0 0 0 2 -2l1 -12"/>
                                            <path d="M9 7v-3a1 1 0 0 1 1 -1h4a1 1 0 0 1 1 1v3"/>
                                        </svg>
                                    </button>
                                </div>
                            }
                        }).collect();
                        view! {
                            <div class="recents">
                                <h3 class="recents-heading">"Recents"</h3>
                                {items}
                            </div>
                        }.into_any()
                    } else {
                        view! { <div style="display:none"/> }.into_any()
                    }
                }}

                /* Connection form card */
                <div class="connection-card card">
                    <div class="card-header">
                        <div class="connection-title-row">
                            {icons::icon_database(24)}
                            <h1>"Connect to PostgreSQL"</h1>
                        </div>
                        <p class="card-description">"Enter your database connection details to get started"</p>
                    </div>

                    <div class="card-content">
                        <form on:submit=on_connect>
                            <div class="field">
                                <label for="host">"Host"</label>
                                <input type="text" id="host" placeholder="localhost" prop:value=host on:input=move |ev| set_host.set(val(ev)) required />
                            </div>
                            <div class="field">
                                <label for="port">"Port"</label>
                                <input type="text" id="port" placeholder="5432" prop:value=port on:input=move |ev| set_port.set(val(ev)) required />
                            </div>
                            <div class="field">
                                <label for="database">"Database"</label>
                                <input type="text" id="database" placeholder="my_database" prop:value=database on:input=move |ev| set_database.set(val(ev)) required />
                            </div>
                            <div class="field">
                                <label for="username">"Username"</label>
                                <input type="text" id="username" placeholder="postgres" prop:value=username on:input=move |ev| set_username.set(val(ev)) required />
                            </div>
                            <div class="field">
                                <label for="password">"Password"</label>
                                <input type="password" id="password" placeholder="Enter your password" prop:value=password on:input=move |ev| set_password.set(val(ev)) required />
                            </div>

                            {move || {
                                let s = status.get();
                                match s {
                                    Status::Success(ref msg) => Some(view! {
                                        <div class="status-msg status-success">
                                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                                                <path d="M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0 -18 0"/>
                                                <path d="M9 12l2 2l4 -4"/>
                                            </svg>
                                            <span>{msg.clone()}</span>
                                        </div>
                                    }.into_any()),
                                    Status::Error(ref msg) => Some(view! {
                                        <div class="status-msg status-error">
                                            <svg xmlns="http://www.w3.org/2000/svg" width="16" height="16" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
                                                <path stroke="none" d="M0 0h24v24H0z" fill="none"/>
                                                <path d="M12 12m-9 0a9 9 0 1 0 18 0a9 9 0 1 0 -18 0"/>
                                                <path d="M10 10l4 4m0 -4l-4 4"/>
                                            </svg>
                                            <span>{msg.clone()}</span>
                                        </div>
                                    }.into_any()),
                                    _ => None,
                                }
                            }}

                            <div class="connection-actions">
                                <button type="button" class="btn btn-secondary" on:click=on_test disabled=is_loading>
                                    {move || if matches!(status.get(), Status::Testing) {
                                        view! { {icons::icon_spinner(16)} }.into_any()
                                    } else { view! { <span/> }.into_any() }}
                                    "Test"
                                </button>
                                <button type="submit" class="btn btn-primary" disabled=is_loading>
                                    {move || if matches!(status.get(), Status::Connecting) {
                                        view! { {icons::icon_spinner(16)} }.into_any()
                                    } else { view! { <span/> }.into_any() }}
                                    "Connect"
                                </button>
                            </div>
                        </form>
                    </div>
                </div>

            </div>
        </div>
    }
}
