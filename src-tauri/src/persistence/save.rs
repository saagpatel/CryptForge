use std::time::{SystemTime, UNIX_EPOCH};

use rand::rngs::StdRng;
use rand::SeedableRng;
use rusqlite::Connection;

use super::database;
use crate::engine::state::World;

/// Get today's date as YYYY-MM-DD string using UTC.
pub fn today_date_string() -> String {
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    // Convert unix timestamp to date components
    let days = (secs / 86400) as i64;
    // Algorithm from http://howardhinnant.github.io/date_algorithms.html
    let z = days + 719468;
    let era = if z >= 0 { z } else { z - 146096 } / 146097;
    let doe = (z - era * 146097) as u64;
    let yoe = (doe - doe / 1460 + doe / 36524 - doe / 146096) / 365;
    let y = (yoe as i64) + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    let y = if m <= 2 { y + 1 } else { y };
    format!("{:04}-{:02}-{:02}", y, m, d)
}

/// Save the current game world to the database.
pub fn save_world(conn: &Connection, world: &World) -> Result<(), String> {
    let data = serde_json::to_vec(world).map_err(|e| format!("Serialize error: {}", e))?;
    database::save_game_state(conn, &data, world.seed, world.floor, world.turn)
}

/// Load the game world from the database.
/// Re-seeds the RNG from seed + turn to restore determinism.
pub fn load_world(conn: &Connection) -> Result<Option<World>, String> {
    let data = database::load_game_state(conn)?;
    match data {
        Some(bytes) => {
            let mut world: World =
                serde_json::from_slice(&bytes).map_err(|e| format!("Deserialize error: {}", e))?;

            // Re-seed RNG from seed + turn for determinism
            let combined_seed = world.seed.wrapping_add(world.turn as u64);
            world.rng = StdRng::seed_from_u64(combined_seed);

            Ok(Some(world))
        }
        None => Ok(None),
    }
}

/// Check if a save exists.
pub fn has_save(conn: &Connection) -> Result<bool, String> {
    let data = database::load_game_state(conn)?;
    Ok(data.is_some())
}

/// Delete the active save and record the run.
pub fn end_run(conn: &Connection, world: &World) -> Result<(), String> {
    let cause = if world.victory {
        None
    } else {
        world
            .last_damage_source
            .as_deref()
            .or(Some("Slain in the dungeon"))
    };

    let score = {
        let floor_score = world.floor * 100;
        let kill_score = world.enemies_killed * 10;
        let boss_score = world.bosses_killed * 500;
        let level_score = world.player_level * 50;
        let victory_bonus = if world.victory { 5000 } else { 0 };
        floor_score + kill_score + boss_score + level_score + victory_bonus
    };

    let class_str = format!("{:?}", world.player_class);
    let modifier_strs: Vec<String> = world.modifiers.iter().map(|m| format!("{:?}", m)).collect();
    database::record_run(
        conn,
        &world.seed.to_string(),
        world.floor,
        world.enemies_killed,
        world.bosses_killed,
        world.player_level,
        world.turn,
        score,
        cause,
        world.victory,
        &class_str,
        &modifier_strs,
    )?;

    // Track lifetime stats
    let _ = database::increment_stat(conn, "total_runs", 1);
    let _ = database::increment_stat(conn, "total_kills", world.enemies_killed as i64);
    let _ = database::increment_stat(conn, "total_bosses_killed", world.bosses_killed as i64);
    let _ = database::increment_stat(conn, "total_floors", world.floor as i64);
    let _ = database::increment_stat(conn, "total_turns", world.turn as i64);
    let _ = database::increment_stat(conn, "total_gold", world.gold as i64);
    if world.victory {
        let _ = database::increment_stat(conn, "total_victories", 1);
    } else if let Some(ref cause_str) = cause {
        let death_key = format!("deaths_by_{}", cause_str.to_lowercase().replace(' ', "_"));
        let _ = database::increment_stat(conn, &death_key, 1);
    }
    let class_key = format!("class_{}", class_str.to_lowercase());
    let _ = database::increment_stat(conn, &class_key, 1);

    // Record daily challenge result if this was a daily run
    if world.is_daily {
        let today = today_date_string();
        let _ = database::record_daily_result(
            conn,
            &today,
            &world.seed.to_string(),
            score,
            world.floor,
            world.victory,
        );
    }

    database::delete_save(conn)?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::persistence::database::init_schema;

    fn in_memory_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn save_and_load_round_trip() {
        let conn = in_memory_db();
        let world = World::new(42);
        let original_seed = world.seed;
        let original_floor = world.floor;
        let original_turn = world.turn;

        save_world(&conn, &world).unwrap();
        let loaded = load_world(&conn).unwrap();
        assert!(loaded.is_some());

        let loaded = loaded.unwrap();
        assert_eq!(loaded.seed, original_seed);
        assert_eq!(loaded.floor, original_floor);
        assert_eq!(loaded.turn, original_turn);
        assert_eq!(loaded.entities.len(), world.entities.len());
    }

    #[test]
    fn load_returns_none_when_no_save() {
        let conn = in_memory_db();
        let loaded = load_world(&conn).unwrap();
        assert!(loaded.is_none());
    }

    #[test]
    fn has_save_works() {
        let conn = in_memory_db();
        assert!(!has_save(&conn).unwrap());
        let world = World::new(42);
        save_world(&conn, &world).unwrap();
        assert!(has_save(&conn).unwrap());
    }

    #[test]
    fn end_run_deletes_save_and_records() {
        let conn = in_memory_db();
        let world = World::new(42);
        save_world(&conn, &world).unwrap();
        assert!(has_save(&conn).unwrap());

        end_run(&conn, &world).unwrap();
        assert!(!has_save(&conn).unwrap());

        // Should have a run in history
        let runs = database::get_run_history(&conn).unwrap();
        assert_eq!(runs.len(), 1);
    }

    #[test]
    fn overwrite_save() {
        let conn = in_memory_db();
        let mut world = World::new(42);
        save_world(&conn, &world).unwrap();

        world.floor = 5;
        world.turn = 100;
        save_world(&conn, &world).unwrap();

        let loaded = load_world(&conn).unwrap().unwrap();
        assert_eq!(loaded.floor, 5);
        assert_eq!(loaded.turn, 100);
    }
}
