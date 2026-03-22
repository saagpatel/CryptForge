use rusqlite::{params, Connection};
use std::collections::HashMap;
use std::path::Path;

#[allow(dead_code)]
const SCHEMA_VERSION: u32 = 2;

/// Open (or create) the database at the given path and run migrations.
pub fn open_database(path: &Path) -> Result<Connection, String> {
    let conn = Connection::open(path).map_err(|e| format!("Failed to open database: {}", e))?;

    // Enable WAL mode for better concurrency
    conn.execute_batch("PRAGMA journal_mode=WAL;")
        .map_err(|e| format!("Failed to set journal mode: {}", e))?;

    run_migrations(&conn)?;
    Ok(conn)
}

/// Initialize schema on an existing connection (useful for in-memory testing).
pub fn init_schema(conn: &Connection) -> Result<(), String> {
    run_migrations(conn)
}

fn run_migrations(conn: &Connection) -> Result<(), String> {
    // Create schema_version table if it doesn't exist
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS schema_version (
            version INTEGER PRIMARY KEY NOT NULL
        );",
    )
    .map_err(|e| format!("Migration error: {}", e))?;

    let current_version: u32 = conn
        .query_row(
            "SELECT COALESCE(MAX(version), 0) FROM schema_version",
            [],
            |row| row.get(0),
        )
        .unwrap_or(0);

    if current_version < 1 {
        migrate_v1(conn)?;
    }
    if current_version < 2 {
        migrate_v2(conn)?;
    }

    Ok(())
}

fn migrate_v1(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS save_state (
            id INTEGER PRIMARY KEY CHECK (id = 1),
            data BLOB NOT NULL,
            seed TEXT NOT NULL,
            floor INTEGER NOT NULL,
            turn INTEGER NOT NULL,
            updated_at TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS runs (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            seed TEXT NOT NULL,
            floor_reached INTEGER NOT NULL,
            enemies_killed INTEGER NOT NULL,
            bosses_killed INTEGER NOT NULL,
            level_reached INTEGER NOT NULL,
            turns_taken INTEGER NOT NULL,
            score INTEGER NOT NULL,
            cause_of_death TEXT,
            victory INTEGER NOT NULL DEFAULT 0,
            timestamp TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS high_scores (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            score INTEGER NOT NULL,
            floor_reached INTEGER NOT NULL,
            seed TEXT NOT NULL,
            victory INTEGER NOT NULL DEFAULT 0,
            timestamp TEXT NOT NULL DEFAULT (datetime('now'))
        );

        CREATE TABLE IF NOT EXISTS settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );

        INSERT OR REPLACE INTO schema_version (version) VALUES (1);",
    )
    .map_err(|e| format!("Migration v1 error: {}", e))?;

    Ok(())
}

fn migrate_v2(conn: &Connection) -> Result<(), String> {
    conn.execute_batch(
        "ALTER TABLE runs ADD COLUMN class TEXT DEFAULT 'Warrior';
         ALTER TABLE runs ADD COLUMN modifiers TEXT DEFAULT '[]';
         ALTER TABLE high_scores ADD COLUMN class TEXT DEFAULT 'Warrior';

         CREATE TABLE IF NOT EXISTS daily_challenges (
             date TEXT PRIMARY KEY,
             seed TEXT NOT NULL,
             score INTEGER,
             floor_reached INTEGER,
             completed INTEGER NOT NULL DEFAULT 0,
             timestamp TEXT NOT NULL DEFAULT (datetime('now'))
         );

         CREATE TABLE IF NOT EXISTS lifetime_stats (
             key TEXT PRIMARY KEY,
             value INTEGER NOT NULL DEFAULT 0
         );

         INSERT OR REPLACE INTO schema_version (version) VALUES (2);",
    )
    .map_err(|e| format!("Migration v2 error: {}", e))?;

    Ok(())
}

/// Check if an active save exists.
pub fn has_save(conn: &Connection) -> bool {
    conn.query_row("SELECT COUNT(*) FROM save_state", [], |row| {
        row.get::<_, i64>(0)
    })
    .unwrap_or(0)
        > 0
}

/// Save game state as a BLOB.
pub fn save_game_state(
    conn: &Connection,
    data: &[u8],
    seed: u64,
    floor: u32,
    turn: u32,
) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO save_state (id, data, seed, floor, turn, updated_at)
         VALUES (1, ?1, ?2, ?3, ?4, datetime('now'))",
        params![data, seed.to_string(), floor, turn],
    )
    .map_err(|e| format!("Save error: {}", e))?;
    Ok(())
}

/// Load game state BLOB.
pub fn load_game_state(conn: &Connection) -> Result<Option<Vec<u8>>, String> {
    let result = conn.query_row("SELECT data FROM save_state WHERE id = 1", [], |row| {
        row.get::<_, Vec<u8>>(0)
    });

    match result {
        Ok(data) => Ok(Some(data)),
        Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
        Err(e) => Err(format!("Load error: {}", e)),
    }
}

/// Delete the active save (on death or victory).
pub fn delete_save(conn: &Connection) -> Result<(), String> {
    conn.execute("DELETE FROM save_state", [])
        .map_err(|e| format!("Delete save error: {}", e))?;
    Ok(())
}

/// Record a completed run.
pub fn record_run(
    conn: &Connection,
    seed: &str,
    floor_reached: u32,
    enemies_killed: u32,
    bosses_killed: u32,
    level_reached: u32,
    turns_taken: u32,
    score: u32,
    cause_of_death: Option<&str>,
    victory: bool,
    class: &str,
    modifiers: &[String],
) -> Result<(), String> {
    let modifiers_json = serde_json::to_string(modifiers).unwrap_or_else(|_| "[]".to_string());
    conn.execute(
        "INSERT INTO runs (seed, floor_reached, enemies_killed, bosses_killed, level_reached, turns_taken, score, cause_of_death, victory, class, modifiers)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10, ?11)",
        params![seed, floor_reached, enemies_killed, bosses_killed, level_reached, turns_taken, score, cause_of_death, victory as i32, class, modifiers_json],
    ).map_err(|e| format!("Record run error: {}", e))?;

    // Also insert into high_scores
    conn.execute(
        "INSERT INTO high_scores (score, floor_reached, seed, victory, class)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![score, floor_reached, seed, victory as i32, class],
    )
    .map_err(|e| format!("Record high score error: {}", e))?;

    // Keep only top 10 high scores
    conn.execute(
        "DELETE FROM high_scores WHERE id NOT IN (SELECT id FROM high_scores ORDER BY score DESC LIMIT 10)",
        [],
    ).map_err(|e| format!("Prune high scores error: {}", e))?;

    Ok(())
}

/// Get top 10 high scores.
pub fn get_high_scores(conn: &Connection) -> Result<Vec<crate::engine::entity::HighScore>, String> {
    let mut stmt = conn
        .prepare("SELECT score, floor_reached, seed, victory, timestamp, COALESCE(class, 'Warrior') FROM high_scores ORDER BY score DESC LIMIT 10")
        .map_err(|e| format!("Query error: {}", e))?;

    let scores = stmt
        .query_map([], |row| {
            Ok(crate::engine::entity::HighScore {
                rank: 0, // filled in below
                score: row.get(0)?,
                floor_reached: row.get(1)?,
                seed: row.get(2)?,
                victory: row.get::<_, i32>(3)? != 0,
                timestamp: row.get(4)?,
                class: row.get(5)?,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?
        .filter_map(|r| r.ok())
        .enumerate()
        .map(|(i, mut s)| {
            s.rank = (i + 1) as u32;
            s
        })
        .collect();

    Ok(scores)
}

/// Get run history (most recent 50).
pub fn get_run_history(
    conn: &Connection,
) -> Result<Vec<crate::engine::entity::RunSummary>, String> {
    let mut stmt = conn
        .prepare("SELECT seed, floor_reached, enemies_killed, bosses_killed, level_reached, turns_taken, score, cause_of_death, victory, timestamp, COALESCE(class, 'Warrior'), COALESCE(modifiers, '[]') FROM runs ORDER BY id DESC LIMIT 50")
        .map_err(|e| format!("Query error: {}", e))?;

    let runs = stmt
        .query_map([], |row| {
            let modifiers_json: String = row.get(11)?;
            let modifiers: Vec<String> = serde_json::from_str(&modifiers_json).unwrap_or_default();
            Ok(crate::engine::entity::RunSummary {
                seed: row.get(0)?,
                floor_reached: row.get(1)?,
                enemies_killed: row.get(2)?,
                bosses_killed: row.get(3)?,
                level_reached: row.get(4)?,
                turns_taken: row.get(5)?,
                score: row.get(6)?,
                cause_of_death: row.get(7)?,
                victory: row.get::<_, i32>(8)? != 0,
                timestamp: row.get(9)?,
                class: row.get(10)?,
                modifiers,
            })
        })
        .map_err(|e| format!("Query error: {}", e))?
        .filter_map(|r| r.ok())
        .collect();

    Ok(runs)
}

/// Increment a lifetime stat by the given amount (UPSERT).
pub fn increment_stat(conn: &Connection, key: &str, amount: i64) -> Result<(), String> {
    conn.execute(
        "INSERT INTO lifetime_stats (key, value) VALUES (?1, ?2)
         ON CONFLICT(key) DO UPDATE SET value = value + ?2",
        params![key, amount],
    )
    .map_err(|e| format!("Increment stat error: {}", e))?;
    Ok(())
}

/// Read all lifetime stats as a HashMap.
pub fn get_all_stats(conn: &Connection) -> Result<HashMap<String, i64>, String> {
    let mut stmt = conn
        .prepare("SELECT key, value FROM lifetime_stats")
        .map_err(|e| format!("Query error: {}", e))?;

    let rows = stmt
        .query_map([], |row| {
            Ok((row.get::<_, String>(0)?, row.get::<_, i64>(1)?))
        })
        .map_err(|e| format!("Query error: {}", e))?;

    let mut map = HashMap::new();
    for row in rows {
        if let Ok((k, v)) = row {
            map.insert(k, v);
        }
    }
    Ok(map)
}

/// Check if a daily challenge has been played for the given date.
pub fn has_played_daily(conn: &Connection, date: &str) -> bool {
    conn.query_row(
        "SELECT COUNT(*) FROM daily_challenges WHERE date = ?1",
        params![date],
        |row| row.get::<_, i64>(0),
    )
    .unwrap_or(0)
        > 0
}

/// Record a daily challenge result.
pub fn record_daily_result(
    conn: &Connection,
    date: &str,
    seed: &str,
    score: u32,
    floor_reached: u32,
    completed: bool,
) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO daily_challenges (date, seed, score, floor_reached, completed, timestamp)
         VALUES (?1, ?2, ?3, ?4, ?5, datetime('now'))",
        params![date, seed, score, floor_reached, completed as i32],
    )
    .map_err(|e| format!("Record daily result error: {}", e))?;
    Ok(())
}

/// Get the daily challenge status for today's date.
pub fn get_daily_status(conn: &Connection, date: &str) -> crate::engine::entity::DailyStatus {
    let result = conn.query_row(
        "SELECT score, floor_reached FROM daily_challenges WHERE date = ?1",
        params![date],
        |row| Ok((row.get::<_, Option<u32>>(0)?, row.get::<_, Option<u32>>(1)?)),
    );

    match result {
        Ok((score, floor_reached)) => crate::engine::entity::DailyStatus {
            date: date.to_string(),
            played: true,
            score,
            floor_reached,
        },
        Err(_) => crate::engine::entity::DailyStatus {
            date: date.to_string(),
            played: false,
            score: None,
            floor_reached: None,
        },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        init_schema(&conn).unwrap();
        conn
    }

    #[test]
    fn schema_creates_tables() {
        let conn = test_db();
        // Verify all tables exist by querying them
        assert!(!has_save(&conn));
        let scores = get_high_scores(&conn).unwrap();
        assert!(scores.is_empty());
        let runs = get_run_history(&conn).unwrap();
        assert!(runs.is_empty());
    }

    #[test]
    fn save_and_load_game_state() {
        let conn = test_db();
        let data = b"test data".to_vec();
        save_game_state(&conn, &data, 42, 1, 10).unwrap();
        assert!(has_save(&conn));

        let loaded = load_game_state(&conn).unwrap().unwrap();
        assert_eq!(loaded, data);
    }

    #[test]
    fn delete_save_works() {
        let conn = test_db();
        save_game_state(&conn, b"data", 42, 1, 10).unwrap();
        assert!(has_save(&conn));
        delete_save(&conn).unwrap();
        assert!(!has_save(&conn));
    }

    #[test]
    fn record_run_and_history() {
        let conn = test_db();
        record_run(
            &conn,
            "42",
            5,
            10,
            1,
            3,
            50,
            1250,
            Some("Slain by goblin"),
            false,
            "Warrior",
            &[],
        )
        .unwrap();
        let runs = get_run_history(&conn).unwrap();
        assert_eq!(runs.len(), 1);
        assert_eq!(runs[0].floor_reached, 5);
        assert_eq!(runs[0].score, 1250);
        assert!(!runs[0].victory);
    }

    #[test]
    fn high_scores_ranked() {
        let conn = test_db();
        record_run(&conn, "1", 1, 5, 0, 1, 20, 500, None, false, "Warrior", &[]).unwrap();
        record_run(&conn, "2", 3, 15, 0, 2, 50, 1500, None, false, "Rogue", &[]).unwrap();
        record_run(&conn, "3", 10, 50, 3, 5, 200, 5000, None, true, "Mage", &[]).unwrap();

        let scores = get_high_scores(&conn).unwrap();
        assert_eq!(scores.len(), 3);
        assert_eq!(scores[0].score, 5000);
        assert_eq!(scores[0].rank, 1);
        assert_eq!(scores[1].score, 1500);
        assert_eq!(scores[2].score, 500);
    }

    #[test]
    fn high_scores_pruned_to_10() {
        let conn = test_db();
        for i in 0..15u32 {
            record_run(
                &conn,
                &i.to_string(),
                1,
                i,
                0,
                1,
                10,
                i * 100,
                None,
                false,
                "Warrior",
                &[],
            )
            .unwrap();
        }
        let scores = get_high_scores(&conn).unwrap();
        assert_eq!(scores.len(), 10);
    }

    #[test]
    fn idempotent_migrations() {
        let conn = test_db();
        // Running migrations again should not fail
        init_schema(&conn).unwrap();
        assert!(!has_save(&conn));
    }
}
