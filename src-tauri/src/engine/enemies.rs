use crate::engine::entity::*;

pub struct EnemyTemplate {
    pub name: &'static str,
    pub glyph: u32,
    pub hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub speed: i32,
    pub crit_chance: f32,
    pub ai: AIBehavior,
    pub special: Option<EnemySpecial>,
    pub min_floor: u32,
}

pub enum EnemySpecial {
    PoisonOnHit { damage: i32, duration: u32 },
    BurningOnHit { damage: i32, duration: u32 },
    SlowOnHit { magnitude: i32, duration: u32 },
    ConfuseOnHit { duration: u32 },
    LifeSteal,
    PhaseThroughWalls,
    DrainMaxHp,
    Invisible,
    HighCrit,
    SummonSkeleton { interval: u32 },
    Cleave,
    TeleportWhenAdjacent,
    AoeFrost,
    Disguised,
}

pub fn all_enemies() -> Vec<EnemyTemplate> {
    vec![
        // Floors 1-3: The Dungeon
        EnemyTemplate {
            name: "Rat",
            glyph: 0x72,
            hp: 8,
            attack: 2,
            defense: 0,
            speed: 120,
            crit_chance: 0.05,
            ai: AIBehavior::Melee,
            special: None,
            min_floor: 1,
        },
        EnemyTemplate {
            name: "Goblin",
            glyph: 0x67,
            hp: 15,
            attack: 4,
            defense: 1,
            speed: 100,
            crit_chance: 0.05,
            ai: AIBehavior::Melee,
            special: None,
            min_floor: 1,
        },
        EnemyTemplate {
            name: "Goblin Archer",
            glyph: 0x47,
            hp: 10,
            attack: 3,
            defense: 0,
            speed: 100,
            crit_chance: 0.05,
            ai: AIBehavior::Ranged {
                range: 5,
                preferred_distance: 3,
            },
            special: None,
            min_floor: 2,
        },
        EnemyTemplate {
            name: "Skeleton",
            glyph: 0x73,
            hp: 18,
            attack: 5,
            defense: 3,
            speed: 90,
            crit_chance: 0.05,
            ai: AIBehavior::Melee,
            special: None,
            min_floor: 3,
        },
        EnemyTemplate {
            name: "Giant Spider",
            glyph: 0x53,
            hp: 12,
            attack: 3,
            defense: 1,
            speed: 110,
            crit_chance: 0.05,
            ai: AIBehavior::Melee,
            special: Some(EnemySpecial::PoisonOnHit {
                damage: 2,
                duration: 3,
            }),
            min_floor: 3,
        },
        // Floors 4-6: The Caves
        EnemyTemplate {
            name: "Orc",
            glyph: 0x6F,
            hp: 30,
            attack: 7,
            defense: 3,
            speed: 90,
            crit_chance: 0.05,
            ai: AIBehavior::Melee,
            special: None,
            min_floor: 4,
        },
        EnemyTemplate {
            name: "Dark Mage",
            glyph: 0x4D,
            hp: 15,
            attack: 2,
            defense: 1,
            speed: 100,
            crit_chance: 0.05,
            ai: AIBehavior::Ranged {
                range: 6,
                preferred_distance: 4,
            },
            special: Some(EnemySpecial::ConfuseOnHit { duration: 3 }),
            min_floor: 4,
        },
        EnemyTemplate {
            name: "Cave Troll",
            glyph: 0x54,
            hp: 50,
            attack: 10,
            defense: 5,
            speed: 70,
            crit_chance: 0.05,
            ai: AIBehavior::Melee,
            special: None,
            min_floor: 5,
        },
        EnemyTemplate {
            name: "Vampire Bat",
            glyph: 0x62,
            hp: 12,
            attack: 4,
            defense: 0,
            speed: 130,
            crit_chance: 0.05,
            ai: AIBehavior::Melee,
            special: Some(EnemySpecial::LifeSteal),
            min_floor: 4,
        },
        EnemyTemplate {
            name: "Mimic",
            glyph: 0x6D,
            hp: 25,
            attack: 8,
            defense: 3,
            speed: 100,
            crit_chance: 0.05,
            ai: AIBehavior::Passive,
            special: Some(EnemySpecial::Disguised),
            min_floor: 5,
        },
        // Floors 7-9: The Deep
        EnemyTemplate {
            name: "Wraith",
            glyph: 0x57,
            hp: 20,
            attack: 8,
            defense: 2,
            speed: 110,
            crit_chance: 0.05,
            ai: AIBehavior::Melee,
            special: Some(EnemySpecial::DrainMaxHp),
            min_floor: 7,
        },
        EnemyTemplate {
            name: "Fire Elemental",
            glyph: 0x46,
            hp: 35,
            attack: 9,
            defense: 4,
            speed: 100,
            crit_chance: 0.05,
            ai: AIBehavior::Melee,
            special: Some(EnemySpecial::BurningOnHit {
                damage: 3,
                duration: 3,
            }),
            min_floor: 7,
        },
        EnemyTemplate {
            name: "Ice Golem",
            glyph: 0x49,
            hp: 60,
            attack: 7,
            defense: 8,
            speed: 60,
            crit_chance: 0.05,
            ai: AIBehavior::Melee,
            special: Some(EnemySpecial::SlowOnHit {
                magnitude: 30,
                duration: 2,
            }),
            min_floor: 7,
        },
        EnemyTemplate {
            name: "Shadow",
            glyph: 0x5A,
            hp: 15,
            attack: 12,
            defense: 1,
            speed: 120,
            crit_chance: 0.30,
            ai: AIBehavior::Melee,
            special: Some(EnemySpecial::Invisible),
            min_floor: 8,
        },
        EnemyTemplate {
            name: "Necromancer",
            glyph: 0x4E,
            hp: 25,
            attack: 3,
            defense: 2,
            speed: 90,
            crit_chance: 0.05,
            ai: AIBehavior::Ranged {
                range: 6,
                preferred_distance: 5,
            },
            special: Some(EnemySpecial::SummonSkeleton { interval: 5 }),
            min_floor: 8,
        },
    ]
}

pub fn boss_templates() -> Vec<EnemyTemplate> {
    vec![
        EnemyTemplate {
            name: "Goblin King",
            glyph: 0x4B,
            hp: 80,
            attack: 8,
            defense: 4,
            speed: 100,
            crit_chance: 0.10,
            ai: AIBehavior::Boss(BossPhase::Phase1),
            special: Some(EnemySpecial::SummonSkeleton { interval: 4 }),
            min_floor: 3,
        },
        EnemyTemplate {
            name: "Troll Warlord",
            glyph: 0x57,
            hp: 150,
            attack: 14,
            defense: 7,
            speed: 80,
            crit_chance: 0.10,
            ai: AIBehavior::Boss(BossPhase::Phase1),
            special: Some(EnemySpecial::Cleave),
            min_floor: 6,
        },
        EnemyTemplate {
            name: "The Lich",
            glyph: 0x4C,
            hp: 120,
            attack: 10,
            defense: 5,
            speed: 100,
            crit_chance: 0.15,
            ai: AIBehavior::Boss(BossPhase::Phase1),
            special: Some(EnemySpecial::TeleportWhenAdjacent),
            min_floor: 10,
        },
    ]
}

pub fn get_enemy_pool(floor: u32) -> Vec<&'static str> {
    match floor {
        1 => vec!["Rat", "Goblin"],
        2 => vec!["Rat", "Goblin", "Goblin Archer"],
        3 => vec!["Goblin", "Goblin Archer", "Skeleton", "Giant Spider"],
        4 => vec!["Orc", "Dark Mage", "Vampire Bat"],
        5 => vec!["Orc", "Dark Mage", "Cave Troll", "Vampire Bat", "Mimic"],
        6 => vec!["Orc", "Dark Mage", "Cave Troll", "Vampire Bat", "Mimic"],
        7 => vec!["Wraith", "Fire Elemental", "Ice Golem"],
        8 => vec![
            "Wraith",
            "Fire Elemental",
            "Ice Golem",
            "Shadow",
            "Necromancer",
        ],
        9 => vec![
            "Wraith",
            "Fire Elemental",
            "Ice Golem",
            "Shadow",
            "Necromancer",
        ],
        10 => vec![
            "Wraith",
            "Fire Elemental",
            "Ice Golem",
            "Shadow",
            "Necromancer",
        ],
        _ => vec![
            "Wraith",
            "Fire Elemental",
            "Ice Golem",
            "Shadow",
            "Necromancer",
            "Orc",
            "Cave Troll",
        ],
    }
}

pub fn get_boss_for_floor(floor: u32) -> Option<&'static str> {
    match floor {
        3 => Some("Goblin King"),
        6 => Some("Troll Warlord"),
        10 => Some("The Lich"),
        f if f > 10 && f % 5 == 0 => {
            // Cycle bosses for endless mode
            let boss_idx = ((f - 15) / 5) % 3;
            match boss_idx {
                0 => Some("Goblin King"),
                1 => Some("Troll Warlord"),
                _ => Some("The Lich"),
            }
        }
        _ => None,
    }
}

pub fn apply_endless_scaling(template: &EnemyTemplate, floor: u32) -> (i32, i32, i32) {
    if floor <= 10 {
        return (template.hp, template.attack, template.defense);
    }
    let multiplier = 1.0 + (floor as f64 - 10.0) * 0.15;
    (
        (template.hp as f64 * multiplier) as i32,
        (template.attack as f64 * multiplier) as i32,
        (template.defense as f64 * multiplier) as i32,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_enemies_returns_15() {
        let enemies = all_enemies();
        assert_eq!(enemies.len(), 15);
    }

    #[test]
    fn boss_templates_returns_3() {
        let bosses = boss_templates();
        assert_eq!(bosses.len(), 3);
    }

    #[test]
    fn all_enemies_have_positive_stats() {
        for e in all_enemies() {
            assert!(e.hp > 0, "{} has non-positive hp", e.name);
            assert!(e.attack > 0, "{} has non-positive attack", e.name);
            assert!(e.speed > 0, "{} has non-positive speed", e.name);
        }
    }

    #[test]
    fn bosses_have_boss_ai() {
        for b in boss_templates() {
            assert!(
                matches!(b.ai, AIBehavior::Boss(_)),
                "{} should have Boss AI",
                b.name
            );
        }
    }

    #[test]
    fn enemy_pool_per_floor() {
        for floor in 1..=10 {
            let pool = get_enemy_pool(floor);
            assert!(!pool.is_empty(), "Floor {} has empty enemy pool", floor);
        }
    }

    #[test]
    fn boss_floors() {
        assert_eq!(get_boss_for_floor(3), Some("Goblin King"));
        assert_eq!(get_boss_for_floor(6), Some("Troll Warlord"));
        assert_eq!(get_boss_for_floor(10), Some("The Lich"));
        assert_eq!(get_boss_for_floor(1), None);
        assert_eq!(get_boss_for_floor(5), None);
    }

    #[test]
    fn endless_scaling_no_change_before_11() {
        let template = &all_enemies()[0]; // Rat
        let (hp, atk, def) = apply_endless_scaling(template, 5);
        assert_eq!(hp, template.hp);
        assert_eq!(atk, template.attack);
        assert_eq!(def, template.defense);
    }

    #[test]
    fn endless_scaling_increases_after_10() {
        let template = &all_enemies()[0]; // Rat
        let (hp, atk, _def) = apply_endless_scaling(template, 20);
        assert!(hp > template.hp, "HP should scale up");
        assert!(atk > template.attack, "Attack should scale up");
    }

    #[test]
    fn unique_enemy_names() {
        let enemies = all_enemies();
        let mut names: Vec<&str> = enemies.iter().map(|e| e.name).collect();
        let len_before = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), len_before, "Enemy names should be unique");
    }
}
