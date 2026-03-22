// Temporary legacy lint allowlist; keep narrow and retire over time.
#![allow(
    clippy::collapsible_if,
    clippy::derivable_impls,
    clippy::doc_lazy_continuation,
    clippy::field_reassign_with_default,
    clippy::if_same_then_else,
    clippy::len_without_is_empty,
    clippy::let_and_return,
    clippy::manual_contains,
    clippy::manual_div_ceil,
    clippy::manual_flatten,
    clippy::manual_is_multiple_of,
    clippy::manual_range_contains,
    clippy::needless_borrow,
    clippy::needless_range_loop,
    clippy::new_without_default,
    clippy::ptr_arg,
    clippy::redundant_closure,
    clippy::single_match,
    clippy::too_many_arguments,
    clippy::unnecessary_cast,
    clippy::unnecessary_map_or,
    clippy::useless_format
)]

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
