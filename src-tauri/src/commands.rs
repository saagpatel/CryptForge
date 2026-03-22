use std::sync::Mutex;

use rusqlite::Connection;
use tauri::State;

use crate::engine::achievements;
use crate::engine::entity::*;
use crate::engine::state::World;
use crate::persistence::{config, database, save};

pub struct AppState {
    pub world: Mutex<Option<World>>,
    pub db: Mutex<Connection>,
}

#[tauri::command]
pub fn new_game(
    seed: Option<String>,
    class: Option<String>,
    modifiers: Option<Vec<String>>,
    state: State<'_, AppState>,
) -> Result<TurnResult, String> {
    let seed_val: u64 = match seed {
        Some(s) if !s.is_empty() => s.parse().unwrap_or_else(|_| {
            // Hash the string to get a seed
            let mut h: u64 = 5381;
            for c in s.bytes() {
                h = h.wrapping_mul(33).wrapping_add(c as u64);
            }
            h
        }),
        _ => {
            // Random seed from system time
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map(|d| d.as_millis() as u64)
                .unwrap_or(42)
        }
    };

    let player_class = match class.as_deref() {
        Some("Rogue") => PlayerClass::Rogue,
        Some("Mage") => PlayerClass::Mage,
        _ => PlayerClass::Warrior,
    };

    let run_modifiers: Vec<RunModifier> = modifiers
        .unwrap_or_default()
        .iter()
        .filter_map(|m| match m.as_str() {
            "GlassCannon" => Some(RunModifier::GlassCannon),
            "Marathon" => Some(RunModifier::Marathon),
            "Pacifist" => Some(RunModifier::Pacifist),
            "Cursed" => Some(RunModifier::Cursed),
            _ => None,
        })
        .collect();

    let mut world = World::new_with_class(seed_val, player_class, run_modifiers);

    // Add unlocked achievement rewards to starting inventory
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        let rewards = achievements::get_unlocked_rewards(&db);
        if !rewards.is_empty() {
            world.add_unlocked_rewards(rewards);
        }
    }

    let result = world.build_turn_result(Vec::new());

    *state.world.lock().map_err(|e| e.to_string())? = Some(world);

    Ok(result)
}

#[tauri::command]
pub fn get_statistics(
    state: State<'_, AppState>,
) -> Result<std::collections::HashMap<String, i64>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    database::get_all_stats(&db)
}

#[tauri::command]
pub fn player_action(
    action: PlayerAction,
    state: State<'_, AppState>,
) -> Result<TurnResult, String> {
    let mut world_lock = state.world.lock().map_err(|e| e.to_string())?;
    let world = world_lock.as_mut().ok_or("No active game")?;

    let mut result = world.resolve_turn(action);

    // Check achievements
    match state.db.lock() {
        Ok(db) => {
            let unlocked = achievements::check_achievements(world, &result.events, &db);
            for name in unlocked {
                result.events.push(GameEvent::AchievementUnlocked { name });
            }
        }
        Err(e) => eprintln!("Failed to lock db for achievements: {e}"),
    }

    // Auto-save every 10 turns
    if world.turn % 10 == 0 && !world.game_over {
        if let Err(e) = state.db.lock().map(|db| {
            let _ = save::save_world(&db, world);
        }) {
            eprintln!("Failed to lock db for auto-save: {e}");
        }
    }

    // Handle game over (death or victory — victory sets game_over = true)
    if world.game_over {
        if let Err(e) = state.db.lock().map(|db| {
            let _ = save::end_run(&db, world);
        }) {
            eprintln!("Failed to lock db for end-run: {e}");
        }
    }

    Ok(result)
}

#[tauri::command]
pub fn get_game_state(state: State<'_, AppState>) -> Result<Option<GameState>, String> {
    let world_lock = state.world.lock().map_err(|e| e.to_string())?;
    match world_lock.as_ref() {
        Some(world) => {
            let result = world.build_turn_result(Vec::new());
            Ok(Some(result.state))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub fn save_game(state: State<'_, AppState>) -> Result<(), String> {
    let world_lock = state.world.lock().map_err(|e| e.to_string())?;
    let world = world_lock.as_ref().ok_or("No active game")?;

    if world.game_over {
        return Err("Cannot save a finished game".to_string());
    }

    let db = state.db.lock().map_err(|e| e.to_string())?;
    save::save_world(&db, world)
}

#[tauri::command]
pub fn load_game(state: State<'_, AppState>) -> Result<Option<TurnResult>, String> {
    // Load from db first, then release the lock before acquiring world lock
    // (consistent ordering: world before db everywhere else)
    let loaded = {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        save::load_world(&db)?
    };

    match loaded {
        Some(world) => {
            let result = world.build_turn_result(Vec::new());
            *state.world.lock().map_err(|e| e.to_string())? = Some(world);
            Ok(Some(result))
        }
        None => Ok(None),
    }
}

#[tauri::command]
pub fn inspect_entity(
    entity_id: u32,
    state: State<'_, AppState>,
) -> Result<Option<EntityDetail>, String> {
    let world_lock = state.world.lock().map_err(|e| e.to_string())?;
    let world = world_lock.as_ref().ok_or("No active game")?;

    let entity = match world.get_entity(entity_id) {
        Some(e) => e,
        None => return Ok(None),
    };

    // Only allow inspecting visible entities
    let player_fov = world
        .get_entity(world.player_id)
        .and_then(|p| p.fov.as_ref());

    let is_visible = entity.id == world.player_id
        || player_fov
            .map(|f| f.visible_tiles.contains(&entity.position))
            .unwrap_or(false);

    if !is_visible {
        return Ok(None);
    }

    let attack = entity
        .combat
        .as_ref()
        .map(|_| crate::engine::combat::effective_attack(entity));
    let defense = entity
        .combat
        .as_ref()
        .map(|_| crate::engine::combat::effective_defense(entity));

    Ok(Some(EntityDetail {
        id: entity.id,
        name: entity.name.clone(),
        entity_type: if entity.ai.is_some() {
            EntityType::Enemy
        } else if entity.id == world.player_id {
            EntityType::Player
        } else {
            EntityType::Item
        },
        hp: entity.health.as_ref().map(|h| (h.current, h.max)),
        attack,
        defense,
        status_effects: entity
            .status_effects
            .iter()
            .map(|s| StatusView {
                effect_type: s.effect_type,
                duration: s.duration,
                magnitude: s.magnitude,
            })
            .collect(),
        flavor_text: entity.flavor_text.clone(),
    }))
}

#[tauri::command]
pub fn get_run_history(state: State<'_, AppState>) -> Result<Vec<RunSummary>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    database::get_run_history(&db)
}

#[tauri::command]
pub fn get_high_scores(state: State<'_, AppState>) -> Result<Vec<HighScore>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    database::get_high_scores(&db)
}

#[tauri::command]
pub fn get_settings(state: State<'_, AppState>) -> Result<Settings, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(config::load_settings(&db))
}

#[tauri::command]
pub fn update_settings(settings: Settings, state: State<'_, AppState>) -> Result<(), String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    config::save_settings(&db, &settings)
}

#[tauri::command]
pub fn has_save_game(state: State<'_, AppState>) -> Result<bool, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(database::has_save(&db))
}

#[tauri::command]
pub fn get_adjacent_shop(state: State<'_, AppState>) -> Result<Option<ShopView>, String> {
    let world_lock = state.world.lock().map_err(|e| e.to_string())?;
    let world = world_lock.as_ref().ok_or("No active game")?;

    let player = world.get_entity(world.player_id).ok_or("No player")?;
    let player_pos = player.position;

    // Check all 8 adjacent tiles + current tile for shops
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let check_pos = Position::new(player_pos.x + dx, player_pos.y + dy);
            if let Some(shop_entity) = world
                .entities
                .iter()
                .find(|e| e.position == check_pos && e.shop.is_some())
            {
                let shop = shop_entity.shop.as_ref().unwrap();
                return Ok(Some(ShopView {
                    shop_id: shop_entity.id,
                    name: shop_entity.name.clone(),
                    items: shop
                        .items
                        .iter()
                        .map(|item| ShopItemView {
                            name: item.name.clone(),
                            price: item.price,
                            item_type: item.item_type,
                            slot: item.slot,
                        })
                        .collect(),
                }));
            }
        }
    }

    Ok(None)
}

#[tauri::command]
pub fn get_achievements(
    state: State<'_, AppState>,
) -> Result<Vec<achievements::AchievementStatus>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(achievements::get_all_statuses(&db))
}

#[tauri::command]
pub fn get_unlockables(
    state: State<'_, AppState>,
) -> Result<Vec<achievements::UnlockStatus>, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(achievements::get_all_unlock_statuses(&db))
}

#[tauri::command]
pub fn check_ollama(state: State<'_, AppState>) -> Result<OllamaStatus, String> {
    let db = state.db.lock().map_err(|e| e.to_string())?;
    let settings = config::load_settings(&db);

    if !settings.ollama_enabled {
        return Ok(OllamaStatus {
            available: false,
            model_loaded: false,
            url: settings.ollama_url,
        });
    }

    // Synchronous check with timeout
    let url = format!("{}/api/tags", settings.ollama_url);
    let timeout = std::time::Duration::from_secs(settings.ollama_timeout as u64);
    let available = reqwest::blocking::Client::builder()
        .timeout(timeout)
        .build()
        .ok()
        .and_then(|c| c.get(&url).send().ok())
        .map(|r| r.status().is_success())
        .unwrap_or(false);

    Ok(OllamaStatus {
        available,
        model_loaded: available, // simplified — real check would parse the response
        url: settings.ollama_url,
    })
}

#[tauri::command]
pub fn start_daily_challenge(state: State<'_, AppState>) -> Result<TurnResult, String> {
    let today = save::today_date_string();
    let date_key = format!("daily-{}", today);

    // Check if already played today
    {
        let db = state.db.lock().map_err(|e| e.to_string())?;
        if database::has_played_daily(&db, &today) {
            return Err("Already played today's daily challenge".to_string());
        }
    }

    // Generate deterministic seed from date string
    let mut h: u64 = 5381;
    for c in date_key.bytes() {
        h = h.wrapping_mul(33).wrapping_add(c as u64);
    }
    let seed = h;

    let mut world = World::new_with_class(seed, PlayerClass::Warrior, Vec::new());
    world.is_daily = true;
    let result = world.build_turn_result(Vec::new());

    *state.world.lock().map_err(|e| e.to_string())? = Some(world);

    Ok(result)
}

#[tauri::command]
pub fn get_daily_status(state: State<'_, AppState>) -> Result<DailyStatus, String> {
    let today = save::today_date_string();
    let db = state.db.lock().map_err(|e| e.to_string())?;
    Ok(database::get_daily_status(&db, &today))
}
