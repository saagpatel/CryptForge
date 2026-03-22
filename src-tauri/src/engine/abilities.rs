use super::entity::*;

#[derive(Debug, Clone)]
pub struct Ability {
    pub id: String,
    pub name: String,
    pub mana_cost: i32,
    pub class: PlayerClass,
    pub targeting: AbilityTargeting,
    pub effect: AbilityEffect,
    pub description: String,
}

#[derive(Debug, Clone)]
pub enum AbilityTargeting {
    SelfOnly,
    Adjacent,
    Targeted { range: i32, radius: i32 },
    Direction,
}

#[derive(Debug, Clone)]
pub enum AbilityEffect {
    Damage { amount: i32 },
    StatusSelf { status: StatusType, duration: u32 },
    StatusTarget { status: StatusType, duration: u32 },
    Move { distance: i32 },
    Teleport { range: i32 },
    Shield { absorb: i32 },
    DamageAdjacent { amount: i32 },
    PoisonNextAttack { damage: i32, duration: u32 },
}

pub fn get_abilities(class: PlayerClass) -> Vec<Ability> {
    match class {
        PlayerClass::Warrior => vec![
            Ability {
                id: "shield_bash".to_string(),
                name: "Shield Bash".to_string(),
                mana_cost: 15,
                class: PlayerClass::Warrior,
                targeting: AbilityTargeting::Adjacent,
                effect: AbilityEffect::StatusTarget {
                    status: StatusType::Stunned,
                    duration: 2,
                },
                description: "Stun an adjacent enemy for 2 turns.".to_string(),
            },
            Ability {
                id: "war_cry".to_string(),
                name: "War Cry".to_string(),
                mana_cost: 20,
                class: PlayerClass::Warrior,
                targeting: AbilityTargeting::SelfOnly,
                effect: AbilityEffect::StatusSelf {
                    status: StatusType::Strengthened,
                    duration: 10,
                },
                description: "Gain Strengthened for 10 turns.".to_string(),
            },
            Ability {
                id: "whirlwind".to_string(),
                name: "Whirlwind".to_string(),
                mana_cost: 25,
                class: PlayerClass::Warrior,
                targeting: AbilityTargeting::SelfOnly,
                effect: AbilityEffect::DamageAdjacent { amount: 8 },
                description: "Deal 8 damage to all adjacent enemies.".to_string(),
            },
        ],
        PlayerClass::Rogue => vec![
            Ability {
                id: "smoke_bomb".to_string(),
                name: "Smoke Bomb".to_string(),
                mana_cost: 15,
                class: PlayerClass::Rogue,
                targeting: AbilityTargeting::SelfOnly,
                effect: AbilityEffect::StatusSelf {
                    status: StatusType::Invisible,
                    duration: 3,
                },
                description: "Become invisible for 3 turns.".to_string(),
            },
            Ability {
                id: "dash".to_string(),
                name: "Dash".to_string(),
                mana_cost: 10,
                class: PlayerClass::Rogue,
                targeting: AbilityTargeting::Direction,
                effect: AbilityEffect::Move { distance: 3 },
                description: "Move 3 tiles in a direction.".to_string(),
            },
            Ability {
                id: "poison_strike".to_string(),
                name: "Poison Strike".to_string(),
                mana_cost: 20,
                class: PlayerClass::Rogue,
                targeting: AbilityTargeting::SelfOnly,
                effect: AbilityEffect::PoisonNextAttack {
                    damage: 3,
                    duration: 5,
                },
                description: "Your next attack poisons the target.".to_string(),
            },
        ],
        PlayerClass::Mage => vec![
            Ability {
                id: "fireball".to_string(),
                name: "Fireball".to_string(),
                mana_cost: 20,
                class: PlayerClass::Mage,
                targeting: AbilityTargeting::Targeted {
                    range: 6,
                    radius: 2,
                },
                effect: AbilityEffect::Damage { amount: 12 },
                description: "Launch a fireball that explodes in a 2-tile radius.".to_string(),
            },
            Ability {
                id: "frost_bolt".to_string(),
                name: "Frost Bolt".to_string(),
                mana_cost: 15,
                class: PlayerClass::Mage,
                targeting: AbilityTargeting::Targeted {
                    range: 8,
                    radius: 0,
                },
                effect: AbilityEffect::StatusTarget {
                    status: StatusType::Slowed,
                    duration: 3,
                },
                description: "Freeze a target at range, slowing them for 3 turns.".to_string(),
            },
            Ability {
                id: "blink".to_string(),
                name: "Blink".to_string(),
                mana_cost: 25,
                class: PlayerClass::Mage,
                targeting: AbilityTargeting::Targeted {
                    range: 5,
                    radius: 0,
                },
                effect: AbilityEffect::Teleport { range: 5 },
                description: "Teleport to a visible tile within 5 range.".to_string(),
            },
            Ability {
                id: "arcane_shield".to_string(),
                name: "Arcane Shield".to_string(),
                mana_cost: 20,
                class: PlayerClass::Mage,
                targeting: AbilityTargeting::SelfOnly,
                effect: AbilityEffect::Shield { absorb: 20 },
                description: "Create a magical shield that absorbs 20 damage.".to_string(),
            },
        ],
    }
}

pub fn get_ability(class: PlayerClass, ability_id: &str) -> Option<Ability> {
    get_abilities(class)
        .into_iter()
        .find(|a| a.id == ability_id)
}

pub fn to_ability_views(class: PlayerClass) -> Vec<AbilityView> {
    get_abilities(class)
        .iter()
        .map(|a| AbilityView {
            id: a.id.clone(),
            name: a.name.clone(),
            mana_cost: a.mana_cost,
            description: a.description.clone(),
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn warrior_has_three_abilities() {
        let abilities = get_abilities(PlayerClass::Warrior);
        assert_eq!(abilities.len(), 3);
        assert_eq!(abilities[0].id, "shield_bash");
    }

    #[test]
    fn rogue_has_three_abilities() {
        let abilities = get_abilities(PlayerClass::Rogue);
        assert_eq!(abilities.len(), 3);
        assert_eq!(abilities[0].id, "smoke_bomb");
    }

    #[test]
    fn mage_has_four_abilities() {
        let abilities = get_abilities(PlayerClass::Mage);
        assert_eq!(abilities.len(), 4);
        assert_eq!(abilities[0].id, "fireball");
    }

    #[test]
    fn get_ability_by_id() {
        let a = get_ability(PlayerClass::Mage, "fireball");
        assert!(a.is_some());
        assert_eq!(a.unwrap().mana_cost, 20);

        let none = get_ability(PlayerClass::Warrior, "fireball");
        assert!(none.is_none());
    }

    #[test]
    fn ability_views() {
        let views = to_ability_views(PlayerClass::Warrior);
        assert_eq!(views.len(), 3);
        assert_eq!(views[0].name, "Shield Bash");
        assert_eq!(views[0].mana_cost, 15);
    }
}
