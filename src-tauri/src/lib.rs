mod db;
mod pg;

use db::DbState;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_store::Builder::default().build())
        .manage(DbState::new())
        .invoke_handler(tauri::generate_handler![
            db::test_connection,
            db::connect_to_db,
            db::get_tables,
            db::get_table_data,
            db::get_foreign_keys,
            db::get_indexes,
            db::get_roles,
            db::get_table_privileges,
            db::get_row_by_pk,
            db::disconnect_db,
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
