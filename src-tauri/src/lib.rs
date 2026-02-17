mod db;

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
            db::get_row_by_pk,
            db::disconnect_db,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
