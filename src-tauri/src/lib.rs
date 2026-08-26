mod commands;
mod pg;
mod state;
mod types;

use state::DbState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(DbState::new())
        .invoke_handler(tauri::generate_handler![
            commands::test_connection,
            commands::connect_to_db,
            commands::get_tables,
            commands::get_table_data,
            commands::get_foreign_keys,
            commands::get_indexes,
            commands::get_roles,
            commands::get_table_privileges,
            commands::get_row_by_pk,
            commands::disconnect_db,
        ])
        .setup(|app| {
            // On Windows/Linux, remove native decorations so our custom title bar
            // is the only chrome. On macOS, decorations stay enabled because
            // `titleBarStyle: "overlay"` already hides the title bar while
            // preserving traffic lights and rounded corners.
            #[cfg(not(target_os = "macos"))]
            {
                use tauri::Manager;
                let window = app
                    .get_webview_window("main")
                    .expect("main window not found");
                let _ = window.set_decorations(false);
            }

            #[cfg(target_os = "macos")]
            let _ = app;

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
