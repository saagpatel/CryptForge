pub mod commands;
pub mod engine;
pub mod flavor;
pub mod persistence;

use std::sync::Mutex;
use tauri::Manager;

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .setup(|app| {
            let app_dir = app
                .path()
                .app_data_dir()
                .expect("Failed to get app data dir");
            std::fs::create_dir_all(&app_dir).expect("Failed to create app data dir");

            let db_path = app_dir.join("cryptforge.db");
            let conn =
                persistence::database::open_database(&db_path).expect("Failed to open database");
            engine::achievements::ensure_table(&conn);

            app.manage(commands::AppState {
                world: Mutex::new(None),
                db: Mutex::new(conn),
            });

            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            commands::new_game,
            commands::player_action,
            commands::get_game_state,
            commands::save_game,
            commands::load_game,
            commands::inspect_entity,
            commands::get_run_history,
            commands::get_high_scores,
            commands::get_settings,
            commands::update_settings,
            commands::has_save_game,
            commands::check_ollama,
            commands::get_adjacent_shop,
            commands::get_achievements,
            commands::get_unlockables,
            commands::get_statistics,
            commands::start_daily_challenge,
            commands::get_daily_status,
        ])
        .run(tauri::generate_context!())
        .expect("error while running CryptForge");
}
