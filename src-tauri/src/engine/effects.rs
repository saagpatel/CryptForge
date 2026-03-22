use super::entity::*;

/// Apply a status effect to an entity, respecting stacking rules.
/// Same-type effects refresh duration rather than stacking.
pub fn apply_status(
    entity: &mut Entity,
    effect_type: StatusType,
    duration: u32,
    magnitude: i32,
    source: &str,
) {
    // Boss immunities
    if is_boss(entity)
        && (effect_type == StatusType::Stunned || effect_type == StatusType::Confused)
    {
        return;
    }

    // Refresh if same type exists
    if let Some(existing) = entity
        .status_effects
        .iter_mut()
        .find(|s| s.effect_type == effect_type)
    {
        existing.duration = existing.duration.max(duration);
        existing.magnitude = existing.magnitude.max(magnitude);
        return;
    }

    entity.status_effects.push(StatusEffect {
        effect_type,
        duration,
        magnitude,
        source: source.to_string(),
    });
}

/// Remove all negative status effects (used by Antidote/CureStatus).
pub fn cure_negative_effects(entity: &mut Entity) {
    entity
        .status_effects
        .retain(|s| !s.effect_type.is_negative());
}

/// Tick all effects on an entity. Returns (damage_taken, healing_done, expired_effects).
pub fn tick_effects(entity: &mut Entity) -> EffectTickResult {
    let mut result = EffectTickResult::default();

    for effect in &entity.status_effects {
        match effect.effect_type {
            StatusType::Poison => {
                let damage = effect.magnitude.max(2);
                result.damage += damage;
            }
            StatusType::Burning => {
                let damage = effect.magnitude.max(3);
                result.damage += damage;
            }
            StatusType::Regenerating => {
                let heal = effect.magnitude.max(2);
                result.healing += heal;
            }
            _ => {}
        }
    }

    // Apply damage/healing
    if let Some(ref mut health) = entity.health {
        health.current -= result.damage;
        health.current = (health.current + result.healing).min(health.max);
    }

    // Decrement durations
    for effect in &mut entity.status_effects {
        if effect.duration > 0 {
            effect.duration -= 1;
            if effect.duration == 0 {
                result.expired.push(effect.effect_type);
            }
        }
    }
    entity.status_effects.retain(|e| e.duration > 0);

    result
}

/// Check if entity is stunned (should skip turn).
pub fn is_stunned(entity: &Entity) -> bool {
    entity
        .status_effects
        .iter()
        .any(|s| s.effect_type == StatusType::Stunned)
}

/// Check if entity is invisible.
pub fn is_invisible(entity: &Entity) -> bool {
    entity
        .status_effects
        .iter()
        .any(|s| s.effect_type == StatusType::Invisible)
}

/// Check if entity is confused.
pub fn is_confused(entity: &Entity) -> bool {
    entity
        .status_effects
        .iter()
        .any(|s| s.effect_type == StatusType::Confused)
}

/// Get the effective FOV radius considering Blinded status.
pub fn effective_fov_radius(entity: &Entity) -> i32 {
    let base = entity.fov.as_ref().map(|f| f.radius).unwrap_or(8);
    if entity
        .status_effects
        .iter()
        .any(|s| s.effect_type == StatusType::Blinded)
    {
        2
    } else {
        base
    }
}

/// Get shield HP buffer if Shielded.
pub fn shield_buffer(entity: &Entity) -> i32 {
    entity
        .status_effects
        .iter()
        .find(|s| s.effect_type == StatusType::Shielded)
        .map(|s| s.magnitude)
        .unwrap_or(0)
}

/// Absorb damage through shield. Returns remaining damage after shield absorption.
pub fn absorb_shield_damage(entity: &mut Entity, damage: i32) -> i32 {
    if let Some(shield) = entity
        .status_effects
        .iter_mut()
        .find(|s| s.effect_type == StatusType::Shielded)
    {
        if shield.magnitude >= damage {
            shield.magnitude -= damage;
            return 0;
        } else {
            let remaining = damage - shield.magnitude;
            shield.magnitude = 0;
            shield.duration = 0;
            // Remove the depleted shield immediately
            entity
                .status_effects
                .retain(|s| !(s.effect_type == StatusType::Shielded && s.magnitude == 0));
            return remaining;
        }
    }
    damage
}

fn is_boss(entity: &Entity) -> bool {
    matches!(entity.ai, Some(AIBehavior::Boss(_)))
}

#[derive(Default)]
pub struct EffectTickResult {
    pub damage: i32,
    pub healing: i32,
    pub expired: Vec<StatusType>,
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_entity(hp: i32) -> Entity {
        Entity {
            id: 1,
            name: "Test".to_string(),
            position: Position::new(5, 5),
            glyph: 0x67,
            render_order: RenderOrder::Enemy,
            blocks_movement: true,
            blocks_fov: false,
            health: Some(Health::new(hp)),
            combat: Some(CombatStats {
                base_attack: 5,
                base_defense: 2,
                base_speed: 100,
                crit_chance: 0.05,
                dodge_chance: 0.0,
                ranged: None,
                on_hit: None,
            }),
            ai: Some(AIBehavior::Melee),
            inventory: None,
            equipment: None,
            item: None,
            status_effects: Vec::new(),
            fov: Some(FieldOfView::new(8)),
            door: None,
            trap: None,
            stair: None,
            loot_table: None,
            flavor_text: None,
            shop: None,
            interactive: None,
            elite: None,
            resurrection_timer: None,
        }
    }

    #[test]
    fn poison_deals_damage() {
        let mut entity = make_entity(50);
        apply_status(&mut entity, StatusType::Poison, 3, 5, "test");

        let result = tick_effects(&mut entity);
        assert_eq!(result.damage, 5);
        assert_eq!(entity.health.as_ref().unwrap().current, 45);
    }

    #[test]
    fn regeneration_heals() {
        let mut entity = make_entity(50);
        entity.health.as_mut().unwrap().current = 30;
        apply_status(&mut entity, StatusType::Regenerating, 3, 5, "ring");

        let result = tick_effects(&mut entity);
        assert_eq!(result.healing, 5);
        assert_eq!(entity.health.as_ref().unwrap().current, 35);
    }

    #[test]
    fn effects_expire() {
        let mut entity = make_entity(50);
        apply_status(&mut entity, StatusType::Poison, 1, 2, "test");
        assert_eq!(entity.status_effects.len(), 1);

        let result = tick_effects(&mut entity);
        assert_eq!(result.expired.len(), 1);
        assert_eq!(entity.status_effects.len(), 0);
    }

    #[test]
    fn same_type_refreshes_duration() {
        let mut entity = make_entity(50);
        apply_status(&mut entity, StatusType::Poison, 3, 2, "spider");
        apply_status(&mut entity, StatusType::Poison, 5, 2, "trap");

        assert_eq!(entity.status_effects.len(), 1);
        assert_eq!(entity.status_effects[0].duration, 5);
    }

    #[test]
    fn boss_immune_to_stun() {
        let mut entity = make_entity(100);
        entity.ai = Some(AIBehavior::Boss(BossPhase::Phase1));

        apply_status(&mut entity, StatusType::Stunned, 3, 0, "test");
        assert!(entity.status_effects.is_empty());
    }

    #[test]
    fn boss_immune_to_confusion() {
        let mut entity = make_entity(100);
        entity.ai = Some(AIBehavior::Boss(BossPhase::Phase1));

        apply_status(&mut entity, StatusType::Confused, 3, 0, "test");
        assert!(entity.status_effects.is_empty());
    }

    #[test]
    fn shield_absorbs_damage() {
        let mut entity = make_entity(50);
        apply_status(&mut entity, StatusType::Shielded, 999, 10, "spell");

        let remaining = absorb_shield_damage(&mut entity, 7);
        assert_eq!(remaining, 0);
        assert_eq!(entity.status_effects[0].magnitude, 3);
    }

    #[test]
    fn shield_breaks_on_excess_damage() {
        let mut entity = make_entity(50);
        apply_status(&mut entity, StatusType::Shielded, 999, 5, "spell");

        let remaining = absorb_shield_damage(&mut entity, 8);
        assert_eq!(remaining, 3);
        assert!(entity.status_effects.is_empty());
    }

    #[test]
    fn cure_removes_negative_only() {
        let mut entity = make_entity(50);
        apply_status(&mut entity, StatusType::Poison, 3, 2, "spider");
        apply_status(&mut entity, StatusType::Hasted, 5, 0, "potion");

        cure_negative_effects(&mut entity);
        assert_eq!(entity.status_effects.len(), 1);
        assert_eq!(entity.status_effects[0].effect_type, StatusType::Hasted);
    }

    #[test]
    fn blinded_reduces_fov() {
        let mut entity = make_entity(50);
        assert_eq!(effective_fov_radius(&entity), 8);

        apply_status(&mut entity, StatusType::Blinded, 3, 0, "dark mage");
        assert_eq!(effective_fov_radius(&entity), 2);
    }
}
