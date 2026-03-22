use rusqlite::{params, Connection};

use crate::engine::entity::Settings;

/// Load settings from the database, using defaults for missing keys.
pub fn load_settings(conn: &Connection) -> Settings {
    let mut settings = Settings::default();

    if let Ok(v) = get_setting(conn, "tile_size") {
        settings.tile_size = v.parse().unwrap_or(32);
    }
    if let Ok(v) = get_setting(conn, "master_volume") {
        settings.master_volume = v.parse().unwrap_or(80);
    }
    if let Ok(v) = get_setting(conn, "sfx_volume") {
        settings.sfx_volume = v.parse().unwrap_or(80);
    }
    if let Ok(v) = get_setting(conn, "ambient_volume") {
        settings.ambient_volume = v.parse().unwrap_or(50);
    }
    if let Ok(v) = get_setting(conn, "fullscreen") {
        settings.fullscreen = v == "true";
    }
    if let Ok(v) = get_setting(conn, "ollama_enabled") {
        settings.ollama_enabled = v == "true";
    }
    if let Ok(v) = get_setting(conn, "ollama_url") {
        settings.ollama_url = v;
    }
    if let Ok(v) = get_setting(conn, "ollama_model") {
        settings.ollama_model = v;
    }
    if let Ok(v) = get_setting(conn, "ollama_timeout") {
        settings.ollama_timeout = v.parse().unwrap_or(3);
    }
    if let Ok(v) = get_setting(conn, "tileset_mode") {
        settings.tileset_mode = v;
    }

    settings
}

/// Save settings to the database.
pub fn save_settings(conn: &Connection, settings: &Settings) -> Result<(), String> {
    set_setting(conn, "tile_size", &settings.tile_size.to_string())?;
    set_setting(conn, "master_volume", &settings.master_volume.to_string())?;
    set_setting(conn, "sfx_volume", &settings.sfx_volume.to_string())?;
    set_setting(conn, "ambient_volume", &settings.ambient_volume.to_string())?;
    set_setting(
        conn,
        "fullscreen",
        if settings.fullscreen { "true" } else { "false" },
    )?;
    set_setting(
        conn,
        "ollama_enabled",
        if settings.ollama_enabled {
            "true"
        } else {
            "false"
        },
    )?;
    set_setting(conn, "ollama_url", &settings.ollama_url)?;
    set_setting(conn, "ollama_model", &settings.ollama_model)?;
    set_setting(conn, "ollama_timeout", &settings.ollama_timeout.to_string())?;
    set_setting(conn, "tileset_mode", &settings.tileset_mode)?;
    Ok(())
}

fn get_setting(conn: &Connection, key: &str) -> Result<String, String> {
    conn.query_row(
        "SELECT value FROM settings WHERE key = ?1",
        params![key],
        |row| row.get(0),
    )
    .map_err(|e| format!("Get setting '{}': {}", key, e))
}

fn set_setting(conn: &Connection, key: &str, value: &str) -> Result<(), String> {
    conn.execute(
        "INSERT OR REPLACE INTO settings (key, value) VALUES (?1, ?2)",
        params![key, value],
    )
    .map_err(|e| format!("Set setting '{}': {}", key, e))?;
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
    fn load_defaults_when_empty() {
        let conn = in_memory_db();
        let settings = load_settings(&conn);
        assert_eq!(settings.tile_size, 32);
        assert_eq!(settings.master_volume, 80);
        assert_eq!(settings.sfx_volume, 80);
        assert_eq!(settings.ambient_volume, 50);
        assert!(!settings.fullscreen);
        assert!(!settings.ollama_enabled);
    }

    #[test]
    fn save_and_load_round_trip() {
        let conn = in_memory_db();
        let mut settings = Settings::default();
        settings.master_volume = 50;
        settings.sfx_volume = 30;
        settings.fullscreen = true;
        settings.ollama_enabled = true;
        settings.ollama_model = "mistral".to_string();

        save_settings(&conn, &settings).unwrap();
        let loaded = load_settings(&conn);

        assert_eq!(loaded.master_volume, 50);
        assert_eq!(loaded.sfx_volume, 30);
        assert!(loaded.fullscreen);
        assert!(loaded.ollama_enabled);
        assert_eq!(loaded.ollama_model, "mistral");
    }

    #[test]
    fn overwrite_settings() {
        let conn = in_memory_db();
        let mut settings = Settings::default();
        settings.master_volume = 100;
        save_settings(&conn, &settings).unwrap();

        settings.master_volume = 25;
        save_settings(&conn, &settings).unwrap();

        let loaded = load_settings(&conn);
        assert_eq!(loaded.master_volume, 25);
    }
}
