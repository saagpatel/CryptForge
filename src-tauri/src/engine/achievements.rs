use serde::{Deserialize, Serialize};

use super::entity::*;
use super::state::World;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementDef {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: AchievementCategory,
    pub target: u32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AchievementCategory {
    Exploration,
    Combat,
    Collection,
    Challenge,
    Misc,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AchievementStatus {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: AchievementCategory,
    pub target: u32,
    pub progress: u32,
    pub unlocked: bool,
}

pub const ACHIEVEMENTS: &[AchievementDef] = &[
    // Exploration
    AchievementDef {
        id: "reach_floor_5",
        name: "Delver",
        description: "Reach floor 5",
        category: AchievementCategory::Exploration,
        target: 5,
    },
    AchievementDef {
        id: "reach_floor_10",
        name: "Deep Diver",
        description: "Reach floor 10",
        category: AchievementCategory::Exploration,
        target: 10,
    },
    AchievementDef {
        id: "reach_floor_20",
        name: "Abyssal Explorer",
        description: "Reach floor 20 in endless mode",
        category: AchievementCategory::Exploration,
        target: 20,
    },
    // Combat
    AchievementDef {
        id: "kill_50",
        name: "Warrior",
        description: "Kill 50 enemies total",
        category: AchievementCategory::Combat,
        target: 50,
    },
    AchievementDef {
        id: "kill_100",
        name: "Veteran",
        description: "Kill 100 enemies total",
        category: AchievementCategory::Combat,
        target: 100,
    },
    AchievementDef {
        id: "kill_500",
        name: "Legend",
        description: "Kill 500 enemies total",
        category: AchievementCategory::Combat,
        target: 500,
    },
    AchievementDef {
        id: "kill_boss_1",
        name: "Boss Slayer I",
        description: "Defeat the floor 3 boss",
        category: AchievementCategory::Combat,
        target: 1,
    },
    AchievementDef {
        id: "kill_boss_2",
        name: "Boss Slayer II",
        description: "Defeat the floor 6 boss",
        category: AchievementCategory::Combat,
        target: 1,
    },
    AchievementDef {
        id: "kill_boss_3",
        name: "Boss Slayer III",
        description: "Defeat the floor 10 boss",
        category: AchievementCategory::Combat,
        target: 1,
    },
    // Collection
    AchievementDef {
        id: "collect_100_gold",
        name: "Moneybags",
        description: "Accumulate 100 gold in a single run",
        category: AchievementCategory::Collection,
        target: 100,
    },
    AchievementDef {
        id: "collect_500_gold",
        name: "Hoarder",
        description: "Accumulate 500 gold in a single run",
        category: AchievementCategory::Collection,
        target: 500,
    },
    AchievementDef {
        id: "buy_from_shop",
        name: "Customer",
        description: "Buy an item from a shop",
        category: AchievementCategory::Collection,
        target: 1,
    },
    AchievementDef {
        id: "equip_all_slots",
        name: "Fully Equipped",
        description: "Have all 6 equipment slots filled",
        category: AchievementCategory::Collection,
        target: 1,
    },
    // Challenge
    AchievementDef {
        id: "win_game",
        name: "Champion",
        description: "Defeat the final boss and win",
        category: AchievementCategory::Challenge,
        target: 1,
    },
    AchievementDef {
        id: "win_fast",
        name: "Speedrunner",
        description: "Win the game in under 500 turns",
        category: AchievementCategory::Challenge,
        target: 1,
    },
    AchievementDef {
        id: "reach_level_10",
        name: "Seasoned",
        description: "Reach player level 10",
        category: AchievementCategory::Challenge,
        target: 10,
    },
    AchievementDef {
        id: "endless_floor_15",
        name: "Endless Wanderer",
        description: "Reach floor 15 in endless mode",
        category: AchievementCategory::Challenge,
        target: 15,
    },
    // Misc
    AchievementDef {
        id: "die_floor_1",
        name: "Oops",
        description: "Die on floor 1",
        category: AchievementCategory::Misc,
        target: 1,
    },
    AchievementDef {
        id: "smash_20_barrels",
        name: "Barrel Smasher",
        description: "Smash 20 barrels total",
        category: AchievementCategory::Misc,
        target: 20,
    },
    AchievementDef {
        id: "use_10_fountains",
        name: "Fountain Drinker",
        description: "Use 10 fountains total",
        category: AchievementCategory::Misc,
        target: 10,
    },
    AchievementDef {
        id: "die_to_trap",
        name: "Watch Your Step",
        description: "Die to a trap",
        category: AchievementCategory::Misc,
        target: 1,
    },
];

/// Check all achievements against current world state and events.
/// Returns list of newly unlocked achievement names.
pub fn check_achievements(
    world: &World,
    events: &[GameEvent],
    db: &rusqlite::Connection,
) -> Vec<String> {
    let mut newly_unlocked = Vec::new();

    for def in ACHIEVEMENTS {
        // Skip already unlocked
        if is_unlocked(db, def.id) {
            continue;
        }

        let progress = compute_progress(def, world, events, db);

        // Update progress in DB
        set_progress(db, def.id, progress);

        if progress >= def.target {
            unlock(db, def.id);
            newly_unlocked.push(def.name.to_string());
        }
    }

    newly_unlocked
}

fn compute_progress(
    def: &AchievementDef,
    world: &World,
    events: &[GameEvent],
    db: &rusqlite::Connection,
) -> u32 {
    match def.id {
        // Exploration — based on current run
        "reach_floor_5" | "reach_floor_10" | "reach_floor_20" => world.floor,
        "endless_floor_15" => {
            if world.floor > 10 {
                world.floor
            } else {
                0
            }
        }

        // Combat — accumulated across runs (read from DB + current run delta)
        "kill_50" | "kill_100" | "kill_500" => {
            let db_kills = get_progress(db, def.id);
            // Add kills from current events
            let new_kills = events
                .iter()
                .filter(|e| matches!(e, GameEvent::Attacked { killed: true, .. }))
                .count() as u32;
            let total = db_kills + new_kills;
            total
        }

        "kill_boss_1" => {
            if events
                .iter()
                .any(|e| matches!(e, GameEvent::BossDefeated { floor: 3, .. }))
            {
                1
            } else {
                get_progress(db, def.id)
            }
        }
        "kill_boss_2" => {
            if events
                .iter()
                .any(|e| matches!(e, GameEvent::BossDefeated { floor: 6, .. }))
            {
                1
            } else {
                get_progress(db, def.id)
            }
        }
        "kill_boss_3" => {
            if events
                .iter()
                .any(|e| matches!(e, GameEvent::BossDefeated { floor: 10, .. }))
            {
                1
            } else {
                get_progress(db, def.id)
            }
        }

        // Collection
        "collect_100_gold" | "collect_500_gold" => world.gold,
        "buy_from_shop" => {
            if events
                .iter()
                .any(|e| matches!(e, GameEvent::ItemBought { .. }))
            {
                1
            } else {
                get_progress(db, def.id)
            }
        }
        "equip_all_slots" => {
            let filled = world
                .get_entity(world.player_id)
                .and_then(|p| p.equipment.as_ref())
                .map(|eq| {
                    [
                        eq.main_hand,
                        eq.off_hand,
                        eq.head,
                        eq.body,
                        eq.ring,
                        eq.amulet,
                    ]
                    .iter()
                    .filter(|s| s.is_some())
                    .count() as u32
                })
                .unwrap_or(0);
            if filled >= 6 {
                1
            } else {
                0
            }
        }

        // Challenge
        "win_game" => {
            if world.victory {
                1
            } else {
                get_progress(db, def.id)
            }
        }
        "win_fast" => {
            if world.victory && world.turn < 500 {
                1
            } else {
                get_progress(db, def.id)
            }
        }
        "reach_level_10" => world.player_level,

        // Misc
        "die_floor_1" => {
            if world.game_over && !world.victory && world.floor == 1 {
                1
            } else {
                get_progress(db, def.id)
            }
        }
        "smash_20_barrels" => {
            let db_count = get_progress(db, def.id);
            let new = events
                .iter()
                .filter(|e| matches!(e, GameEvent::BarrelSmashed { .. }))
                .count() as u32;
            db_count + new
        }
        "use_10_fountains" => {
            let db_count = get_progress(db, def.id);
            let new = events
                .iter()
                .filter(|e| matches!(e, GameEvent::FountainUsed { .. }))
                .count() as u32;
            db_count + new
        }
        "die_to_trap" => {
            let died_to_trap = world.game_over
                && !world.victory
                && world
                    .last_damage_source
                    .as_deref()
                    .map(|s| s.contains("trap") || s.contains("spike") || s.contains("poison"))
                    .unwrap_or(false);
            if died_to_trap {
                1
            } else {
                get_progress(db, def.id)
            }
        }

        _ => 0,
    }
}

// --- Unlockable rewards ---

pub struct UnlockReward {
    pub achievement_id: &'static str,
    pub reward_item: &'static str,
    pub description: &'static str,
}

pub const UNLOCK_REWARDS: &[UnlockReward] = &[
    UnlockReward {
        achievement_id: "win_game",
        reward_item: "Blessed Sword",
        description: "Start with a Blessed Sword",
    },
    UnlockReward {
        achievement_id: "kill_500",
        reward_item: "Veteran's Ring",
        description: "Start with Veteran's Ring (+2 ATK)",
    },
    UnlockReward {
        achievement_id: "reach_floor_20",
        reward_item: "Abyss Cloak",
        description: "Start with an Abyss Cloak (+3 DEF)",
    },
    UnlockReward {
        achievement_id: "win_fast",
        reward_item: "Speed Boots",
        description: "Start with Speed Boots (+20 SPD)",
    },
];

/// Get list of reward item names the player has unlocked.
pub fn get_unlocked_rewards(conn: &rusqlite::Connection) -> Vec<&'static str> {
    UNLOCK_REWARDS
        .iter()
        .filter(|r| is_unlocked(conn, r.achievement_id))
        .map(|r| r.reward_item)
        .collect()
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockStatus {
    pub achievement_id: String,
    pub achievement_name: String,
    pub reward_item: String,
    pub description: String,
    pub unlocked: bool,
}

pub fn get_all_unlock_statuses(conn: &rusqlite::Connection) -> Vec<UnlockStatus> {
    UNLOCK_REWARDS
        .iter()
        .map(|r| {
            let achievement_name = ACHIEVEMENTS
                .iter()
                .find(|a| a.id == r.achievement_id)
                .map(|a| a.name)
                .unwrap_or("Unknown");
            UnlockStatus {
                achievement_id: r.achievement_id.to_string(),
                achievement_name: achievement_name.to_string(),
                reward_item: r.reward_item.to_string(),
                description: r.description.to_string(),
                unlocked: is_unlocked(conn, r.achievement_id),
            }
        })
        .collect()
}

// --- DB helpers ---

pub fn ensure_table(conn: &rusqlite::Connection) {
    let _ = conn.execute_batch(
        "CREATE TABLE IF NOT EXISTS achievements (
            id TEXT PRIMARY KEY,
            progress INTEGER NOT NULL DEFAULT 0,
            unlocked INTEGER NOT NULL DEFAULT 0,
            unlocked_at TEXT
        );",
    );
}

fn is_unlocked(conn: &rusqlite::Connection, id: &str) -> bool {
    conn.query_row(
        "SELECT unlocked FROM achievements WHERE id = ?1",
        [id],
        |row| row.get::<_, i32>(0),
    )
    .unwrap_or(0)
        != 0
}

fn get_progress(conn: &rusqlite::Connection, id: &str) -> u32 {
    conn.query_row(
        "SELECT progress FROM achievements WHERE id = ?1",
        [id],
        |row| row.get::<_, u32>(0),
    )
    .unwrap_or(0)
}

fn set_progress(conn: &rusqlite::Connection, id: &str, progress: u32) {
    let _ = conn.execute(
        "INSERT INTO achievements (id, progress) VALUES (?1, ?2)
         ON CONFLICT(id) DO UPDATE SET progress = ?2 WHERE unlocked = 0",
        rusqlite::params![id, progress],
    );
}

fn unlock(conn: &rusqlite::Connection, id: &str) {
    let progress = get_progress(conn, id);
    let _ = conn.execute(
        "INSERT INTO achievements (id, progress, unlocked, unlocked_at) VALUES (?1, ?2, 1, datetime('now'))
         ON CONFLICT(id) DO UPDATE SET unlocked = 1, unlocked_at = datetime('now')",
        rusqlite::params![id, progress],
    );
}

pub fn get_all_statuses(conn: &rusqlite::Connection) -> Vec<AchievementStatus> {
    ACHIEVEMENTS
        .iter()
        .map(|def| {
            let (progress, unlocked) = conn
                .query_row(
                    "SELECT progress, unlocked FROM achievements WHERE id = ?1",
                    [def.id],
                    |row| Ok((row.get::<_, u32>(0)?, row.get::<_, i32>(1)?)),
                )
                .unwrap_or((0, 0));

            AchievementStatus {
                id: def.id.to_string(),
                name: def.name.to_string(),
                description: def.description.to_string(),
                category: def.category,
                target: def.target,
                progress,
                unlocked: unlocked != 0,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use rusqlite::Connection;

    fn test_db() -> Connection {
        let conn = Connection::open_in_memory().unwrap();
        ensure_table(&conn);
        conn
    }

    #[test]
    fn achievement_table_creation() {
        let conn = test_db();
        let statuses = get_all_statuses(&conn);
        assert_eq!(statuses.len(), ACHIEVEMENTS.len());
        assert!(statuses.iter().all(|s| !s.unlocked));
        assert!(statuses.iter().all(|s| s.progress == 0));
    }

    #[test]
    fn progress_tracking() {
        let conn = test_db();
        set_progress(&conn, "kill_50", 25);
        assert_eq!(get_progress(&conn, "kill_50"), 25);

        set_progress(&conn, "kill_50", 50);
        assert_eq!(get_progress(&conn, "kill_50"), 50);
    }

    #[test]
    fn unlock_achievement() {
        let conn = test_db();
        assert!(!is_unlocked(&conn, "win_game"));
        unlock(&conn, "win_game");
        assert!(is_unlocked(&conn, "win_game"));
    }

    #[test]
    fn progress_frozen_after_unlock() {
        let conn = test_db();
        set_progress(&conn, "kill_50", 50);
        unlock(&conn, "kill_50");

        // Trying to set progress should not change it (due to WHERE unlocked = 0)
        set_progress(&conn, "kill_50", 0);
        assert_eq!(get_progress(&conn, "kill_50"), 50);
    }

    #[test]
    fn check_achievements_unlocks_floor_5() {
        let conn = test_db();
        let mut world = World::new(42);
        world.floor = 5;

        let unlocked = check_achievements(&world, &[], &conn);
        assert!(unlocked.contains(&"Delver".to_string()));
    }

    #[test]
    fn check_achievements_no_double_unlock() {
        let conn = test_db();
        let mut world = World::new(42);
        world.floor = 5;

        let unlocked1 = check_achievements(&world, &[], &conn);
        assert!(!unlocked1.is_empty());

        let unlocked2 = check_achievements(&world, &[], &conn);
        assert!(
            unlocked2.is_empty(),
            "Already unlocked achievements should not unlock again"
        );
    }
}
