use std::collections::{HashMap, HashSet};

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use serde::{Deserialize, Serialize};

use super::ai;
use super::combat;
use super::dungeon;
use super::dungeon::placement;
use super::enemies;
use super::entity::*;
use super::fov;
use super::map::{Map, TileType};
use super::pathfinding::{self, has_line_of_sight, DijkstraMap};

const ENERGY_THRESHOLD: i32 = 100;

#[derive(Serialize, Deserialize)]
pub struct World {
    pub seed: u64,
    pub floor: u32,
    pub turn: u32,
    pub map: Map,
    pub entities: Vec<Entity>,
    pub player_id: EntityId,
    pub energy: HashMap<EntityId, i32>,
    pub dijkstra: Option<DijkstraMap>,
    pub messages: Vec<LogMessage>,
    pub pending_level_up: bool,
    pub player_level: u32,
    pub player_xp: u32,
    pub enemies_killed: u32,
    pub bosses_killed: u32,
    pub game_over: bool,
    pub victory: bool,
    pub gold: u32,
    pub last_damage_source: Option<String>,
    pub spotted_enemies: HashSet<EntityId>,
    #[serde(with = "rng_serde")]
    pub rng: StdRng,
    pub player_class: PlayerClass,
    pub mana: i32,
    pub max_mana: i32,
    pub hunger: i32,
    pub max_hunger: i32,
    pub modifiers: Vec<RunModifier>,
    #[serde(default)]
    pub is_daily: bool,
    pub cleave_bonus: i32,
    pub spell_power_bonus: i32,
    pub mana_regen: i32,
    /// Tracks per-boss action counters for timed abilities (e.g., summon intervals).
    pub boss_action_counter: HashMap<EntityId, u32>,
}

mod rng_serde {
    use rand::rngs::StdRng;
    use rand::SeedableRng;
    use serde::{self, Deserialize, Deserializer, Serializer};

    pub fn serialize<S>(_rng: &StdRng, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        // We can't easily get the internal state, so we store a placeholder.
        // On load, the RNG will be re-seeded from the seed + turn.
        serializer.serialize_u64(0)
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<StdRng, D::Error>
    where
        D: Deserializer<'de>,
    {
        // Placeholder - will be re-seeded on load
        let _: u64 = Deserialize::deserialize(deserializer)?;
        Ok(StdRng::seed_from_u64(0))
    }
}

impl World {
    pub fn new(seed: u64) -> Self {
        Self::new_with_class(seed, PlayerClass::Warrior, Vec::new())
    }

    pub fn new_with_class(seed: u64, class: PlayerClass, modifiers: Vec<RunModifier>) -> Self {
        let floor = 1;
        let mut rng = StdRng::seed_from_u64(seed);
        let map = dungeon::generate_floor(seed, floor);

        let template = super::classes::get_class_template(class);

        // Find start room position for player
        let start_pos = map
            .rooms
            .iter()
            .find(|r| r.room_type == super::map::RoomType::Start)
            .map(|r| r.center())
            .unwrap_or(Position::new(1, 1));

        let player = placement::spawn_player_with_class(start_pos, &template);
        let mut entities = vec![player];

        // Spawn floor entities
        let floor_entities = placement::spawn_entities(&map, floor, &mut rng);
        entities.extend(floor_entities);

        // Place stairs entities
        place_stairs(&map, &mut entities);

        // Cursed modifier: mark floor items as unidentified
        if modifiers.contains(&RunModifier::Cursed) {
            for entity in &mut entities {
                if let Some(ref mut item) = entity.item {
                    if item.item_type.is_consumable() {
                        item.identified = false;
                    }
                }
            }
        }

        // Initialize energy map
        let mut energy = HashMap::new();
        for entity in &entities {
            if entity.combat.is_some() {
                energy.insert(entity.id, 0);
            }
        }

        let mut world = World {
            seed,
            floor,
            turn: 0,
            map,
            entities,
            player_id: 0,
            energy,
            dijkstra: None,
            messages: Vec::new(),
            pending_level_up: false,
            player_level: 1,
            player_xp: 0,
            enemies_killed: 0,
            bosses_killed: 0,
            game_over: false,
            victory: false,
            gold: 0,
            last_damage_source: None,
            spotted_enemies: HashSet::new(),
            rng,
            player_class: class,
            mana: template.mana,
            max_mana: template.max_mana,
            hunger: 1000,
            max_hunger: 1000,
            modifiers,
            is_daily: false,
            cleave_bonus: 0,
            spell_power_bonus: 0,
            mana_regen: 0,
            boss_action_counter: HashMap::new(),
        };

        // Initial FOV computation
        world.recompute_fov();
        world.recompute_dijkstra();

        world.push_message(
            "Welcome to CryptForge! Press ? for help.",
            LogSeverity::Info,
        );

        world
    }

    /// Add unlocked achievement reward items to the player's starting inventory.
    pub fn add_unlocked_rewards(&mut self, reward_names: Vec<&str>) {
        use super::dungeon::placement::next_id;
        use super::items;

        let player_id = self.player_id;
        for name in reward_names {
            if let Some(t) = items::find_template(name) {
                let item_entity = Entity {
                    id: next_id(),
                    name: t.name.to_string(),
                    position: Position::new(0, 0),
                    glyph: t.glyph,
                    render_order: RenderOrder::Item,
                    blocks_movement: false,
                    blocks_fov: false,
                    health: None,
                    combat: None,
                    ai: None,
                    inventory: None,
                    equipment: None,
                    item: Some(ItemProperties {
                        item_type: t.item_type,
                        slot: t.slot,
                        power: t.power,
                        speed_mod: t.speed_mod,
                        effect: t.effect.clone(),
                        charges: t.charges,
                        energy_cost: t.energy_cost,
                        ammo_type: t.ammo_type,
                        ranged: t.ranged,
                        hunger_restore: t.hunger_restore,
                        enchant_level: 0,
                        identified: true,
                    }),
                    status_effects: Vec::new(),
                    fov: None,
                    door: None,
                    trap: None,
                    stair: None,
                    loot_table: None,
                    flavor_text: None,
                    shop: None,
                    interactive: None,
                    elite: None,
                    resurrection_timer: None,
                };
                if let Some(player) = self.get_entity_mut(player_id) {
                    let _ = super::inventory::add_to_inventory(player, item_entity);
                }
            }
        }
    }

    /// Main turn resolution. Called when the player takes an action.
    pub fn resolve_turn(&mut self, action: PlayerAction) -> TurnResult {
        let mut events = Vec::new();

        if self.game_over {
            return self.build_turn_result(events);
        }

        // Level-up choice is a free action — apply and return without advancing turn
        if self.pending_level_up {
            if let PlayerActionType::LevelUpChoice(choice) = &action.action_type {
                let lu_events = self.apply_level_up(*choice);
                events.extend(lu_events);
            }
            return self.build_turn_result(events);
        }

        // 1. Resolve player action
        let player_events = self.resolve_player_action(&action);
        events.extend(player_events);

        // Check if player died from their own action (trap, etc.)
        if self.is_player_dead() {
            return self.handle_player_death(events);
        }

        // 2. Grant energy and process enemy turns
        self.turn += 1;
        let enemy_events = self.process_enemy_turns();
        events.extend(enemy_events);

        // Check if player died from enemy attacks
        if self.is_player_dead() {
            return self.handle_player_death(events);
        }

        // 3. Tick status effects
        let effect_events = self.tick_status_effects();
        events.extend(effect_events);

        // Check if player died from status effects
        if self.is_player_dead() {
            return self.handle_player_death(events);
        }

        // 3b. Hunger tick
        let hunger_events = self.tick_hunger();
        events.extend(hunger_events);

        if self.is_player_dead() {
            return self.handle_player_death(events);
        }

        // 3c. Mana regen
        let regen = 1 + self.mana_regen;
        if self.mana < self.max_mana {
            self.mana = (self.mana + regen).min(self.max_mana);
        }

        // 3d. Floor-specific effects (resurrection, fire spread, etc.)
        let floor_events = self.tick_floor_effects();
        events.extend(floor_events);

        if self.is_player_dead() {
            return self.handle_player_death(events);
        }

        // 4. Recompute FOV and Dijkstra
        self.recompute_fov();
        self.recompute_dijkstra();

        // 5. Check for newly spotted enemies
        let spot_events = self.check_spotted_enemies();
        events.extend(spot_events);

        let mut result = self.build_turn_result(events);

        // Check auto-explore interrupts
        if matches!(&action.action_type, PlayerActionType::AutoExplore) {
            result.auto_explore_interrupt = self.check_auto_explore_interrupt(&result.events);
        }

        result
    }

    fn resolve_player_action(&mut self, action: &PlayerAction) -> Vec<GameEvent> {
        let mut events = Vec::new();

        match &action.action_type {
            PlayerActionType::Move(dir) => {
                let player_pos = self.get_entity(self.player_id).unwrap().position;
                let new_pos = player_pos.apply_direction(*dir);

                // Check for bump-to-attack
                if let Some(target_id) = self.hostile_entity_at(new_pos) {
                    let attack_events = self.perform_attack(self.player_id, target_id);
                    events.extend(attack_events);
                } else if let Some(shop_id) = self.shop_entity_at(new_pos) {
                    // Bump into shopkeeper opens shop
                    let shop_name = self
                        .get_entity(shop_id)
                        .map(|e| e.name.clone())
                        .unwrap_or_default();
                    self.push_message(
                        &format!("{} welcomes you to their shop.", shop_name),
                        LogSeverity::Info,
                    );
                } else if self.can_move_to(new_pos) {
                    // Check for doors
                    if let Some(door_id) = self.door_at(new_pos) {
                        let door_events = self.try_open_door(door_id);
                        events.extend(door_events);
                    } else {
                        let from = player_pos;
                        self.move_entity(self.player_id, new_pos);
                        events.push(GameEvent::Moved {
                            entity_id: self.player_id,
                            from,
                            to: new_pos,
                        });

                        // Check for traps
                        let trap_events = self.check_traps(self.player_id, new_pos);
                        events.extend(trap_events);
                    }
                } else if self.map.in_bounds(new_pos.x, new_pos.y)
                    && self.map.get_tile(new_pos.x, new_pos.y) == TileType::SecretWall
                {
                    // Bump-to-reveal secret wall
                    self.map.set_tile(new_pos.x, new_pos.y, TileType::Floor);
                    self.map.refresh_blocked();
                    self.push_message("You discover a secret passage!", LogSeverity::Good);
                    events.push(GameEvent::SecretRoomFound { position: new_pos });
                    self.recompute_fov();
                } else {
                    self.push_message("You can't move there.", LogSeverity::Info);
                }
            }

            PlayerActionType::Wait => {
                self.push_message("You wait.", LogSeverity::Info);
            }

            PlayerActionType::PickUp => {
                events.extend(self.try_pickup());
            }

            PlayerActionType::UseStairs => {
                events.extend(self.try_use_stairs());
            }

            PlayerActionType::UseItem(item_idx) => {
                events.extend(self.try_use_item(*item_idx));
            }

            PlayerActionType::DropItem(item_idx) => {
                events.extend(self.try_drop_item(*item_idx));
            }

            PlayerActionType::EquipItem(item_idx) => {
                events.extend(self.try_equip_item(*item_idx));
            }

            PlayerActionType::UnequipSlot(slot) => {
                events.extend(self.try_unequip_slot(*slot));
            }

            PlayerActionType::LevelUpChoice(choice) => {
                events.extend(self.apply_level_up(*choice));
            }

            PlayerActionType::AutoExplore => {
                events.extend(self.auto_explore());
            }

            PlayerActionType::RangedAttack { target_id } => {
                events.extend(self.try_ranged_attack(*target_id));
            }

            PlayerActionType::BuyItem { shop_id, index } => {
                events.extend(self.try_buy_item(*shop_id, *index));
            }

            PlayerActionType::SellItem { index, shop_id } => {
                events.extend(self.try_sell_item(*index, *shop_id));
            }

            PlayerActionType::Interact => {
                events.extend(self.try_interact());
            }

            PlayerActionType::ClickMove { x, y } => {
                let target = Position::new(*x, *y);
                let player_pos = self.get_entity(self.player_id).unwrap().position;

                if target == player_pos {
                    // Clicking on self = pick up
                    events.extend(self.try_pickup());
                } else if let Some(target_id) = self.hostile_entity_at(target) {
                    // Click on hostile entity — if adjacent, bump-attack; otherwise pathfind toward
                    if player_pos.chebyshev_distance(&target) <= 1 {
                        let attack_events = self.perform_attack(self.player_id, target_id);
                        events.extend(attack_events);
                    } else if let Some(path) = pathfinding::astar(&self.map, player_pos, target) {
                        if let Some(first_step) = path.first() {
                            // Check if there's a hostile at the first step (bump-attack)
                            if let Some(blocking_id) = self.hostile_entity_at(*first_step) {
                                events.extend(self.perform_attack(self.player_id, blocking_id));
                            } else if self.can_move_to(*first_step) {
                                let from = player_pos;
                                // Check for doors
                                if let Some(door_id) = self.door_at(*first_step) {
                                    events.extend(self.try_open_door(door_id));
                                } else {
                                    self.move_entity(self.player_id, *first_step);
                                    events.push(GameEvent::Moved {
                                        entity_id: self.player_id,
                                        from,
                                        to: *first_step,
                                    });
                                    let trap_events = self.check_traps(self.player_id, *first_step);
                                    events.extend(trap_events);
                                }
                            }
                        }
                    } else {
                        self.push_message("No path to that location.", LogSeverity::Info);
                    }
                } else {
                    // Click on empty tile — pathfind and take first step
                    if let Some(path) = pathfinding::astar(&self.map, player_pos, target) {
                        if let Some(first_step) = path.first() {
                            if let Some(blocking_id) = self.hostile_entity_at(*first_step) {
                                events.extend(self.perform_attack(self.player_id, blocking_id));
                            } else if self.can_move_to(*first_step) {
                                let from = player_pos;
                                if let Some(door_id) = self.door_at(*first_step) {
                                    events.extend(self.try_open_door(door_id));
                                } else {
                                    self.move_entity(self.player_id, *first_step);
                                    events.push(GameEvent::Moved {
                                        entity_id: self.player_id,
                                        from,
                                        to: *first_step,
                                    });
                                    let trap_events = self.check_traps(self.player_id, *first_step);
                                    events.extend(trap_events);
                                }
                            }
                        }
                    } else {
                        self.push_message("No path to that location.", LogSeverity::Info);
                    }
                }
            }

            PlayerActionType::UseAbility { ability_id, target } => {
                let ability = super::abilities::get_ability(self.player_class, ability_id);
                if let Some(ab) = ability {
                    if self.mana >= ab.mana_cost {
                        self.mana -= ab.mana_cost;
                        events.push(GameEvent::ManaChanged {
                            amount: -ab.mana_cost,
                        });
                        self.push_message(&format!("You cast {}!", ab.name), LogSeverity::Good);
                        let pos = target.unwrap_or_else(|| {
                            self.get_entity(self.player_id)
                                .map(|p| p.position)
                                .unwrap_or(Position::new(0, 0))
                        });
                        events.push(GameEvent::AbilityUsed {
                            name: ab.name.clone(),
                            position: pos,
                            targets: vec![pos],
                        });
                    } else {
                        self.push_message("Not enough mana!", LogSeverity::Warning);
                    }
                } else {
                    self.push_message("Unknown ability.", LogSeverity::Warning);
                }
            }

            PlayerActionType::Craft {
                weapon_idx,
                scroll_idx,
            } => {
                // Check if player is adjacent to an anvil
                let player_pos = self.get_entity(self.player_id).unwrap().position;
                let has_anvil = self.entities.iter().any(|e| {
                    e.interactive
                        .as_ref()
                        .map_or(false, |i| i.interaction_type == InteractionType::Anvil)
                        && e.position.chebyshev_distance(&player_pos) <= 1
                });

                if !has_anvil {
                    self.push_message(
                        "You need to be next to an anvil to craft.",
                        LogSeverity::Warning,
                    );
                } else {
                    // Simple enchanting: increment weapon power, consume scroll, deduct gold
                    let weapon_idx = *weapon_idx as usize;
                    let scroll_idx = *scroll_idx as usize;

                    let (weapon_name, enchant_level) = self
                        .get_entity(self.player_id)
                        .and_then(|p| p.inventory.as_ref())
                        .and_then(|inv| inv.items.get(weapon_idx))
                        .map(|i| {
                            (
                                i.name.clone(),
                                i.item.as_ref().map_or(0, |p| p.enchant_level),
                            )
                        })
                        .unwrap_or(("".to_string(), 0));

                    if enchant_level >= 3 {
                        self.push_message(
                            "This weapon is already at maximum enchantment (+3).",
                            LogSeverity::Warning,
                        );
                    } else {
                        let cost = (10 * (enchant_level + 1)) as u32;
                        if self.gold < cost {
                            self.push_message(
                                &format!("You need {} gold to enchant.", cost),
                                LogSeverity::Warning,
                            );
                        } else {
                            self.gold -= cost;

                            // Remove scroll from inventory
                            if let Some(player) = self.get_entity_mut(self.player_id) {
                                if let Some(ref mut inv) = player.inventory {
                                    if scroll_idx < inv.items.len() {
                                        inv.items.remove(scroll_idx);
                                    }
                                }
                            }

                            // Enhance weapon
                            let new_level = enchant_level + 1;
                            if let Some(player) = self.get_entity_mut(self.player_id) {
                                if let Some(ref mut inv) = player.inventory {
                                    if let Some(item) = inv.items.get_mut(weapon_idx) {
                                        if let Some(ref mut props) = item.item {
                                            props.power += 1;
                                            props.enchant_level = new_level;
                                        }
                                        // Append +N to name
                                        item.name = format!(
                                            "{} +{}",
                                            weapon_name
                                                .trim_end_matches(&format!(" +{}", enchant_level)),
                                            new_level
                                        );
                                    }
                                }
                            }

                            self.push_message(
                                &format!("The anvil glows! {} is now +{}!", weapon_name, new_level),
                                LogSeverity::Good,
                            );
                            events.push(GameEvent::ItemEnchanted {
                                item_name: weapon_name,
                                new_level,
                            });
                        }
                    }
                }
            }
        }

        events
    }

    fn perform_attack(&mut self, attacker_id: EntityId, target_id: EntityId) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let attacker = self.get_entity(attacker_id).unwrap().clone();
        let target = self.get_entity(target_id).unwrap().clone();

        // Check dodge before resolving attack
        let dodge_chance = target.combat.as_ref().map_or(0.0, |c| c.dodge_chance);
        if dodge_chance > 0.0 && self.rng.gen::<f32>() < dodge_chance {
            let target_name = target.name.clone();
            let attacker_name = attacker.name.clone();
            self.push_message(
                &format!("{} dodges {}'s attack!", target_name, attacker_name),
                LogSeverity::Good,
            );
            events.push(GameEvent::Attacked {
                attacker_id,
                target_id,
                damage: 0,
                killed: false,
                damage_type: "physical".to_string(),
                dodged: true,
            });
            return events;
        }

        let mut result = combat::resolve_attack(&attacker, &target, &mut self.rng);

        // Cleave bonus: flat bonus damage for player melee attacks
        if attacker_id == self.player_id && self.cleave_bonus > 0 {
            result.damage += self.cleave_bonus;
        }

        // GlassCannon modifier: 2x all damage
        if self.modifiers.contains(&RunModifier::GlassCannon) {
            result.damage *= 2;
        }

        // Determine damage_type from attacker's on_hit effect
        let damage_type = attacker
            .combat
            .as_ref()
            .and_then(|c| c.on_hit.as_ref())
            .map(|oh| match oh {
                OnHitEffect::Burn { .. } => "fire",
                OnHitEffect::Poison { .. } => "poison",
                OnHitEffect::Slow { .. } => "ice",
                _ => "physical",
            })
            .unwrap_or("physical")
            .to_string();

        // Apply damage
        if let Some(target_entity) = self.get_entity_mut(target_id) {
            if let Some(ref mut health) = target_entity.health {
                health.current -= result.damage;
            }
        }

        let target_name = target.name.clone();
        let attacker_name = attacker.name.clone();

        // Track what damages the player for death screen
        if target_id == self.player_id {
            self.last_damage_source = Some(format!("Slain by {}", attacker_name));
        }

        if result.is_crit {
            self.push_message(
                &format!(
                    "{} critically hits {} for {} damage!",
                    attacker_name, target_name, result.damage
                ),
                LogSeverity::Danger,
            );
        } else {
            self.push_message(
                &format!(
                    "{} hits {} for {} damage.",
                    attacker_name, target_name, result.damage
                ),
                LogSeverity::Info,
            );
        }

        events.push(GameEvent::Attacked {
            attacker_id,
            target_id,
            damage: result.damage,
            killed: result.killed,
            damage_type,
            dodged: false,
        });

        if result.killed {
            events.extend(self.handle_entity_death(target_id));
        } else {
            // Apply on-hit effects from attacker's combat stats
            let on_hit = attacker.combat.as_ref().and_then(|c| c.on_hit.clone());
            if let Some(effect) = on_hit {
                self.apply_on_hit_effect(
                    &effect,
                    attacker_id,
                    target_id,
                    result.damage,
                    &attacker_name,
                    &target_name,
                );
            }

            // Activate passive enemies when they take damage
            if let Some(target_entity) = self.get_entity_mut(target_id) {
                ai::activate_passive(target_entity);
                if ai::check_boss_phase(target_entity) {
                    let boss_name = target_entity.name.clone();
                    self.push_message(
                        &format!("{} enters a frenzied state!", boss_name),
                        LogSeverity::Danger,
                    );
                }
            }
        }

        events
    }

    fn apply_on_hit_effect(
        &mut self,
        effect: &OnHitEffect,
        attacker_id: EntityId,
        target_id: EntityId,
        damage: i32,
        attacker_name: &str,
        target_name: &str,
    ) {
        match effect {
            OnHitEffect::Poison {
                damage: dmg,
                duration,
            } => {
                if let Some(target) = self.get_entity_mut(target_id) {
                    target.status_effects.push(StatusEffect {
                        effect_type: StatusType::Poison,
                        duration: *duration,
                        magnitude: *dmg,
                        source: attacker_name.to_string(),
                    });
                }
                self.push_message(
                    &format!("{} poisons {}!", attacker_name, target_name),
                    LogSeverity::Danger,
                );
            }
            OnHitEffect::Burn {
                damage: dmg,
                duration,
            } => {
                if let Some(target) = self.get_entity_mut(target_id) {
                    target.status_effects.push(StatusEffect {
                        effect_type: StatusType::Burning,
                        duration: *duration,
                        magnitude: *dmg,
                        source: attacker_name.to_string(),
                    });
                }
                self.push_message(
                    &format!("{} sets {} ablaze!", attacker_name, target_name),
                    LogSeverity::Danger,
                );
            }
            OnHitEffect::Slow {
                magnitude: _,
                duration,
            } => {
                if let Some(target) = self.get_entity_mut(target_id) {
                    target.status_effects.push(StatusEffect {
                        effect_type: StatusType::Slowed,
                        duration: *duration,
                        magnitude: 0,
                        source: attacker_name.to_string(),
                    });
                }
                self.push_message(
                    &format!("{} slows {}!", attacker_name, target_name),
                    LogSeverity::Info,
                );
            }
            OnHitEffect::Confuse { duration } => {
                if let Some(target) = self.get_entity_mut(target_id) {
                    target.status_effects.push(StatusEffect {
                        effect_type: StatusType::Confused,
                        duration: *duration,
                        magnitude: 0,
                        source: attacker_name.to_string(),
                    });
                }
                self.push_message(
                    &format!("{} confuses {}!", attacker_name, target_name),
                    LogSeverity::Danger,
                );
            }
            OnHitEffect::LifeSteal => {
                let heal = damage / 2;
                if heal > 0 {
                    if let Some(attacker_entity) = self.get_entity_mut(attacker_id) {
                        if let Some(ref mut health) = attacker_entity.health {
                            health.current = (health.current + heal).min(health.max);
                        }
                    }
                    self.push_message(
                        &format!("{} drains life from {}!", attacker_name, target_name),
                        LogSeverity::Danger,
                    );
                }
            }
            OnHitEffect::DrainMaxHp => {
                if let Some(target) = self.get_entity_mut(target_id) {
                    if let Some(ref mut health) = target.health {
                        let drain = 2;
                        health.max = (health.max - drain).max(1);
                        health.current = health.current.min(health.max);
                    }
                }
                self.push_message(
                    &format!("{} drains {}'s vitality!", attacker_name, target_name),
                    LogSeverity::Danger,
                );
            }
        }
    }

    fn handle_entity_death(&mut self, entity_id: EntityId) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let entity = self.get_entity(entity_id).unwrap().clone();
        let entity_name = entity.name.clone();
        let entity_pos = entity.position;
        let is_boss = matches!(entity.ai, Some(AIBehavior::Boss(_)));
        let is_elite = entity.elite.is_some();
        let is_ally = matches!(entity.ai, Some(AIBehavior::Ally { .. }));

        // Ally death: no XP/gold, just a message
        if is_ally {
            self.push_message(
                &format!("Your ally {} falls!", entity_name),
                LogSeverity::Danger,
            );
            self.remove_entity(entity_id);
            return events;
        }

        // Grant XP and gold to player
        if entity.ai.is_some() {
            let xp = entity.health.as_ref().map_or(0, |h| h.max as u32);
            // Pacifist modifier: no XP from kills
            if !self.modifiers.contains(&RunModifier::Pacifist) {
                self.player_xp += xp;
            }
            self.enemies_killed += 1;

            // Gold drop: 1-5 scaled by floor; elite gets 2x, boss gets 5x
            let gold_drop = self.rng.gen_range(1..=5) + self.floor;
            let gold_drop = if is_boss {
                gold_drop * 5
            } else if is_elite {
                gold_drop * 2
            } else {
                gold_drop
            };
            self.gold += gold_drop;
            events.push(GameEvent::GoldGained { amount: gold_drop });

            // Elite enemies have a 50% chance to drop an extra item
            if is_elite && self.rng.gen::<f32>() < 0.50 {
                let all_item_templates = crate::engine::items::all_items();
                if let Some(mut item) =
                    placement::pick_weighted_item(self.floor, &mut self.rng, &all_item_templates)
                {
                    item.position = entity_pos;
                    self.push_message(
                        &format!("The {} drops a {}!", entity_name, item.name),
                        LogSeverity::Good,
                    );
                    self.entities.push(item);
                }
            }

            if is_boss {
                self.bosses_killed += 1;
                events.push(GameEvent::BossDefeated {
                    name: entity_name.clone(),
                    floor: self.floor,
                });
                self.push_message(
                    &format!("{} has been defeated!", entity_name),
                    LogSeverity::Good,
                );

                // Check victory condition (floor 10 boss, or floor 20 for Marathon)
                let victory_floor = if self.modifiers.contains(&RunModifier::Marathon) {
                    20
                } else {
                    10
                };
                if self.floor == victory_floor {
                    self.victory = true;
                    self.game_over = true;
                    events.push(GameEvent::Victory);
                    self.push_message(
                        "Victory! You have conquered the dungeon!",
                        LogSeverity::Good,
                    );
                }
            } else {
                self.push_message(&format!("{} is defeated.", entity_name), LogSeverity::Good);
            }

            // Check level up
            let xp_to_next = self.player_level * 150;
            if self.player_xp >= xp_to_next {
                self.player_xp -= xp_to_next;
                self.player_level += 1;
                self.pending_level_up = true;
                events.push(GameEvent::LevelUp {
                    new_level: self.player_level,
                });
                self.push_message(
                    &format!("Level up! You are now level {}.", self.player_level),
                    LogSeverity::Good,
                );
            }
        }

        // Crypt biome: non-boss enemies get resurrection timer instead of removal
        let biome = Biome::for_floor(self.floor);
        if biome == Biome::Crypt && !is_boss && !is_ally {
            if let Some(entity) = self.get_entity_mut(entity_id) {
                entity.resurrection_timer = Some(5);
                entity.ai = None; // Remove AI so it doesn't act while "dead"
                entity.blocks_movement = false; // Don't block movement while dead
            }
            self.push_message(
                &format!("The {} collapses... but may rise again.", entity_name),
                LogSeverity::Warning,
            );
        } else {
            // Remove the dead entity
            self.remove_entity(entity_id);
        }

        events
    }

    fn process_enemy_turns(&mut self) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // Collect entities that can act
        let entity_ids: Vec<EntityId> = self
            .entities
            .iter()
            .filter(|e| e.ai.is_some() && e.combat.is_some() && e.id != self.player_id)
            .map(|e| e.id)
            .collect();

        // Grant energy to all combat entities
        for &id in &entity_ids {
            let speed = self
                .get_entity(id)
                .map(|e| combat::effective_speed(e))
                .unwrap_or(100);
            *self.energy.entry(id).or_insert(0) += speed;
        }

        // Also grant player energy (not used for action gating since player acts on input)
        let player_speed = self
            .get_entity(self.player_id)
            .map(|e| combat::effective_speed(e))
            .unwrap_or(100);
        *self.energy.entry(self.player_id).or_insert(0) += player_speed;

        // Process each enemy that has enough energy
        for &id in &entity_ids {
            if self.get_entity(id).is_none() {
                continue; // Entity was killed
            }

            let current_energy = *self.energy.get(&id).unwrap_or(&0);
            if current_energy >= ENERGY_THRESHOLD {
                *self.energy.entry(id).or_insert(0) -= ENERGY_THRESHOLD;

                // Check if stunned
                let is_stunned = self
                    .get_entity(id)
                    .map(|e| {
                        e.status_effects
                            .iter()
                            .any(|s| s.effect_type == StatusType::Stunned)
                    })
                    .unwrap_or(false);

                if is_stunned {
                    continue;
                }

                let enemy_events = self.resolve_enemy_turn(id);
                events.extend(enemy_events);

                // Check if player died
                if self.is_player_dead() {
                    break;
                }
            }
        }

        events
    }

    fn resolve_enemy_turn(&mut self, entity_id: EntityId) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let entity = match self.get_entity(entity_id) {
            Some(e) => e.clone(),
            None => return events,
        };

        let player = match self.get_entity(self.player_id) {
            Some(p) => p.clone(),
            None => return events,
        };

        // Boss summon check: Goblin King has a timed summon mechanic driven by
        // the boss_action_counter, independent of the AI decision.
        let is_goblin_king =
            entity.name == "Goblin King" && matches!(&entity.ai, Some(AIBehavior::Boss(_)));
        if is_goblin_king {
            let is_phase2 = matches!(&entity.ai, Some(AIBehavior::Boss(BossPhase::Phase2)));
            let counter = self.boss_action_counter.entry(entity_id).or_insert(0);
            *counter += 1;
            let interval = if is_phase2 { 3 } else { 4 };
            if *counter >= interval {
                *counter = 0;
                let summon_events = self.boss_summon_minions(entity_id, is_phase2);
                events.extend(summon_events);
                // Summoning consumes the turn
                return events;
            }
        }

        // Use AI module for decision making (handles confusion, fleeing, LOS, etc.)
        let action = ai::decide_action(&entity, &player, &self.dijkstra, &self.map, &self.entities);

        match action {
            ai::AIAction::MeleeAttack(_) | ai::AIAction::RangedAttack(_) => {
                events.extend(self.perform_attack(entity_id, self.player_id));
            }
            ai::AIAction::MoveToward(_) => {
                events.extend(self.move_toward_player(entity_id));
            }
            ai::AIAction::MoveAway(_) => {
                events.extend(self.move_away_from_player(entity_id));
            }
            ai::AIAction::MoveRandom => {
                events.extend(self.move_random(entity_id));
            }
            ai::AIAction::Wait => {}
            ai::AIAction::BossSummon { summon_archers } => {
                // Fallback if reached through AI (shouldn't normally happen for Goblin King)
                events.extend(self.boss_summon_minions(entity_id, summon_archers));
            }
            ai::AIAction::BossCharge { stun } => {
                events.extend(self.boss_charge(entity_id, stun));
            }
            ai::AIAction::BossTeleport => {
                events.extend(self.boss_teleport(entity_id));
            }
            ai::AIAction::BossFrostBolt => {
                events.extend(self.boss_frost_bolt(entity_id));
            }
        }

        events
    }

    // --- Boss-specific action methods ---

    /// Goblin King summons 1-2 Goblins (Phase 1) or Goblin Archers (Phase 2).
    fn boss_summon_minions(&mut self, boss_id: EntityId, summon_archers: bool) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let boss = match self.get_entity(boss_id) {
            Some(e) => e.clone(),
            None => return events,
        };

        let boss_pos = boss.position;
        let count = self.rng.gen_range(1..=2u32);
        let summon_name = if summon_archers {
            "Goblin Archer"
        } else {
            "Goblin"
        };

        let all_templates = enemies::all_enemies();
        let template = match all_templates.iter().find(|t| t.name == summon_name) {
            Some(t) => t,
            None => return events,
        };

        let mut summoned_names = Vec::new();

        // Find walkable tiles adjacent to boss
        for _ in 0..count {
            let spawn_pos = self.find_spawn_position_near(boss_pos, 2);
            if let Some(pos) = spawn_pos {
                let minion = Entity {
                    id: placement::next_id(),
                    name: template.name.to_string(),
                    position: pos,
                    glyph: template.glyph,
                    render_order: RenderOrder::Enemy,
                    blocks_movement: true,
                    blocks_fov: false,
                    health: Some(Health::new(template.hp)),
                    combat: Some(CombatStats {
                        base_attack: template.attack,
                        base_defense: template.defense,
                        base_speed: template.speed,
                        crit_chance: template.crit_chance,
                        dodge_chance: 0.0,
                        ranged: if summon_archers {
                            None // Ranged behavior is handled by AI, not stats
                        } else {
                            None
                        },
                        on_hit: None,
                    }),
                    ai: Some(template.ai.clone()),
                    inventory: None,
                    equipment: None,
                    item: None,
                    status_effects: Vec::new(),
                    fov: Some(FieldOfView::new(6)),
                    door: None,
                    trap: None,
                    stair: None,
                    loot_table: None,
                    flavor_text: None,
                    shop: None,
                    interactive: None,
                    elite: None,
                    resurrection_timer: None,
                };

                let minion_id = minion.id;
                summoned_names.push(minion.name.clone());
                self.energy.insert(minion_id, 0);
                self.entities.push(minion);
            }
        }

        if !summoned_names.is_empty() {
            let msg = format!("{} summons {}!", boss.name, summoned_names.join(" and "));
            self.push_message(&msg, LogSeverity::Danger);

            events.push(GameEvent::BossSummon {
                boss_name: boss.name.clone(),
                summoned: summoned_names,
            });
        }

        events
    }

    /// Troll Warlord charges to an adjacent tile next to the player and attacks with 2x damage.
    /// In Phase 2, also stuns the player for 1 turn.
    fn boss_charge(&mut self, boss_id: EntityId, stun: bool) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let boss = match self.get_entity(boss_id) {
            Some(e) => e.clone(),
            None => return events,
        };

        let player = match self.get_entity(self.player_id) {
            Some(p) => p.clone(),
            None => return events,
        };

        let boss_pos = boss.position;
        let player_pos = player.position;

        // Find an unblocked tile adjacent to the player to charge to
        let charge_target = Direction::ALL
            .iter()
            .map(|d| player_pos.apply_direction(*d))
            .find(|pos| {
                self.map.in_bounds(pos.x, pos.y)
                    && self.map.is_walkable(pos.x, pos.y)
                    && !self.is_blocked(*pos, boss_id)
            });

        if let Some(target_pos) = charge_target {
            // Move boss to charge position
            let from = boss_pos;
            self.move_entity(boss_id, target_pos);

            let boss_name = boss.name.clone();
            self.push_message(
                &format!("{} charges at you!", boss_name),
                LogSeverity::Danger,
            );

            events.push(GameEvent::BossCharge {
                boss_id,
                from,
                to: target_pos,
            });

            // Attack with 2x damage: temporarily boost attack, perform attack, restore
            let original_attack = self
                .get_entity(boss_id)
                .and_then(|e| e.combat.as_ref())
                .map(|c| c.base_attack)
                .unwrap_or(0);

            if let Some(boss_entity) = self.get_entity_mut(boss_id) {
                if let Some(ref mut combat) = boss_entity.combat {
                    combat.base_attack *= 2;
                }
            }

            events.extend(self.perform_attack(boss_id, self.player_id));

            // Restore original attack
            if let Some(boss_entity) = self.get_entity_mut(boss_id) {
                if let Some(ref mut combat) = boss_entity.combat {
                    combat.base_attack = original_attack;
                }
            }

            // Phase 2: stun the player for 1 turn
            if stun {
                let already_stunned = self
                    .get_entity(self.player_id)
                    .map(|p| {
                        p.status_effects
                            .iter()
                            .any(|s| s.effect_type == StatusType::Stunned)
                    })
                    .unwrap_or(false);

                if !already_stunned {
                    if let Some(player_entity) = self.get_entity_mut(self.player_id) {
                        player_entity.status_effects.push(StatusEffect {
                            effect_type: StatusType::Stunned,
                            duration: 1,
                            magnitude: 0,
                            source: "Troll Warlord charge".to_string(),
                        });
                    }
                    self.push_message("The charge stuns you!", LogSeverity::Danger);
                    events.push(GameEvent::StatusApplied {
                        entity_id: self.player_id,
                        effect: StatusType::Stunned,
                        duration: 1,
                    });
                }
            }
        } else {
            // No valid charge position: fall back to move toward player
            events.extend(self.move_toward_player(boss_id));
        }

        events
    }

    /// The Lich teleports 4-6 tiles away from the player and applies Burning to
    /// the player if they were adjacent (fire at old position).
    fn boss_teleport(&mut self, boss_id: EntityId) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let boss = match self.get_entity(boss_id) {
            Some(e) => e.clone(),
            None => return events,
        };

        let player = match self.get_entity(self.player_id) {
            Some(p) => p.clone(),
            None => return events,
        };

        let old_pos = boss.position;
        let player_pos = player.position;

        // Find a valid teleport destination 4-6 tiles away from the player
        let teleport_dest = self.find_teleport_destination(player_pos, 4, 6, boss_id);

        if let Some(new_pos) = teleport_dest {
            self.move_entity(boss_id, new_pos);

            self.push_message(
                &format!("{} vanishes in a flash of dark flame!", boss.name),
                LogSeverity::Danger,
            );

            events.push(GameEvent::Moved {
                entity_id: boss_id,
                from: old_pos,
                to: new_pos,
            });

            // Leave fire at old position: apply Burning to the player if adjacent
            // (i.e., the player was next to the Lich and stepped into the fire zone)
            let player_distance_to_old = player_pos.chebyshev_distance(&old_pos);
            if player_distance_to_old <= 1 {
                let already_burning = self
                    .get_entity(self.player_id)
                    .map(|p| {
                        p.status_effects
                            .iter()
                            .any(|s| s.effect_type == StatusType::Burning)
                    })
                    .unwrap_or(false);

                if !already_burning {
                    if let Some(player_entity) = self.get_entity_mut(self.player_id) {
                        player_entity.status_effects.push(StatusEffect {
                            effect_type: StatusType::Burning,
                            duration: 3,
                            magnitude: 4,
                            source: "Lich's dark flame".to_string(),
                        });
                    }
                    self.push_message("Dark flames sear you!", LogSeverity::Danger);
                    events.push(GameEvent::StatusApplied {
                        entity_id: self.player_id,
                        effect: StatusType::Burning,
                        duration: 3,
                    });
                }
            }

            // Phase 2 additional mechanic is handled by the AI returning BossFrostBolt
            // on subsequent turns (not during teleport turn).
        } else {
            // Couldn't teleport; fall back to melee attack
            events.extend(self.perform_attack(boss_id, self.player_id));
        }

        events
    }

    /// The Lich (Phase 2) fires a frost bolt at the player, dealing damage and applying Slowed.
    fn boss_frost_bolt(&mut self, boss_id: EntityId) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let boss = match self.get_entity(boss_id) {
            Some(e) => e.clone(),
            None => return events,
        };

        let player = match self.get_entity(self.player_id) {
            Some(p) => p.clone(),
            None => return events,
        };

        let boss_pos = boss.position;
        let player_pos = player.position;

        // Check LOS
        if !has_line_of_sight(&self.map, boss_pos, player_pos) {
            // No LOS, fall back to move
            events.extend(self.move_toward_player(boss_id));
            return events;
        }

        // Calculate frost bolt damage (base_attack * 0.8)
        let damage = boss
            .combat
            .as_ref()
            .map(|c| (c.base_attack as f32 * 0.8) as i32)
            .unwrap_or(5)
            .max(3);

        // Apply damage to player
        if let Some(player_entity) = self.get_entity_mut(self.player_id) {
            if let Some(ref mut health) = player_entity.health {
                health.current -= damage;
            }
        }

        self.last_damage_source = Some("Struck by the Lich's frost bolt".to_string());
        self.push_message(
            &format!(
                "{} hurls a frost bolt at you for {} damage!",
                boss.name, damage
            ),
            LogSeverity::Danger,
        );

        events.push(GameEvent::ProjectileFired {
            from: boss_pos,
            to: player_pos,
            hit: true,
        });

        events.push(GameEvent::DamageTaken {
            entity_id: self.player_id,
            amount: damage,
            source: "Frost Bolt".to_string(),
        });

        // Apply Slowed status
        let already_slowed = self
            .get_entity(self.player_id)
            .map(|p| {
                p.status_effects
                    .iter()
                    .any(|s| s.effect_type == StatusType::Slowed)
            })
            .unwrap_or(false);

        if !already_slowed {
            if let Some(player_entity) = self.get_entity_mut(self.player_id) {
                player_entity.status_effects.push(StatusEffect {
                    effect_type: StatusType::Slowed,
                    duration: 2,
                    magnitude: 30,
                    source: "Lich's frost bolt".to_string(),
                });
            }
            self.push_message(
                "The frost chills your bones, slowing you!",
                LogSeverity::Warning,
            );
            events.push(GameEvent::StatusApplied {
                entity_id: self.player_id,
                effect: StatusType::Slowed,
                duration: 2,
            });
        }

        events
    }

    /// Find a walkable, unblocked position within `radius` tiles of `center`.
    fn find_spawn_position_near(&mut self, center: Position, radius: i32) -> Option<Position> {
        let mut candidates = Vec::new();
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx == 0 && dy == 0 {
                    continue;
                }
                let pos = Position::new(center.x + dx, center.y + dy);
                if self.map.in_bounds(pos.x, pos.y)
                    && self.map.is_walkable(pos.x, pos.y)
                    && !self.is_blocked(pos, 0)
                // 0 won't match any entity
                {
                    candidates.push(pos);
                }
            }
        }
        if candidates.is_empty() {
            return None;
        }
        let idx = self.rng.gen_range(0..candidates.len());
        Some(candidates[idx])
    }

    /// Find a teleport destination `min_dist` to `max_dist` tiles away from `away_from`,
    /// on a walkable, unblocked tile.
    fn find_teleport_destination(
        &mut self,
        away_from: Position,
        min_dist: i32,
        max_dist: i32,
        self_id: EntityId,
    ) -> Option<Position> {
        let mut candidates = Vec::new();
        for dy in -max_dist..=max_dist {
            for dx in -max_dist..=max_dist {
                let pos = Position::new(away_from.x + dx, away_from.y + dy);
                let dist = pos.chebyshev_distance(&away_from);
                if dist >= min_dist
                    && dist <= max_dist
                    && self.map.in_bounds(pos.x, pos.y)
                    && self.map.is_walkable(pos.x, pos.y)
                    && !self.is_blocked(pos, self_id)
                {
                    candidates.push(pos);
                }
            }
        }
        if candidates.is_empty() {
            return None;
        }
        let idx = self.rng.gen_range(0..candidates.len());
        Some(candidates[idx])
    }

    fn move_toward_player(&mut self, entity_id: EntityId) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let entity_pos = match self.get_entity(entity_id) {
            Some(e) => e.position,
            None => return events,
        };

        if let Some(ref dijkstra) = self.dijkstra {
            if let Some(next_pos) = dijkstra.best_neighbor(entity_pos, &self.map) {
                if !self.is_blocked(next_pos, entity_id) {
                    let from = entity_pos;
                    self.move_entity(entity_id, next_pos);
                    events.push(GameEvent::Moved {
                        entity_id,
                        from,
                        to: next_pos,
                    });
                }
            }
        }

        events
    }

    fn move_away_from_player(&mut self, entity_id: EntityId) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let entity_pos = match self.get_entity(entity_id) {
            Some(e) => e.position,
            None => return events,
        };

        if let Some(ref dijkstra) = self.dijkstra {
            if let Some(next_pos) = dijkstra.flee_neighbor(entity_pos, &self.map) {
                if !self.is_blocked(next_pos, entity_id) {
                    let from = entity_pos;
                    self.move_entity(entity_id, next_pos);
                    events.push(GameEvent::Moved {
                        entity_id,
                        from,
                        to: next_pos,
                    });
                }
            }
        }

        events
    }

    fn move_random(&mut self, entity_id: EntityId) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let entity_pos = match self.get_entity(entity_id) {
            Some(e) => e.position,
            None => return events,
        };

        let directions = [
            Direction::N,
            Direction::S,
            Direction::E,
            Direction::W,
            Direction::NE,
            Direction::NW,
            Direction::SE,
            Direction::SW,
        ];
        let dir = directions[self.rng.gen_range(0..directions.len())];
        let new_pos = entity_pos.apply_direction(dir);

        if self.map.in_bounds(new_pos.x, new_pos.y)
            && self.map.is_walkable(new_pos.x, new_pos.y)
            && !self.is_blocked(new_pos, entity_id)
        {
            let from = entity_pos;
            self.move_entity(entity_id, new_pos);
            events.push(GameEvent::Moved {
                entity_id,
                from,
                to: new_pos,
            });
        }

        events
    }

    fn tick_status_effects(&mut self) -> Vec<GameEvent> {
        let mut events = Vec::new();
        let entity_ids: Vec<EntityId> = self.entities.iter().map(|e| e.id).collect();

        for id in entity_ids {
            let effects: Vec<StatusEffect> = self
                .get_entity(id)
                .map(|e| e.status_effects.clone())
                .unwrap_or_default();

            for effect in &effects {
                match effect.effect_type {
                    StatusType::Poison => {
                        let damage = effect.magnitude.max(2);
                        if id == self.player_id {
                            self.last_damage_source = Some("Succumbed to poison".to_string());
                        }
                        if let Some(entity) = self.get_entity_mut(id) {
                            if let Some(ref mut health) = entity.health {
                                health.current -= damage;
                            }
                        }
                        let name = self
                            .get_entity(id)
                            .map(|e| e.name.clone())
                            .unwrap_or_default();
                        events.push(GameEvent::DamageTaken {
                            entity_id: id,
                            amount: damage,
                            source: "poison".to_string(),
                        });
                        self.push_message(
                            &format!("{} takes {} poison damage.", name, damage),
                            LogSeverity::Warning,
                        );
                    }
                    StatusType::Burning => {
                        let damage = effect.magnitude.max(3);
                        if id == self.player_id {
                            self.last_damage_source = Some("Burned to death".to_string());
                        }
                        if let Some(entity) = self.get_entity_mut(id) {
                            if let Some(ref mut health) = entity.health {
                                health.current -= damage;
                            }
                        }
                        let name = self
                            .get_entity(id)
                            .map(|e| e.name.clone())
                            .unwrap_or_default();
                        events.push(GameEvent::DamageTaken {
                            entity_id: id,
                            amount: damage,
                            source: "fire".to_string(),
                        });
                        self.push_message(
                            &format!("{} takes {} fire damage.", name, damage),
                            LogSeverity::Warning,
                        );
                    }
                    StatusType::Regenerating => {
                        let heal = effect.magnitude.max(2);
                        if let Some(entity) = self.get_entity_mut(id) {
                            if let Some(ref mut health) = entity.health {
                                health.current = (health.current + heal).min(health.max);
                            }
                        }
                        events.push(GameEvent::Healed {
                            entity_id: id,
                            amount: heal,
                        });
                    }
                    _ => {}
                }
            }

            // Decrement durations and remove expired
            if let Some(entity) = self.get_entity_mut(id) {
                let mut expired = Vec::new();
                for effect in &mut entity.status_effects {
                    if effect.duration > 0 {
                        effect.duration -= 1;
                        if effect.duration == 0 {
                            expired.push(effect.effect_type);
                        }
                    }
                }
                entity.status_effects.retain(|e| e.duration > 0);

                for effect_type in expired {
                    events.push(GameEvent::StatusExpired {
                        entity_id: id,
                        effect: effect_type,
                    });
                }
            }
        }

        events
    }

    fn tick_hunger(&mut self) -> Vec<GameEvent> {
        let mut events = Vec::new();
        if self.hunger <= 0 {
            // Starving: 1 damage every 5 turns
            if self.turn % 5 == 0 {
                self.last_damage_source = Some("Starved to death".to_string());
                if let Some(player) = self.get_entity_mut(self.player_id) {
                    if let Some(ref mut health) = player.health {
                        health.current -= 1;
                    }
                }
                events.push(GameEvent::DamageTaken {
                    entity_id: self.player_id,
                    amount: 1,
                    source: "starvation".to_string(),
                });
                self.push_message("You are starving!", LogSeverity::Danger);
            }
            return events;
        }

        self.hunger -= 1;

        // Threshold messages
        match self.hunger {
            500 => self.push_message("You are getting hungry.", LogSeverity::Warning),
            250 => self.push_message("You are very hungry!", LogSeverity::Warning),
            100 => self.push_message("You are famished!", LogSeverity::Danger),
            0 => self.push_message("You are starving! Find food!", LogSeverity::Danger),
            _ => {}
        }

        events.push(GameEvent::HungerChanged { level: self.hunger });
        events
    }

    fn apply_cursed_to_items(&mut self) {
        for entity in &mut self.entities {
            if entity.id == self.player_id {
                continue; // Don't modify player's existing inventory
            }
            if let Some(ref mut item) = entity.item {
                if item.item_type.is_consumable() {
                    item.identified = false;
                }
            }
        }
    }

    fn tick_floor_effects(&mut self) -> Vec<GameEvent> {
        let mut events = Vec::new();
        let biome = Biome::for_floor(self.floor);

        match biome {
            Biome::Crypt => {
                // Undead resurrection: decrement timers, resurrect at 0
                let mut resurrect_ids = Vec::new();
                for entity in &mut self.entities {
                    if let Some(ref mut timer) = entity.resurrection_timer {
                        if *timer > 0 {
                            *timer -= 1;
                            if *timer == 0 {
                                resurrect_ids.push(entity.id);
                            }
                        }
                    }
                }
                for id in resurrect_ids {
                    if let Some(entity) = self.get_entity_mut(id) {
                        // Resurrect at 50% HP
                        if let Some(ref mut health) = entity.health {
                            health.current = health.max / 2;
                        }
                        entity.resurrection_timer = None;
                        entity.ai = Some(AIBehavior::Melee); // Restore AI
                        let name = entity.name.clone();
                        let pos = entity.position;
                        self.energy.insert(id, 0);
                        self.push_message(
                            &format!("The {} rises from the dead!", name),
                            LogSeverity::Danger,
                        );
                        events.push(GameEvent::EnemySpotted {
                            entity_id: id,
                            name,
                        });
                        // Mark as spotted again
                        self.spotted_enemies.remove(&id);
                        // Re-insert to visible set at position
                        let _ = pos; // position is already set on the entity
                    }
                }
            }
            Biome::Abyss => {
                // Reduce player FOV radius by 3 (applied in recompute_fov via check)
                // This is handled in recompute_fov, no per-turn action needed here
            }
            _ => {}
        }

        events
    }

    fn check_spotted_enemies(&mut self) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let visible = self
            .get_entity(self.player_id)
            .and_then(|e| e.fov.as_ref())
            .map(|f| f.visible_tiles.clone())
            .unwrap_or_default();

        let newly_spotted: Vec<(EntityId, String)> = self
            .entities
            .iter()
            .filter(|e| {
                e.ai.is_some()
                    && visible.contains(&e.position)
                    && !self.spotted_enemies.contains(&e.id)
            })
            .map(|e| (e.id, e.name.clone()))
            .collect();

        for (id, name) in newly_spotted {
            self.spotted_enemies.insert(id);
            events.push(GameEvent::EnemySpotted {
                entity_id: id,
                name,
            });
        }

        events
    }

    fn check_traps(&mut self, entity_id: EntityId, pos: Position) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // Find trap at position
        let trap_id = self
            .entities
            .iter()
            .find(|e| e.position == pos && e.trap.is_some() && !e.trap.as_ref().unwrap().triggered)
            .map(|e| e.id);

        let trap_id = match trap_id {
            Some(id) => id,
            None => return events,
        };

        let trap = self.get_entity(trap_id).unwrap().clone();
        let trap_props = trap.trap.as_ref().unwrap().clone();

        // Mark trap as triggered
        if let Some(trap_entity) = self.get_entity_mut(trap_id) {
            if let Some(ref mut t) = trap_entity.trap {
                t.triggered = true;
                t.revealed = true;
            }
        }

        match &trap_props.trap_type {
            TrapType::Spike { damage } => {
                let damage = *damage;
                if entity_id == self.player_id {
                    self.last_damage_source = Some("Killed by a spike trap".to_string());
                }
                if let Some(entity) = self.get_entity_mut(entity_id) {
                    if let Some(ref mut health) = entity.health {
                        health.current -= damage;
                    }
                }
                events.push(GameEvent::TrapTriggered {
                    position: pos,
                    trap_type: "Spike".to_string(),
                    damage,
                });
                self.push_message(
                    &format!("A spike trap springs! {} damage!", damage),
                    LogSeverity::Danger,
                );
            }
            TrapType::Poison { damage, duration } => {
                let damage = *damage;
                let duration = *duration;
                if entity_id == self.player_id {
                    self.last_damage_source = Some("Killed by a poison trap".to_string());
                }
                if let Some(entity) = self.get_entity_mut(entity_id) {
                    entity.status_effects.push(StatusEffect {
                        effect_type: StatusType::Poison,
                        duration,
                        magnitude: damage,
                        source: "trap".to_string(),
                    });
                }
                events.push(GameEvent::TrapTriggered {
                    position: pos,
                    trap_type: "Poison".to_string(),
                    damage: 0,
                });
                events.push(GameEvent::StatusApplied {
                    entity_id,
                    effect: StatusType::Poison,
                    duration,
                });
                self.push_message("A poison trap activates!", LogSeverity::Danger);
            }
            TrapType::Teleport => {
                // Teleport to random floor tile
                let floor_tiles: Vec<Position> = (0..self.map.width as i32)
                    .flat_map(|x| (0..self.map.height as i32).map(move |y| Position::new(x, y)))
                    .filter(|p| {
                        self.map.get_tile(p.x, p.y) == TileType::Floor
                            && !self.is_blocked(*p, entity_id)
                    })
                    .collect();

                if floor_tiles.is_empty() {
                    self.push_message("The teleport trap fizzles...", LogSeverity::Info);
                } else {
                    let idx = self.rng.gen_range(0..floor_tiles.len());
                    let new_pos = floor_tiles[idx];
                    let from = self.get_entity(entity_id).unwrap().position;
                    self.move_entity(entity_id, new_pos);
                    events.push(GameEvent::Moved {
                        entity_id,
                        from,
                        to: new_pos,
                    });
                    self.push_message("A teleport trap whisks you away!", LogSeverity::Warning);
                }
                events.push(GameEvent::TrapTriggered {
                    position: pos,
                    trap_type: "Teleport".to_string(),
                    damage: 0,
                });
            }
            TrapType::Alarm => {
                events.push(GameEvent::TrapTriggered {
                    position: pos,
                    trap_type: "Alarm".to_string(),
                    damage: 0,
                });
                self.push_message("An alarm sounds! Enemies are alerted!", LogSeverity::Danger);
                // Could alert nearby enemies - AI task can handle this
            }
        }

        events
    }

    fn try_open_door(&mut self, door_id: EntityId) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let door = self.get_entity(door_id).unwrap().clone();
        let door_state = door.door.as_ref().unwrap();
        let pos = door.position;

        if door_state.locked {
            // Check if player has the right key
            let key_name = door_state
                .key_id
                .clone()
                .unwrap_or_else(|| "Boss Key".to_string());
            let has_key = self.player_has_item(&key_name);

            if has_key {
                self.remove_player_item(&key_name);
                if let Some(d) = self.get_entity_mut(door_id) {
                    if let Some(ref mut ds) = d.door {
                        ds.locked = false;
                        ds.open = true;
                    }
                }
                self.map.set_tile(pos.x, pos.y, TileType::DoorOpen);
                events.push(GameEvent::DoorOpened { position: pos });
                self.push_message("You unlock and open the door.", LogSeverity::Good);
            } else {
                self.push_message("The door is locked. You need a key.", LogSeverity::Warning);
            }
        } else {
            if let Some(d) = self.get_entity_mut(door_id) {
                if let Some(ref mut ds) = d.door {
                    ds.open = true;
                }
            }
            self.map.set_tile(pos.x, pos.y, TileType::DoorOpen);
            events.push(GameEvent::DoorOpened { position: pos });
            self.push_message("You open the door.", LogSeverity::Info);
        }

        events
    }

    fn try_pickup(&mut self) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let player_pos = self.get_entity(self.player_id).unwrap().position;

        // Find item at player position
        let item_id = self
            .entities
            .iter()
            .find(|e| e.position == player_pos && e.item.is_some() && e.id != self.player_id)
            .map(|e| e.id);

        let item_id = match item_id {
            Some(id) => id,
            None => {
                self.push_message("Nothing to pick up here.", LogSeverity::Info);
                return events;
            }
        };

        // Check inventory space
        let is_full = self
            .get_entity(self.player_id)
            .and_then(|e| e.inventory.as_ref())
            .map(|inv| inv.is_full())
            .unwrap_or(true);

        if is_full {
            self.push_message("Your inventory is full!", LogSeverity::Warning);
            return events;
        }

        // Remove from world and add to inventory
        let item_entity = self.remove_entity(item_id);
        if let Some(item) = item_entity {
            let item_view = entity_to_item_view(&item);
            self.push_message(
                &format!("You pick up the {}.", item.name),
                LogSeverity::Good,
            );
            events.push(GameEvent::ItemPickedUp { item: item_view });

            if let Some(player) = self.get_entity_mut(self.player_id) {
                if let Some(ref mut inv) = player.inventory {
                    inv.items.push(item);
                }
            }
        }

        events
    }

    fn try_use_stairs(&mut self) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let player_pos = self.get_entity(self.player_id).unwrap().position;

        let stair = self
            .entities
            .iter()
            .find(|e| e.position == player_pos && e.stair == Some(StairDirection::Down));

        if stair.is_none() {
            self.push_message("There are no stairs here.", LogSeverity::Info);
            return events;
        }

        // Descend to next floor
        self.floor += 1;
        self.push_message(
            &format!("You descend to floor {}.", self.floor),
            LogSeverity::Info,
        );
        events.push(GameEvent::StairsDescended {
            new_floor: self.floor,
        });

        // Generate new floor
        self.map = dungeon::generate_floor(self.seed, self.floor);

        // Find start room and move player there
        let start_pos = self
            .map
            .rooms
            .iter()
            .find(|r| r.room_type == super::map::RoomType::Start)
            .map(|r| r.center())
            .unwrap_or(Position::new(1, 1));

        // Keep player entity, remove everything else
        let player = self.get_entity(self.player_id).unwrap().clone();
        self.entities.clear();
        self.energy.clear();
        self.spotted_enemies.clear();

        let mut player = player;
        player.position = start_pos;
        if let Some(ref mut f) = player.fov {
            f.dirty = true;
        }
        self.entities.push(player);

        // Spawn new floor entities
        let floor_entities = placement::spawn_entities(&self.map, self.floor, &mut self.rng);
        self.entities.extend(floor_entities);

        // Place stairs
        place_stairs(&self.map, &mut self.entities);

        // Cursed modifier: mark all new floor items as unidentified
        if self.modifiers.contains(&RunModifier::Cursed) {
            self.apply_cursed_to_items();
        }

        // Re-init energy
        for entity in &self.entities {
            if entity.combat.is_some() {
                self.energy.insert(entity.id, 0);
            }
        }

        // Clear boss action counters for new floor
        self.boss_action_counter.clear();

        // Recompute FOV and Dijkstra
        self.recompute_fov();
        self.recompute_dijkstra();

        events
    }

    fn try_use_item(&mut self, item_idx: u32) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let item = {
            let player = match self.get_entity(self.player_id) {
                Some(p) => p,
                None => return events,
            };
            let inv = match &player.inventory {
                Some(inv) => inv,
                None => return events,
            };
            match inv.items.get(item_idx as usize) {
                Some(item) => item.clone(),
                None => {
                    self.push_message("Invalid item.", LogSeverity::Warning);
                    return events;
                }
            }
        };

        let item_props = match &item.item {
            Some(props) => props.clone(),
            None => return events,
        };

        if !item_props.item_type.is_consumable() && item_props.item_type != ItemType::Wand {
            self.push_message(
                "You can't use that item directly. Try equipping it.",
                LogSeverity::Info,
            );
            return events;
        }

        let item_view = entity_to_item_view(&item);
        let effect_desc;

        match &item_props.effect {
            Some(ItemEffect::Heal(amount)) => {
                let amount = *amount;
                if let Some(player) = self.get_entity_mut(self.player_id) {
                    if let Some(ref mut health) = player.health {
                        let healed = amount.min(health.max - health.current);
                        health.current = (health.current + amount).min(health.max);
                        events.push(GameEvent::Healed {
                            entity_id: self.player_id,
                            amount: healed,
                        });
                    }
                }
                effect_desc = format!("healed {} HP", amount);
                self.push_message(
                    &format!("You drink the {}. Healed {} HP.", item.name, amount),
                    LogSeverity::Good,
                );
            }
            Some(ItemEffect::RevealMap) => {
                self.map.reveal_all();
                effect_desc = "revealed the map".to_string();
                self.push_message("The map is revealed!", LogSeverity::Good);
            }
            Some(ItemEffect::RevealSecrets) => {
                let mut count = 0;
                for y in 0..self.map.height as i32 {
                    for x in 0..self.map.width as i32 {
                        if self.map.get_tile(x, y) == TileType::SecretWall {
                            self.map.reveal(x, y);
                            count += 1;
                        }
                    }
                }
                if count > 0 {
                    effect_desc = "revealed hidden passages".to_string();
                    self.push_message(
                        &format!(
                            "You sense {} hidden passage{}!",
                            count,
                            if count > 1 { "s" } else { "" }
                        ),
                        LogSeverity::Good,
                    );
                } else {
                    effect_desc = "found nothing hidden".to_string();
                    self.push_message(
                        "You sense no hidden passages on this floor.",
                        LogSeverity::Info,
                    );
                }
            }
            Some(ItemEffect::Teleport) => {
                let floor_tiles: Vec<Position> = (0..self.map.width as i32)
                    .flat_map(|x| (0..self.map.height as i32).map(move |y| Position::new(x, y)))
                    .filter(|p| {
                        self.map.get_tile(p.x, p.y) == TileType::Floor
                            && !self.is_blocked(*p, self.player_id)
                    })
                    .collect();

                if floor_tiles.is_empty() {
                    effect_desc = "teleportation failed".to_string();
                    self.push_message("The teleportation fizzles...", LogSeverity::Warning);
                } else {
                    let idx = self.rng.gen_range(0..floor_tiles.len());
                    let new_pos = floor_tiles[idx];
                    let from = self.get_entity(self.player_id).unwrap().position;
                    self.move_entity(self.player_id, new_pos);
                    events.push(GameEvent::Moved {
                        entity_id: self.player_id,
                        from,
                        to: new_pos,
                    });
                    effect_desc = "teleported".to_string();
                    self.push_message("You are teleported!", LogSeverity::Info);
                }
            }
            Some(ItemEffect::CureStatus) => {
                if let Some(player) = self.get_entity_mut(self.player_id) {
                    player
                        .status_effects
                        .retain(|s| !s.effect_type.is_negative());
                }
                effect_desc = "cured status effects".to_string();
                self.push_message("Your ailments are cured!", LogSeverity::Good);
            }
            Some(ItemEffect::ApplyStatus { effect, duration }) => {
                let effect = *effect;
                let duration = *duration;
                if let Some(player) = self.get_entity_mut(self.player_id) {
                    // Remove existing same-type effect (refresh)
                    player.status_effects.retain(|s| s.effect_type != effect);
                    player.status_effects.push(StatusEffect {
                        effect_type: effect,
                        duration,
                        magnitude: 0,
                        source: item.name.clone(),
                    });
                }
                events.push(GameEvent::StatusApplied {
                    entity_id: self.player_id,
                    effect,
                    duration,
                });
                effect_desc = format!("applied {:?}", effect);
                self.push_message(
                    &format!("You feel the effects of the {}.", item.name),
                    LogSeverity::Info,
                );
            }
            Some(ItemEffect::DamageArea { damage, radius }) => {
                let damage = *damage + self.spell_power_bonus;
                let radius = *radius;
                let player_pos = self.get_entity(self.player_id).unwrap().position;

                let targets: Vec<EntityId> = self
                    .entities
                    .iter()
                    .filter(|e| {
                        e.ai.is_some() && e.position.chebyshev_distance(&player_pos) <= radius
                    })
                    .map(|e| e.id)
                    .collect();

                for target_id in targets {
                    if let Some(target) = self.get_entity_mut(target_id) {
                        if let Some(ref mut health) = target.health {
                            health.current -= damage;
                        }
                    }
                    events.push(GameEvent::DamageTaken {
                        entity_id: target_id,
                        amount: damage,
                        source: "fireball".to_string(),
                    });

                    // Check death
                    let dead = self
                        .get_entity(target_id)
                        .and_then(|e| e.health.as_ref())
                        .map(|h| h.is_dead())
                        .unwrap_or(false);
                    if dead {
                        events.extend(self.handle_entity_death(target_id));
                    }
                }

                effect_desc = format!("dealt {} damage in radius {}", damage, radius);
                self.push_message(
                    &format!("A fireball explodes! {} damage!", damage),
                    LogSeverity::Danger,
                );
            }
            Some(ItemEffect::RangedAttack { damage, status }) => {
                // For wands, check charges
                let charges = item_props.charges;
                if let Some(c) = charges {
                    if c == 0 {
                        self.push_message("The wand is out of charges.", LogSeverity::Warning);
                        return events;
                    }
                }

                // Find nearest visible enemy
                let player_pos = self.get_entity(self.player_id).unwrap().position;
                let player_fov = self
                    .get_entity(self.player_id)
                    .and_then(|e| e.fov.as_ref())
                    .map(|f| f.visible_tiles.clone())
                    .unwrap_or_default();

                let nearest_enemy = self
                    .entities
                    .iter()
                    .filter(|e| e.ai.is_some() && player_fov.contains(&e.position))
                    .min_by_key(|e| e.position.chebyshev_distance(&player_pos))
                    .map(|e| e.id);

                if let Some(target_id) = nearest_enemy {
                    let damage = *damage;
                    if let Some(target) = self.get_entity_mut(target_id) {
                        if let Some(ref mut health) = target.health {
                            health.current -= damage;
                        }
                    }
                    let target_name = self
                        .get_entity(target_id)
                        .map(|e| e.name.clone())
                        .unwrap_or_default();
                    events.push(GameEvent::DamageTaken {
                        entity_id: target_id,
                        amount: damage,
                        source: item.name.clone(),
                    });
                    self.push_message(
                        &format!(
                            "The {} zaps {} for {} damage!",
                            item.name, target_name, damage
                        ),
                        LogSeverity::Info,
                    );

                    // Apply status if applicable
                    if let Some((status_type, duration)) = status {
                        if let Some(target) = self.get_entity_mut(target_id) {
                            target.status_effects.push(StatusEffect {
                                effect_type: *status_type,
                                duration: *duration,
                                magnitude: 0,
                                source: item.name.clone(),
                            });
                        }
                    }

                    let dead = self
                        .get_entity(target_id)
                        .and_then(|e| e.health.as_ref())
                        .map(|h| h.is_dead())
                        .unwrap_or(false);
                    if dead {
                        events.extend(self.handle_entity_death(target_id));
                    }

                    // Decrement charges
                    if let Some(player) = self.get_entity_mut(self.player_id) {
                        if let Some(ref mut inv) = player.inventory {
                            if let Some(inv_item) = inv.items.iter_mut().find(|i| i.id == item.id) {
                                if let Some(ref mut props) = inv_item.item {
                                    if let Some(ref mut c) = props.charges {
                                        *c = c.saturating_sub(1);
                                    }
                                }
                            }
                        }
                    }

                    effect_desc = format!("zapped for {} damage", damage);
                } else {
                    self.push_message("No visible targets.", LogSeverity::Info);
                    return events;
                }
            }
            None => {
                self.push_message("This item has no effect.", LogSeverity::Info);
                return events;
            }
        }

        events.push(GameEvent::ItemUsed {
            item: item_view,
            effect: effect_desc,
        });

        // Restore hunger if item has hunger_restore
        if item_props.hunger_restore > 0 {
            let old_hunger = self.hunger;
            self.hunger = (self.hunger + item_props.hunger_restore).min(self.max_hunger);
            let restored = self.hunger - old_hunger;
            if restored > 0 {
                self.push_message(
                    &format!("You feel satiated. (+{} fullness)", restored),
                    LogSeverity::Good,
                );
                events.push(GameEvent::HungerChanged { level: self.hunger });
            }
        }

        // Cursed: mark item as identified on use (so player learns its true name)
        if !item_props.identified {
            if let Some(player) = self.get_entity_mut(self.player_id) {
                if let Some(ref mut inv) = player.inventory {
                    if let Some(inv_item) = inv.items.iter_mut().find(|i| i.id == item.id) {
                        if let Some(ref mut props) = inv_item.item {
                            props.identified = true;
                        }
                    }
                }
            }
        }

        // Remove consumable from inventory (but not wands)
        if item_props.item_type.is_consumable() {
            if let Some(player) = self.get_entity_mut(self.player_id) {
                if let Some(ref mut inv) = player.inventory {
                    inv.items.retain(|i| i.id != item.id);
                }
            }
        }

        events
    }

    fn try_drop_item(&mut self, item_idx: u32) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let player_pos = self.get_entity(self.player_id).unwrap().position;

        let item = {
            let player = match self.get_entity(self.player_id) {
                Some(p) => p,
                None => return events,
            };
            let inv = match &player.inventory {
                Some(inv) => inv,
                None => return events,
            };
            match inv.items.get(item_idx as usize) {
                Some(item) => item.clone(),
                None => {
                    self.push_message("Invalid item.", LogSeverity::Warning);
                    return events;
                }
            }
        };

        let item_view = entity_to_item_view(&item);

        // Check if item is equipped
        let is_equipped = {
            let player = self.get_entity(self.player_id).unwrap();
            if let Some(equip) = &player.equipment {
                [
                    equip.main_hand,
                    equip.off_hand,
                    equip.head,
                    equip.body,
                    equip.ring,
                    equip.amulet,
                ]
                .iter()
                .any(|slot| *slot == Some(item.id))
            } else {
                false
            }
        };

        if is_equipped {
            self.push_message("Unequip the item first.", LogSeverity::Warning);
            return events;
        }

        // Remove from inventory and place on floor
        let mut dropped_item = item;
        dropped_item.position = player_pos;

        if let Some(player) = self.get_entity_mut(self.player_id) {
            if let Some(ref mut inv) = player.inventory {
                inv.items.retain(|i| i.id != dropped_item.id);
            }
        }

        self.push_message(
            &format!("You drop the {}.", dropped_item.name),
            LogSeverity::Info,
        );
        events.push(GameEvent::ItemDropped { item: item_view });

        self.entities.push(dropped_item);

        events
    }

    fn try_equip_item(&mut self, item_idx: u32) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let item = {
            let player = match self.get_entity(self.player_id) {
                Some(p) => p,
                None => return events,
            };
            let inv = match &player.inventory {
                Some(inv) => inv,
                None => return events,
            };
            match inv.items.get(item_idx as usize) {
                Some(item) => item.clone(),
                None => {
                    self.push_message("Invalid item.", LogSeverity::Warning);
                    return events;
                }
            }
        };

        let item_props = match &item.item {
            Some(props) => props.clone(),
            None => return events,
        };

        let slot = match item_props.slot {
            Some(s) => s,
            None => {
                self.push_message("This item can't be equipped.", LogSeverity::Info);
                return events;
            }
        };

        let item_view = entity_to_item_view(&item);

        // Check if there's already an item in this slot — unequip it first
        let prev_item_id = self
            .get_entity(self.player_id)
            .and_then(|p| p.equipment.as_ref())
            .and_then(|e| e.get_slot(slot));

        if let Some(prev_id) = prev_item_id {
            // Clear the old slot before equipping new item
            if let Some(player) = self.get_entity_mut(self.player_id) {
                if let Some(ref mut equip) = player.equipment {
                    equip.set_slot(slot, None);
                }
            }
            let prev_name = self
                .get_entity(self.player_id)
                .and_then(|p| p.inventory.as_ref())
                .and_then(|inv| inv.items.iter().find(|i| i.id == prev_id))
                .map(|i| i.name.clone())
                .unwrap_or_else(|| "item".to_string());
            self.push_message(
                &format!("You unequip the {}.", prev_name),
                LogSeverity::Info,
            );
        }

        // Equip the new item
        if let Some(player) = self.get_entity_mut(self.player_id) {
            if let Some(ref mut equip) = player.equipment {
                equip.set_slot(slot, Some(item.id));
            }
        }

        self.push_message(&format!("You equip the {}.", item.name), LogSeverity::Good);
        events.push(GameEvent::ItemEquipped {
            item: item_view,
            slot,
        });

        events
    }

    fn try_unequip_slot(&mut self, slot: EquipSlot) -> Vec<GameEvent> {
        let events = Vec::new();

        let current_id = {
            let player = self.get_entity(self.player_id).unwrap();
            player.equipment.as_ref().and_then(|e| e.get_slot(slot))
        };

        match current_id {
            Some(_id) => {
                if let Some(player) = self.get_entity_mut(self.player_id) {
                    if let Some(ref mut equip) = player.equipment {
                        equip.set_slot(slot, None);
                    }
                }
                self.push_message("You unequip the item.", LogSeverity::Info);
            }
            None => {
                self.push_message("Nothing equipped in that slot.", LogSeverity::Info);
            }
        }

        events
    }

    fn apply_level_up(&mut self, choice: LevelUpChoice) -> Vec<GameEvent> {
        if !self.pending_level_up {
            return Vec::new();
        }

        self.pending_level_up = false;

        let desc = match choice {
            LevelUpChoice::MaxHp => "+10 Max HP!",
            LevelUpChoice::Attack => "+2 Attack!",
            LevelUpChoice::Defense => "+2 Defense!",
            LevelUpChoice::Speed => "+15 Speed!",
            LevelUpChoice::Cleave => "+1 Cleave bonus!",
            LevelUpChoice::Fortify => "+3 Defense!",
            LevelUpChoice::Backstab => "+5% Crit chance!",
            LevelUpChoice::Evasion => "+5% Dodge chance!",
            LevelUpChoice::SpellPower => "+5 Spell Power!",
            LevelUpChoice::ManaRegen => "+1 Mana Regen!",
        };

        // Handle World-level bonuses
        match choice {
            LevelUpChoice::Cleave => self.cleave_bonus += 1,
            LevelUpChoice::SpellPower => self.spell_power_bonus += 5,
            LevelUpChoice::ManaRegen => self.mana_regen += 1,
            _ => {}
        }

        if let Some(player) = self.get_entity_mut(self.player_id) {
            super::level::apply_level_up_choice(player, choice);
        }

        self.push_message(desc, LogSeverity::Good);

        Vec::new()
    }

    // --- Helpers ---

    fn push_message(&mut self, text: &str, severity: LogSeverity) {
        self.messages.push(LogMessage {
            text: text.to_string(),
            turn: self.turn,
            severity,
        });
    }

    pub fn get_entity(&self, id: EntityId) -> Option<&Entity> {
        self.entities.iter().find(|e| e.id == id)
    }

    pub fn get_entity_mut(&mut self, id: EntityId) -> Option<&mut Entity> {
        self.entities.iter_mut().find(|e| e.id == id)
    }

    fn remove_entity(&mut self, id: EntityId) -> Option<Entity> {
        let idx = self.entities.iter().position(|e| e.id == id)?;
        self.energy.remove(&id);
        Some(self.entities.remove(idx))
    }

    fn move_entity(&mut self, id: EntityId, pos: Position) {
        if let Some(entity) = self.get_entity_mut(id) {
            entity.position = pos;
            if let Some(ref mut f) = entity.fov {
                f.dirty = true;
            }
        }
    }

    fn can_move_to(&self, pos: Position) -> bool {
        if !self.map.in_bounds(pos.x, pos.y) {
            return false;
        }
        if !self.map.is_walkable(pos.x, pos.y) {
            // Check if it's a closed door (walkable after opening)
            if self.map.get_tile(pos.x, pos.y) == TileType::DoorClosed {
                return true;
            }
            return false;
        }
        // Check for blocking entities
        !self
            .entities
            .iter()
            .any(|e| e.position == pos && e.blocks_movement && e.id != self.player_id)
    }

    fn is_blocked(&self, pos: Position, self_id: EntityId) -> bool {
        if !self.map.in_bounds(pos.x, pos.y) || !self.map.is_walkable(pos.x, pos.y) {
            return true;
        }
        self.entities
            .iter()
            .any(|e| e.position == pos && e.blocks_movement && e.id != self_id)
    }

    fn hostile_entity_at(&self, pos: Position) -> Option<EntityId> {
        self.entities
            .iter()
            .find(|e| {
                e.position == pos
                    && e.ai.is_some()
                    && !matches!(&e.ai, Some(AIBehavior::Ally { .. }))
                    && e.health.is_some()
            })
            .map(|e| e.id)
    }

    fn shop_entity_at(&self, pos: Position) -> Option<EntityId> {
        self.entities
            .iter()
            .find(|e| e.position == pos && e.shop.is_some())
            .map(|e| e.id)
    }

    fn door_at(&self, pos: Position) -> Option<EntityId> {
        self.entities
            .iter()
            .find(|e| e.position == pos && e.door.is_some() && !e.door.as_ref().unwrap().open)
            .map(|e| e.id)
    }

    fn is_player_dead(&self) -> bool {
        self.get_entity(self.player_id)
            .and_then(|e| e.health.as_ref())
            .map(|h| h.is_dead())
            .unwrap_or(true)
    }

    fn player_has_item(&self, name: &str) -> bool {
        self.get_entity(self.player_id)
            .and_then(|e| e.inventory.as_ref())
            .map(|inv| inv.items.iter().any(|i| i.name == name))
            .unwrap_or(false)
    }

    fn remove_player_item(&mut self, name: &str) {
        if let Some(player) = self.get_entity_mut(self.player_id) {
            if let Some(ref mut inv) = player.inventory {
                if let Some(idx) = inv.items.iter().position(|i| i.name == name) {
                    inv.items.remove(idx);
                }
            }
        }
    }

    fn try_ranged_attack(&mut self, target_id: EntityId) -> Vec<GameEvent> {
        let mut events = Vec::new();

        let player = match self.get_entity(self.player_id) {
            Some(p) => p.clone(),
            None => return events,
        };
        let target = match self.get_entity(target_id) {
            Some(t) => t.clone(),
            None => {
                self.push_message("Invalid target.", LogSeverity::Warning);
                return events;
            }
        };

        let player_pos = player.position;
        let target_pos = target.position;

        // Check if using throwing knives (no bow required)
        let throwing_knife = player.inventory.as_ref().and_then(|inv| {
            inv.items.iter().find(|i| {
                i.item.as_ref().map_or(false, |p| {
                    p.ammo_type == Some(AmmoType::ThrowingKnife) && p.charges.unwrap_or(0) > 0
                })
            })
        });

        if let Some(knife) = throwing_knife {
            let knife_ranged = knife.item.as_ref().and_then(|p| p.ranged);
            let range = knife_ranged.map_or(4, |r| r.range);
            let damage_bonus = knife_ranged.map_or(3, |r| r.damage_bonus);
            let knife_id = knife.id;

            // Range check
            let dist = player_pos.chebyshev_distance(&target_pos);
            if dist > range {
                self.push_message("Target out of range.", LogSeverity::Warning);
                return events;
            }

            // LOS check
            if !pathfinding::has_line_of_sight(&self.map, player_pos, target_pos) {
                self.push_message("No line of sight.", LogSeverity::Warning);
                return events;
            }

            // Resolve attack
            let result =
                combat::resolve_ranged_attack(&player, &target, damage_bonus, &mut self.rng);

            events.push(GameEvent::ProjectileFired {
                from: player_pos,
                to: target_pos,
                hit: true,
            });

            // Apply damage
            if let Some(target_entity) = self.get_entity_mut(target_id) {
                if let Some(ref mut health) = target_entity.health {
                    health.current -= result.damage;
                }
            }

            self.last_damage_source = None; // Player is attacking, not being attacked

            let msg = if result.is_crit {
                format!(
                    "Throwing knife critically hits {} for {} damage!",
                    target.name, result.damage
                )
            } else {
                format!(
                    "Throwing knife hits {} for {} damage.",
                    target.name, result.damage
                )
            };
            self.push_message(
                &msg,
                if result.is_crit {
                    LogSeverity::Danger
                } else {
                    LogSeverity::Info
                },
            );

            events.push(GameEvent::Attacked {
                attacker_id: self.player_id,
                target_id,
                damage: result.damage,
                killed: result.killed,
                damage_type: "physical".to_string(),
                dodged: false,
            });

            if result.killed {
                events.extend(self.handle_entity_death(target_id));
            }

            // Consume one knife
            if let Some(player_entity) = self.get_entity_mut(self.player_id) {
                if let Some(ref mut inv) = player_entity.inventory {
                    if let Some(item) = inv.items.iter_mut().find(|i| i.id == knife_id) {
                        if let Some(ref mut props) = item.item {
                            if let Some(ref mut c) = props.charges {
                                *c = c.saturating_sub(1);
                                if *c == 0 {
                                    // Remove depleted stack
                                    let remove_id = item.id;
                                    inv.items.retain(|i| i.id != remove_id);
                                }
                            }
                        }
                    }
                }
            }

            return events;
        }

        // Need a ranged weapon equipped
        let ranged_stats = combat::equipped_ranged_stats(&player);
        let required_ammo = combat::equipped_ammo_type(&player);

        let (range, damage_bonus) = match ranged_stats {
            Some(rs) => (rs.range, rs.damage_bonus),
            None => {
                self.push_message("No ranged weapon equipped.", LogSeverity::Warning);
                return events;
            }
        };

        // Range check
        let dist = player_pos.chebyshev_distance(&target_pos);
        if dist > range {
            self.push_message("Target out of range.", LogSeverity::Warning);
            return events;
        }

        // LOS check
        if !pathfinding::has_line_of_sight(&self.map, player_pos, target_pos) {
            self.push_message("No line of sight.", LogSeverity::Warning);
            return events;
        }

        // Ammo check
        let ammo_id = if let Some(ammo_type) = required_ammo {
            let ammo = player.inventory.as_ref().and_then(|inv| {
                inv.items.iter().find(|i| {
                    i.item.as_ref().map_or(false, |p| {
                        p.item_type == ItemType::Projectile
                            && p.ammo_type == Some(ammo_type)
                            && p.charges.unwrap_or(0) > 0
                    })
                })
            });
            match ammo {
                Some(a) => Some(a.id),
                None => {
                    let ammo_name = match ammo_type {
                        AmmoType::Arrow => "arrows",
                        AmmoType::Bolt => "bolts",
                        AmmoType::ThrowingKnife => "throwing knives",
                    };
                    self.push_message(
                        &format!("No {} remaining.", ammo_name),
                        LogSeverity::Warning,
                    );
                    return events;
                }
            }
        } else {
            None
        };

        // Resolve attack
        let result = combat::resolve_ranged_attack(&player, &target, damage_bonus, &mut self.rng);

        events.push(GameEvent::ProjectileFired {
            from: player_pos,
            to: target_pos,
            hit: true,
        });

        // Apply damage
        if let Some(target_entity) = self.get_entity_mut(target_id) {
            if let Some(ref mut health) = target_entity.health {
                health.current -= result.damage;
            }
        }

        let weapon_name = player
            .inventory
            .as_ref()
            .and_then(|inv| {
                player
                    .equipment
                    .as_ref()
                    .and_then(|eq| eq.main_hand)
                    .and_then(|wid| inv.items.iter().find(|i| i.id == wid))
            })
            .map(|i| i.name.clone())
            .unwrap_or_else(|| "ranged weapon".to_string());

        let msg = if result.is_crit {
            format!(
                "{} critically hits {} for {} damage!",
                weapon_name, target.name, result.damage
            )
        } else {
            format!(
                "{} hits {} for {} damage.",
                weapon_name, target.name, result.damage
            )
        };
        self.push_message(
            &msg,
            if result.is_crit {
                LogSeverity::Danger
            } else {
                LogSeverity::Info
            },
        );

        events.push(GameEvent::Attacked {
            attacker_id: self.player_id,
            target_id,
            damage: result.damage,
            killed: result.killed,
            damage_type: "physical".to_string(),
            dodged: false,
        });

        if result.killed {
            events.extend(self.handle_entity_death(target_id));
        } else {
            // Activate passive enemies when attacked
            if let Some(target_entity) = self.get_entity_mut(target_id) {
                ai::activate_passive(target_entity);
            }
        }

        // Consume ammo
        if let Some(aid) = ammo_id {
            if let Some(player_entity) = self.get_entity_mut(self.player_id) {
                if let Some(ref mut inv) = player_entity.inventory {
                    let mut remove_id = None;
                    if let Some(item) = inv.items.iter_mut().find(|i| i.id == aid) {
                        if let Some(ref mut props) = item.item {
                            if let Some(ref mut c) = props.charges {
                                *c = c.saturating_sub(1);
                                if *c == 0 {
                                    remove_id = Some(item.id);
                                }
                            }
                        }
                    }
                    if let Some(rid) = remove_id {
                        inv.items.retain(|i| i.id != rid);
                    }
                }
            }
        }

        events
    }

    fn try_buy_item(&mut self, shop_id: u32, index: usize) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // Get shop data
        let shop_entity = self.get_entity(shop_id).cloned();
        let shop = match shop_entity.as_ref().and_then(|e| e.shop.as_ref()) {
            Some(s) => s.clone(),
            None => {
                self.push_message("That's not a shop.", LogSeverity::Warning);
                return events;
            }
        };

        let shop_item = match shop.items.get(index) {
            Some(item) => item.clone(),
            None => {
                self.push_message("Invalid item.", LogSeverity::Warning);
                return events;
            }
        };

        // Check gold
        if self.gold < shop_item.price {
            self.push_message(
                &format!(
                    "Not enough gold! Need {} but have {}.",
                    shop_item.price, self.gold
                ),
                LogSeverity::Warning,
            );
            return events;
        }

        // Check inventory space
        let inv_full = self
            .get_entity(self.player_id)
            .and_then(|p| p.inventory.as_ref())
            .map(|inv| inv.items.len() >= inv.max_size as usize)
            .unwrap_or(true);
        if inv_full {
            self.push_message("Your inventory is full!", LogSeverity::Warning);
            return events;
        }

        // Create item from template
        let all_items = crate::engine::items::all_items();
        let template = match all_items.iter().find(|t| t.name == shop_item.name) {
            Some(t) => t,
            None => return events,
        };

        let item_entity = Entity {
            id: placement::next_id(),
            name: template.name.to_string(),
            position: Position::new(0, 0),
            glyph: template.glyph,
            render_order: RenderOrder::Item,
            blocks_movement: false,
            blocks_fov: false,
            health: None,
            combat: None,
            ai: None,
            inventory: None,
            equipment: None,
            item: Some(ItemProperties {
                item_type: template.item_type,
                slot: template.slot,
                power: template.power,
                speed_mod: template.speed_mod,
                effect: template.effect.clone(),
                charges: template.charges,
                energy_cost: template.energy_cost,
                ammo_type: template.ammo_type,
                ranged: template.ranged,
                hunger_restore: 0,
                enchant_level: 0,
                identified: true,
            }),
            status_effects: Vec::new(),
            fov: None,
            door: None,
            trap: None,
            stair: None,
            loot_table: None,
            flavor_text: None,
            shop: None,
            interactive: None,
            elite: None,
            resurrection_timer: None,
        };

        // Deduct gold and add item
        self.gold -= shop_item.price;
        if let Some(player) = self.get_entity_mut(self.player_id) {
            if let Some(ref mut inv) = player.inventory {
                inv.items.push(item_entity);
            }
        }

        // Remove item from shop
        if let Some(shop_e) = self.get_entity_mut(shop_id) {
            if let Some(ref mut s) = shop_e.shop {
                if index < s.items.len() {
                    s.items.remove(index);
                }
            }
        }

        self.push_message(
            &format!("Bought {} for {} gold.", shop_item.name, shop_item.price),
            LogSeverity::Good,
        );
        events.push(GameEvent::ItemBought {
            name: shop_item.name,
            price: shop_item.price,
        });

        events
    }

    fn try_sell_item(&mut self, item_index: usize, shop_id: u32) -> Vec<GameEvent> {
        let mut events = Vec::new();

        // Validate shop exists
        if self
            .get_entity(shop_id)
            .and_then(|e| e.shop.as_ref())
            .is_none()
        {
            self.push_message("That's not a shop.", LogSeverity::Warning);
            return events;
        }

        // Get item from player inventory
        let item_info = self
            .get_entity(self.player_id)
            .and_then(|p| p.inventory.as_ref())
            .and_then(|inv| inv.items.get(item_index))
            .map(|item| (item.name.clone(), item.id));

        let (item_name, item_id) = match item_info {
            Some(info) => info,
            None => {
                self.push_message("Invalid item.", LogSeverity::Warning);
                return events;
            }
        };

        // Check item isn't equipped
        let is_equipped = self
            .get_entity(self.player_id)
            .and_then(|p| p.equipment.as_ref())
            .map(|e| {
                [e.main_hand, e.off_hand, e.head, e.body, e.ring, e.amulet]
                    .iter()
                    .any(|slot| *slot == Some(item_id))
            })
            .unwrap_or(false);
        if is_equipped {
            self.push_message("Unequip that item first.", LogSeverity::Warning);
            return events;
        }

        // Calculate sell price (50% of buy price)
        let sell_price = item_base_price(&item_name).max(1);

        // Remove item from inventory
        if let Some(player) = self.get_entity_mut(self.player_id) {
            if let Some(ref mut inv) = player.inventory {
                inv.items.retain(|i| i.id != item_id);
            }
        }

        self.gold += sell_price;
        self.push_message(
            &format!("Sold {} for {} gold.", item_name, sell_price),
            LogSeverity::Good,
        );
        events.push(GameEvent::ItemSold {
            name: item_name,
            price: sell_price,
        });

        events
    }

    fn try_interact(&mut self) -> Vec<GameEvent> {
        let mut events = Vec::new();
        let player_pos = self.get_entity(self.player_id).unwrap().position;

        // Find adjacent interactive entity (check all 8 directions + current tile)
        let mut interactive_id: Option<EntityId> = None;
        let check_positions: Vec<Position> = std::iter::once(player_pos)
            .chain(
                Direction::ALL
                    .iter()
                    .map(|d| player_pos.apply_direction(*d)),
            )
            .collect();

        for pos in &check_positions {
            for entity in &self.entities {
                if entity.position == *pos && entity.interactive.is_some() {
                    interactive_id = Some(entity.id);
                    break;
                }
            }
            if interactive_id.is_some() {
                break;
            }
        }

        let entity_id = match interactive_id {
            Some(id) => id,
            None => {
                self.push_message("Nothing to interact with.", LogSeverity::Info);
                return events;
            }
        };

        // Get interaction info
        let (interaction_type, uses_remaining, activated, entity_pos, contained_items) = {
            let e = self.get_entity(entity_id).unwrap();
            let inter = e.interactive.as_ref().unwrap();
            (
                inter.interaction_type,
                inter.uses_remaining,
                inter.activated,
                e.position,
                inter.contained_items.clone(),
            )
        };

        // Check uses
        if let Some(uses) = uses_remaining {
            if uses == 0 {
                self.push_message("It's been used up.", LogSeverity::Info);
                return events;
            }
        }

        match interaction_type {
            InteractionType::Barrel => {
                // Remove barrel entity
                self.entities.retain(|e| e.id != entity_id);

                // 30% drop random item
                let drop_roll: f32 = self.rng.gen();
                let mut dropped_item_name = None;
                if drop_roll < 0.30 {
                    let all_items = crate::engine::items::all_items();
                    let eligible: Vec<&crate::engine::items::ItemTemplate> = all_items
                        .iter()
                        .filter(|t| t.min_floor <= self.floor && t.item_type != ItemType::Key)
                        .collect();
                    if let Some(template) =
                        eligible.get(self.rng.gen_range(0..eligible.len().max(1)))
                    {
                        let item_entity = Entity {
                            id: placement::next_id(),
                            name: template.name.to_string(),
                            position: entity_pos,
                            glyph: template.glyph,
                            render_order: RenderOrder::Item,
                            blocks_movement: false,
                            blocks_fov: false,
                            health: None,
                            combat: None,
                            ai: None,
                            inventory: None,
                            equipment: None,
                            item: Some(ItemProperties {
                                item_type: template.item_type,
                                slot: template.slot,
                                power: template.power,
                                speed_mod: template.speed_mod,
                                effect: template.effect.clone(),
                                charges: template.charges,
                                energy_cost: template.energy_cost,
                                ammo_type: template.ammo_type,
                                ranged: template.ranged,
                                hunger_restore: 0,
                                enchant_level: 0,
                                identified: true,
                            }),
                            status_effects: Vec::new(),
                            fov: None,
                            door: None,
                            trap: None,
                            stair: None,
                            loot_table: None,
                            flavor_text: None,
                            shop: None,
                            interactive: None,
                            elite: None,
                            resurrection_timer: None,
                        };
                        dropped_item_name = Some(template.name.to_string());
                        self.entities.push(item_entity);
                    }
                } else if drop_roll < 0.50 {
                    // 20% chance explode (3 damage in radius 1)
                    self.push_message("The barrel explodes!", LogSeverity::Danger);
                    let damage = 3;
                    // Damage entities in radius 1 of barrel position
                    let nearby: Vec<EntityId> = self
                        .entities
                        .iter()
                        .filter(|e| {
                            e.health.is_some() && e.position.chebyshev_distance(&entity_pos) <= 1
                        })
                        .map(|e| e.id)
                        .collect();
                    for target_id in nearby {
                        if let Some(target) = self.get_entity_mut(target_id) {
                            if let Some(ref mut h) = target.health {
                                h.current -= damage;
                            }
                        }
                        events.push(GameEvent::DamageTaken {
                            entity_id: target_id,
                            amount: damage,
                            source: "exploding barrel".to_string(),
                        });
                        if target_id == self.player_id {
                            self.last_damage_source = Some("an exploding barrel".to_string());
                        }
                    }
                }

                let msg = match &dropped_item_name {
                    Some(name) => format!("You smash the barrel. A {} falls out!", name),
                    None => "You smash the barrel.".to_string(),
                };
                self.push_message(&msg, LogSeverity::Info);
                events.push(GameEvent::BarrelSmashed {
                    position: entity_pos,
                    dropped_item: dropped_item_name,
                });
            }

            InteractionType::Lever => {
                // Toggle activated state
                let new_state = !activated;
                if let Some(e) = self.get_entity_mut(entity_id) {
                    if let Some(ref mut inter) = e.interactive {
                        inter.activated = new_state;
                    }
                }

                // Toggle doors in the same room
                let mut toggled = 0;
                let door_ids: Vec<EntityId> = self
                    .entities
                    .iter()
                    .filter(|e| {
                        e.door.is_some() && e.position.chebyshev_distance(&entity_pos) <= 10
                    })
                    .map(|e| e.id)
                    .collect();
                for door_id in &door_ids {
                    if let Some(door_e) = self.get_entity_mut(*door_id) {
                        if let Some(ref mut door) = door_e.door {
                            door.open = !door.open;
                            // Update blocking
                            door_e.blocks_movement = !door.open;
                            door_e.blocks_fov = !door.open;
                            toggled += 1;
                        }
                    }
                }

                let msg = if new_state {
                    format!("You pull the lever. {} door(s) open.", toggled)
                } else {
                    format!("You pull the lever back. {} door(s) close.", toggled)
                };
                self.push_message(&msg, LogSeverity::Info);
                events.push(GameEvent::LeverPulled {
                    position: entity_pos,
                });
            }

            InteractionType::Fountain => {
                // One use — random effect
                if let Some(e) = self.get_entity_mut(entity_id) {
                    if let Some(ref mut inter) = e.interactive {
                        inter.uses_remaining = Some(0);
                    }
                }

                let roll: f32 = self.rng.gen();
                let effect_name;
                if roll < 0.40 {
                    // Heal 20 HP
                    if let Some(player) = self.get_entity_mut(self.player_id) {
                        if let Some(ref mut h) = player.health {
                            h.current = (h.current + 20).min(h.max);
                        }
                    }
                    effect_name = "healing".to_string();
                    self.push_message(
                        "The fountain's water heals your wounds. (+20 HP)",
                        LogSeverity::Good,
                    );
                    events.push(GameEvent::Healed {
                        entity_id: self.player_id,
                        amount: 20,
                    });
                } else if roll < 0.60 {
                    // Cure all status effects
                    if let Some(player) = self.get_entity_mut(self.player_id) {
                        player.status_effects.clear();
                    }
                    effect_name = "purification".to_string();
                    self.push_message(
                        "The fountain's water purifies your body. Status effects cleared.",
                        LogSeverity::Good,
                    );
                } else if roll < 0.80 {
                    // +1 permanent random stat
                    let stat_roll = self.rng.gen_range(0..3);
                    if let Some(player) = self.get_entity_mut(self.player_id) {
                        if let Some(ref mut c) = player.combat {
                            match stat_roll {
                                0 => {
                                    c.base_attack += 1;
                                    effect_name = "strength".to_string();
                                    self.push_message(
                                        "The fountain's water empowers you. (+1 Attack)",
                                        LogSeverity::Good,
                                    );
                                }
                                1 => {
                                    c.base_defense += 1;
                                    effect_name = "resilience".to_string();
                                    self.push_message(
                                        "The fountain's water toughens your skin. (+1 Defense)",
                                        LogSeverity::Good,
                                    );
                                }
                                _ => {
                                    c.base_speed += 10;
                                    effect_name = "swiftness".to_string();
                                    self.push_message(
                                        "The fountain's water quickens your step. (+10 Speed)",
                                        LogSeverity::Good,
                                    );
                                }
                            }
                        } else {
                            effect_name = "nothing".to_string();
                        }
                    } else {
                        effect_name = "nothing".to_string();
                    }
                } else {
                    // Poison
                    if let Some(player) = self.get_entity_mut(self.player_id) {
                        player.status_effects.push(StatusEffect {
                            effect_type: StatusType::Poison,
                            duration: 5,
                            magnitude: 2,
                            source: "cursed fountain".to_string(),
                        });
                    }
                    effect_name = "poison".to_string();
                    self.push_message(
                        "The fountain's water burns! You are poisoned!",
                        LogSeverity::Danger,
                    );
                    events.push(GameEvent::StatusApplied {
                        entity_id: self.player_id,
                        effect: StatusType::Poison,
                        duration: 5,
                    });
                }

                events.push(GameEvent::FountainUsed {
                    position: entity_pos,
                    effect: effect_name,
                });
            }

            InteractionType::Chest => {
                // Open chest — give contained items, 25% trapped
                if let Some(e) = self.get_entity_mut(entity_id) {
                    if let Some(ref mut inter) = e.interactive {
                        inter.uses_remaining = Some(0);
                        inter.activated = true;
                    }
                    // Change glyph to open chest
                    e.glyph = 0x5F; // _ for open chest
                }

                let trapped = self.rng.gen::<f32>() < 0.25;
                let mut item_names = Vec::new();

                // Spawn contained items at chest position
                let all_items = crate::engine::items::all_items();
                for item_name in &contained_items {
                    if let Some(template) = all_items.iter().find(|t| t.name == item_name) {
                        let item_entity = Entity {
                            id: placement::next_id(),
                            name: template.name.to_string(),
                            position: entity_pos,
                            glyph: template.glyph,
                            render_order: RenderOrder::Item,
                            blocks_movement: false,
                            blocks_fov: false,
                            health: None,
                            combat: None,
                            ai: None,
                            inventory: None,
                            equipment: None,
                            item: Some(ItemProperties {
                                item_type: template.item_type,
                                slot: template.slot,
                                power: template.power,
                                speed_mod: template.speed_mod,
                                effect: template.effect.clone(),
                                charges: template.charges,
                                energy_cost: template.energy_cost,
                                ammo_type: template.ammo_type,
                                ranged: template.ranged,
                                hunger_restore: 0,
                                enchant_level: 0,
                                identified: true,
                            }),
                            status_effects: Vec::new(),
                            fov: None,
                            door: None,
                            trap: None,
                            stair: None,
                            loot_table: None,
                            flavor_text: None,
                            shop: None,
                            interactive: None,
                            elite: None,
                            resurrection_timer: None,
                        };
                        item_names.push(template.name.to_string());
                        self.entities.push(item_entity);
                    }
                }

                if trapped {
                    // Poison gas: apply Poison to entities in radius 2
                    self.push_message(
                        "A cloud of poison gas erupts from the chest!",
                        LogSeverity::Danger,
                    );
                    let nearby: Vec<EntityId> = self
                        .entities
                        .iter()
                        .filter(|e| {
                            e.health.is_some() && e.position.chebyshev_distance(&entity_pos) <= 2
                        })
                        .map(|e| e.id)
                        .collect();
                    for target_id in nearby {
                        if let Some(target) = self.get_entity_mut(target_id) {
                            target.status_effects.push(StatusEffect {
                                effect_type: StatusType::Poison,
                                duration: 4,
                                magnitude: 2,
                                source: "poison gas trap".to_string(),
                            });
                        }
                        events.push(GameEvent::StatusApplied {
                            entity_id: target_id,
                            effect: StatusType::Poison,
                            duration: 4,
                        });
                    }
                }

                let item_list = if item_names.is_empty() {
                    "nothing".to_string()
                } else {
                    item_names.join(", ")
                };
                self.push_message(
                    &format!("You open the chest and find: {}.", item_list),
                    LogSeverity::Good,
                );
                events.push(GameEvent::ChestOpened {
                    position: entity_pos,
                    items: item_names,
                    trapped,
                });
            }

            InteractionType::Altar => {
                // Consume first non-equipped item, grant +1 random stat
                let item_info = {
                    let player = self.get_entity(self.player_id);
                    let equipped_ids: Vec<EntityId> = player
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
                            .filter_map(|s| *s)
                            .collect()
                        })
                        .unwrap_or_default();

                    player
                        .and_then(|p| p.inventory.as_ref())
                        .and_then(|inv| inv.items.iter().find(|i| !equipped_ids.contains(&i.id)))
                        .map(|i| (i.id, i.name.clone()))
                };

                let (item_id, item_name) = match item_info {
                    Some(info) => info,
                    None => {
                        self.push_message(
                            "You have nothing to offer the altar.",
                            LogSeverity::Info,
                        );
                        return events;
                    }
                };

                // Remove item
                if let Some(player) = self.get_entity_mut(self.player_id) {
                    if let Some(ref mut inv) = player.inventory {
                        inv.items.retain(|i| i.id != item_id);
                    }
                }

                // Grant +1 random stat
                let stat_roll = self.rng.gen_range(0..4);
                let stat_name;
                if let Some(player) = self.get_entity_mut(self.player_id) {
                    match stat_roll {
                        0 => {
                            if let Some(ref mut h) = player.health {
                                h.max += 5;
                                h.current += 5;
                            }
                            stat_name = "Max HP +5".to_string();
                        }
                        1 => {
                            if let Some(ref mut c) = player.combat {
                                c.base_attack += 1;
                            }
                            stat_name = "Attack +1".to_string();
                        }
                        2 => {
                            if let Some(ref mut c) = player.combat {
                                c.base_defense += 1;
                            }
                            stat_name = "Defense +1".to_string();
                        }
                        _ => {
                            if let Some(ref mut c) = player.combat {
                                c.base_speed += 10;
                            }
                            stat_name = "Speed +10".to_string();
                        }
                    }
                } else {
                    stat_name = "nothing".to_string();
                }

                // Mark altar as used
                if let Some(e) = self.get_entity_mut(entity_id) {
                    if let Some(ref mut inter) = e.interactive {
                        inter.uses_remaining = Some(0);
                    }
                }

                self.push_message(
                    &format!(
                        "You offer the {} to the altar. You feel empowered! ({})",
                        item_name, stat_name
                    ),
                    LogSeverity::Good,
                );
                events.push(GameEvent::AltarOffering {
                    item_name,
                    stat_gained: stat_name,
                });
            }

            InteractionType::Anvil => {
                self.push_message("An anvil for enchanting weapons. Use the Craft action with a weapon and scroll.", LogSeverity::Info);
            }
        }

        events
    }

    fn auto_explore(&mut self) -> Vec<GameEvent> {
        use std::collections::VecDeque;
        let mut events = Vec::new();
        let player_pos = self.get_entity(self.player_id).unwrap().position;

        // BFS from player to find nearest unrevealed tile adjacent to a revealed walkable tile
        let mut visited = vec![false; self.map.width * self.map.height];
        let mut queue = VecDeque::new();
        let mut came_from: Vec<Option<Position>> = vec![None; self.map.width * self.map.height];

        let start_idx = self.map.idx(player_pos.x, player_pos.y);
        visited[start_idx] = true;
        queue.push_back(player_pos);

        let mut target: Option<Position> = None;

        while let Some(pos) = queue.pop_front() {
            // Check if this tile is adjacent to an unrevealed tile
            let idx = self.map.idx(pos.x, pos.y);
            if self.map.revealed[idx] && self.map.tiles[idx].is_walkable() {
                for (dx, dy) in &[(-1i32, 0), (1, 0), (0, -1), (0, 1)] {
                    let nx = pos.x + dx;
                    let ny = pos.y + dy;
                    if self.map.in_bounds(nx, ny) {
                        let nidx = self.map.idx(nx, ny);
                        if !self.map.revealed[nidx] {
                            target = Some(pos);
                            break;
                        }
                    }
                }
                if target.is_some() {
                    break;
                }
            }

            for (dx, dy) in &[
                (-1i32, 0),
                (1, 0),
                (0, -1),
                (0, 1),
                (-1, -1),
                (-1, 1),
                (1, -1),
                (1, 1),
            ] {
                let nx = pos.x + dx;
                let ny = pos.y + dy;
                if !self.map.in_bounds(nx, ny) {
                    continue;
                }
                let nidx = self.map.idx(nx, ny);
                if visited[nidx] {
                    continue;
                }
                if !self.map.tiles[nidx].is_walkable() {
                    continue;
                }
                if !self.map.revealed[nidx] {
                    continue;
                }
                visited[nidx] = true;
                came_from[nidx] = Some(pos);
                queue.push_back(Position::new(nx, ny));
            }
        }

        match target {
            None => {
                self.push_message("No more reachable unexplored area.", LogSeverity::Info);
            }
            Some(goal) => {
                // Reconstruct path from player to goal
                let mut path = Vec::new();
                let mut current = goal;
                while current != player_pos {
                    path.push(current);
                    let idx = self.map.idx(current.x, current.y);
                    match came_from[idx] {
                        Some(prev) => current = prev,
                        None => break,
                    }
                }
                path.reverse();

                if let Some(&first_step) = path.first() {
                    if let Some(target_id) = self.hostile_entity_at(first_step) {
                        events.extend(self.perform_attack(self.player_id, target_id));
                    } else if self.can_move_to(first_step) {
                        let from = player_pos;
                        if let Some(door_id) = self.door_at(first_step) {
                            events.extend(self.try_open_door(door_id));
                        } else {
                            self.move_entity(self.player_id, first_step);
                            events.push(GameEvent::Moved {
                                entity_id: self.player_id,
                                from,
                                to: first_step,
                            });
                            let trap_events = self.check_traps(self.player_id, first_step);
                            events.extend(trap_events);
                        }
                    }
                }
            }
        }

        events
    }

    fn check_auto_explore_interrupt(&self, events: &[GameEvent]) -> Option<AutoExploreInterrupt> {
        // Check for enemies in FOV
        let visible = self
            .get_entity(self.player_id)
            .and_then(|e| e.fov.as_ref())
            .map(|f| &f.visible_tiles);

        if let Some(vis) = visible {
            let has_enemy = self
                .entities
                .iter()
                .any(|e| e.ai.is_some() && e.health.is_some() && vis.contains(&e.position));
            if has_enemy {
                return Some(AutoExploreInterrupt::EnemySpotted);
            }
        }

        // Check if player took damage
        for event in events {
            if let GameEvent::DamageTaken { entity_id, .. } = event {
                if *entity_id == self.player_id {
                    return Some(AutoExploreInterrupt::TookDamage);
                }
            }
        }

        // Check for items at player position
        let player_pos = self.get_entity(self.player_id).unwrap().position;
        let has_item = self
            .entities
            .iter()
            .any(|e| e.position == player_pos && e.item.is_some() && e.id != self.player_id);
        if has_item {
            return Some(AutoExploreInterrupt::ItemFound);
        }

        // Check for stairs at player position
        let on_stairs = self
            .entities
            .iter()
            .any(|e| e.position == player_pos && e.stair == Some(StairDirection::Down));
        if on_stairs {
            return Some(AutoExploreInterrupt::StairsReached);
        }

        None
    }

    fn recompute_fov(&mut self) {
        let entity_ids: Vec<(EntityId, Position, i32)> = self
            .entities
            .iter()
            .filter(|e| e.fov.as_ref().map(|f| f.dirty).unwrap_or(false))
            .map(|e| (e.id, e.position, e.fov.as_ref().unwrap().radius))
            .collect();

        let abyss_reduction = if Biome::for_floor(self.floor) == Biome::Abyss {
            3
        } else {
            0
        };

        for (id, pos, radius) in entity_ids {
            // Abyss biome reduces player FOV by 3
            let effective_radius = if id == self.player_id {
                (radius - abyss_reduction).max(2)
            } else {
                radius
            };
            let visible = fov::compute_fov(pos, effective_radius, &self.map);

            // If player, also reveal tiles (and grant Pacifist XP for new tiles)
            if id == self.player_id {
                let mut new_tiles = 0u32;
                for p in &visible {
                    let idx = self.map.idx(p.x, p.y);
                    if !self.map.revealed[idx] {
                        new_tiles += 1;
                    }
                    self.map.reveal(p.x, p.y);
                }
                // Pacifist modifier: 2 XP per newly revealed tile
                if self.modifiers.contains(&RunModifier::Pacifist) && new_tiles > 0 {
                    self.player_xp += new_tiles * 2;
                }
            }

            if let Some(entity) = self.get_entity_mut(id) {
                if let Some(ref mut f) = entity.fov {
                    f.visible_tiles = visible;
                    f.dirty = false;
                }
            }
        }
    }

    fn recompute_dijkstra(&mut self) {
        let player_pos = match self.get_entity(self.player_id) {
            Some(p) => p.position,
            None => return,
        };
        self.dijkstra = Some(DijkstraMap::compute(&self.map, &[player_pos]));
    }

    fn handle_player_death(&mut self, mut events: Vec<GameEvent>) -> TurnResult {
        self.game_over = true;

        let cause = self
            .last_damage_source
            .clone()
            .unwrap_or_else(|| "Slain in the dungeon".to_string());
        events.push(GameEvent::PlayerDied {
            cause: cause.clone(),
        });
        self.push_message("You have been slain!", LogSeverity::Danger);

        let mut result = self.build_turn_result(events);
        result.game_over = Some(GameOverInfo {
            cause_of_death: cause.clone(),
            epitaph: None,
            final_score: self.calculate_score(),
            run_summary: RunSummary {
                seed: format!("{}", self.seed),
                floor_reached: self.floor,
                enemies_killed: self.enemies_killed,
                bosses_killed: self.bosses_killed,
                level_reached: self.player_level,
                turns_taken: self.turn,
                score: self.calculate_score(),
                cause_of_death: Some(cause),
                victory: false,
                timestamp: String::new(),
                class: format!("{:?}", self.player_class),
                modifiers: self.modifiers.iter().map(|m| format!("{:?}", m)).collect(),
            },
        });
        result
    }

    fn calculate_score(&self) -> u32 {
        let floor_score = self.floor * 100;
        let kill_score = self.enemies_killed * 10;
        let boss_score = self.bosses_killed * 500;
        let level_score = self.player_level * 50;
        let victory_bonus = if self.victory { 5000 } else { 0 };
        let base_score = floor_score + kill_score + boss_score + level_score + victory_bonus;

        // Apply run modifier score multipliers
        let mut multiplier: f32 = 1.0;
        for modifier in &self.modifiers {
            multiplier *= match modifier {
                RunModifier::GlassCannon => 1.5,
                RunModifier::Marathon => 2.0,
                RunModifier::Pacifist => 2.5,
                RunModifier::Cursed => 1.3,
            };
        }
        (base_score as f32 * multiplier) as u32
    }

    pub fn build_turn_result(&self, events: Vec<GameEvent>) -> TurnResult {
        let player = self.get_entity(self.player_id);
        let player_fov = player
            .and_then(|e| e.fov.as_ref())
            .map(|f| &f.visible_tiles);

        // Build visible tiles
        let mut visible_tiles = Vec::new();
        for y in 0..self.map.height as i32 {
            for x in 0..self.map.width as i32 {
                let idx = self.map.idx(x, y);
                let is_visible = player_fov
                    .map(|fov| fov.contains(&Position::new(x, y)))
                    .unwrap_or(false);
                let is_explored = self.map.revealed[idx];

                if is_visible || is_explored {
                    visible_tiles.push(VisibleTile {
                        x,
                        y,
                        tile_type: self.map.tiles[idx].as_str().to_string(),
                        explored: is_explored,
                        visible: is_visible,
                    });
                }
            }
        }

        // Build visible entities
        let visible_entities: Vec<EntityView> = self
            .entities
            .iter()
            .filter(|e| {
                e.id == self.player_id
                    || player_fov
                        .map(|fov| fov.contains(&e.position))
                        .unwrap_or(false)
            })
            .map(|e| entity_to_view(e))
            .collect();

        // Build player state
        let player_state = self.build_player_state();

        // Build minimap
        let minimap = self.build_minimap();

        // Recent messages (last 50)
        let messages: Vec<LogMessage> = self.messages.iter().rev().take(50).cloned().collect();

        let game_over = if self.victory && self.game_over {
            Some(GameOverInfo {
                cause_of_death: "Victory!".to_string(),
                epitaph: None,
                final_score: self.calculate_score(),
                run_summary: RunSummary {
                    seed: format!("{}", self.seed),
                    floor_reached: self.floor,
                    enemies_killed: self.enemies_killed,
                    bosses_killed: self.bosses_killed,
                    level_reached: self.player_level,
                    turns_taken: self.turn,
                    score: self.calculate_score(),
                    cause_of_death: None,
                    victory: true,
                    timestamp: String::new(),
                    class: format!("{:?}", self.player_class),
                    modifiers: self.modifiers.iter().map(|m| format!("{:?}", m)).collect(),
                },
            })
        } else {
            None
        };

        TurnResult {
            state: GameState {
                player: player_state,
                visible_tiles,
                visible_entities,
                floor: self.floor,
                turn: self.turn,
                messages,
                minimap,
                pending_level_up: self.pending_level_up,
                biome: Biome::for_floor(self.floor),
                seed: self.seed,
                level_up_choices: super::classes::get_level_up_choices(self.player_class),
            },
            events,
            game_over,
            auto_explore_interrupt: None,
        }
    }

    fn build_player_state(&self) -> PlayerState {
        let player = self.get_entity(self.player_id);

        let (hp, max_hp) = player
            .and_then(|p| p.health.as_ref())
            .map(|h| (h.current, h.max))
            .unwrap_or((0, 0));

        let (attack, defense, speed) = player
            .map(|p| {
                (
                    combat::effective_attack(p),
                    combat::effective_defense(p),
                    combat::effective_speed(p),
                )
            })
            .unwrap_or((0, 0, 100));

        let inventory = player
            .and_then(|p| p.inventory.as_ref())
            .map(|inv| inv.items.iter().map(|i| entity_to_item_view(i)).collect())
            .unwrap_or_default();

        let equipment = self.build_equipment_view();

        let status_effects = player
            .map(|p| {
                p.status_effects
                    .iter()
                    .map(|s| StatusView {
                        effect_type: s.effect_type,
                        duration: s.duration,
                        magnitude: s.magnitude,
                    })
                    .collect()
            })
            .unwrap_or_default();

        let xp_to_next = self.player_level * 150;

        PlayerState {
            position: player.map(|p| p.position).unwrap_or(Position::new(0, 0)),
            hp,
            max_hp,
            attack,
            defense,
            speed,
            level: self.player_level,
            xp: self.player_xp,
            xp_to_next,
            gold: self.gold,
            inventory,
            equipment,
            status_effects,
            player_class: self.player_class,
            mana: self.mana,
            max_mana: self.max_mana,
            abilities: super::abilities::to_ability_views(self.player_class),
            hunger: self.hunger,
            max_hunger: self.max_hunger,
        }
    }

    fn build_equipment_view(&self) -> EquipmentView {
        let player = self.get_entity(self.player_id);
        let equip = player.and_then(|p| p.equipment.as_ref());
        let inv = player.and_then(|p| p.inventory.as_ref());

        let get_item_view = |slot_id: Option<EntityId>| -> Option<ItemView> {
            let id = slot_id?;
            let item = inv?.items.iter().find(|i| i.id == id)?;
            Some(entity_to_item_view(item))
        };

        match equip {
            Some(e) => EquipmentView {
                main_hand: get_item_view(e.main_hand),
                off_hand: get_item_view(e.off_hand),
                head: get_item_view(e.head),
                body: get_item_view(e.body),
                ring: get_item_view(e.ring),
                amulet: get_item_view(e.amulet),
            },
            None => EquipmentView {
                main_hand: None,
                off_hand: None,
                head: None,
                body: None,
                ring: None,
                amulet: None,
            },
        }
    }

    fn build_minimap(&self) -> MinimapData {
        let mut tiles = vec![0u8; self.map.width * self.map.height];

        for y in 0..self.map.height {
            for x in 0..self.map.width {
                let idx = y * self.map.width + x;
                if !self.map.revealed[idx] {
                    tiles[idx] = 0;
                } else {
                    tiles[idx] = match self.map.tiles[idx] {
                        TileType::Wall | TileType::SecretWall => 1,
                        TileType::Floor | TileType::DoorClosed | TileType::DoorOpen => 2,
                        TileType::DownStairs | TileType::UpStairs => 3,
                    };
                }
            }
        }

        let player_pos = self
            .get_entity(self.player_id)
            .map(|p| p.position)
            .unwrap_or(Position::new(0, 0));

        MinimapData {
            width: self.map.width,
            height: self.map.height,
            tiles,
            player_x: player_pos.x,
            player_y: player_pos.y,
        }
    }
}

fn place_stairs(map: &Map, entities: &mut Vec<Entity>) {
    // Find down stairs tile on the map and create an entity for it
    for y in 0..map.height as i32 {
        for x in 0..map.width as i32 {
            if map.get_tile(x, y) == TileType::DownStairs {
                entities.push(Entity {
                    id: placement::next_id(),
                    name: "Stairs Down".to_string(),
                    position: Position::new(x, y),
                    glyph: 0x3E, // >
                    render_order: RenderOrder::Background,
                    blocks_movement: false,
                    blocks_fov: false,
                    health: None,
                    combat: None,
                    ai: None,
                    inventory: None,
                    equipment: None,
                    item: None,
                    status_effects: Vec::new(),
                    fov: None,
                    door: None,
                    trap: None,
                    stair: Some(StairDirection::Down),
                    loot_table: None,
                    flavor_text: None,
                    shop: None,
                    interactive: None,
                    elite: None,
                    resurrection_timer: None,
                });
                return;
            }
        }
    }
}

fn entity_to_view(entity: &Entity) -> EntityView {
    let is_ally = matches!(&entity.ai, Some(AIBehavior::Ally { .. }));
    let entity_type = if entity.id == 0 {
        EntityType::Player
    } else if is_ally {
        EntityType::Enemy // Allies still rendered as entities, frontend uses is_ally flag
    } else if entity.ai.is_some() {
        EntityType::Enemy
    } else if entity.item.is_some() {
        EntityType::Item
    } else if entity.door.is_some() {
        EntityType::Door
    } else if entity.trap.is_some() {
        EntityType::Trap
    } else if entity.stair.is_some() {
        EntityType::Stairs
    } else if entity.interactive.is_some() {
        EntityType::Interactive
    } else {
        EntityType::Item
    };

    EntityView {
        id: entity.id,
        name: entity.name.clone(),
        position: entity.position,
        entity_type,
        glyph: entity.glyph,
        hp: entity.health.as_ref().map(|h| (h.current, h.max)),
        flavor_text: entity.flavor_text.clone(),
        status_effects: entity
            .status_effects
            .iter()
            .map(|s| StatusView {
                effect_type: s.effect_type,
                duration: s.duration,
                magnitude: s.magnitude,
            })
            .collect(),
        elite: entity.elite.as_ref().map(|e| format!("{:?}", e)),
        is_ally,
    }
}

fn entity_to_item_view(entity: &Entity) -> ItemView {
    let item_props = entity.item.as_ref();
    let identified = item_props.map(|p| p.identified).unwrap_or(true);
    let name = if identified {
        entity.name.clone()
    } else {
        // Cursed modifier: show generic name for unidentified consumables
        match item_props.map(|p| p.item_type) {
            Some(ItemType::Potion) => "Unknown Potion".to_string(),
            Some(ItemType::Scroll) => "Unknown Scroll".to_string(),
            Some(ItemType::Food) => "Unknown Food".to_string(),
            _ => entity.name.clone(),
        }
    };
    ItemView {
        id: entity.id,
        name,
        item_type: item_props.map(|p| p.item_type).unwrap_or(ItemType::Key),
        slot: item_props.and_then(|p| p.slot),
        charges: item_props.and_then(|p| p.charges),
        identified,
    }
}

/// Base sell price for items (50% of estimated buy value).
fn item_base_price(name: &str) -> u32 {
    let all = crate::engine::items::all_items();
    let template = all.iter().find(|t| t.name == name);
    match template {
        Some(t) => {
            let base = match t.rarity {
                crate::engine::items::Rarity::Common => 5,
                crate::engine::items::Rarity::Uncommon => 12,
                crate::engine::items::Rarity::Rare => 25,
                crate::engine::items::Rarity::VeryRare => 50,
            };
            // Sell for 50% rounded up
            (base + 1) / 2
        }
        None => 1,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn world_creation() {
        let world = World::new(42);
        assert_eq!(world.floor, 1);
        assert_eq!(world.turn, 0);
        assert!(!world.entities.is_empty());
        assert!(world.get_entity(0).is_some()); // Player exists
    }

    #[test]
    fn player_can_move() {
        let mut world = World::new(42);
        let initial_pos = world.get_entity(0).unwrap().position;

        // Try all 8 directions until one works
        let mut moved = false;
        for dir in &Direction::ALL {
            let new_pos = initial_pos.apply_direction(*dir);
            if world.can_move_to(new_pos) {
                let result = world.resolve_turn(PlayerAction {
                    action_type: PlayerActionType::Move(*dir),
                });
                let final_pos = world.get_entity(0).unwrap().position;
                assert_ne!(initial_pos, final_pos);
                moved = true;
                // Check that events include a Moved event
                assert!(result
                    .events
                    .iter()
                    .any(|e| matches!(e, GameEvent::Moved { .. })));
                break;
            }
        }
        assert!(
            moved,
            "Player should be able to move in at least one direction"
        );
    }

    #[test]
    fn wait_advances_turn() {
        let mut world = World::new(42);
        assert_eq!(world.turn, 0);

        world.resolve_turn(PlayerAction {
            action_type: PlayerActionType::Wait,
        });
        assert_eq!(world.turn, 1);
    }

    #[test]
    fn bump_attack_damages_enemy() {
        let mut world = World::new(42);

        // Place an enemy adjacent to player
        let player_pos = world.get_entity(0).unwrap().position;
        let enemy_pos = Position::new(player_pos.x + 1, player_pos.y);

        let enemy = Entity {
            id: 999,
            name: "Test Enemy".to_string(),
            position: enemy_pos,
            glyph: 0x67,
            render_order: RenderOrder::Enemy,
            blocks_movement: true,
            blocks_fov: false,
            health: Some(Health::new(50)),
            combat: Some(CombatStats {
                base_attack: 3,
                base_defense: 0,
                base_speed: 100,
                crit_chance: 0.0,
                dodge_chance: 0.0,
                ranged: None,
                on_hit: None,
            }),
            ai: Some(AIBehavior::Melee),
            inventory: None,
            equipment: None,
            item: None,
            status_effects: Vec::new(),
            fov: Some(FieldOfView::new(6)),
            door: None,
            trap: None,
            stair: None,
            loot_table: None,
            flavor_text: None,
            shop: None,
            interactive: None,
            elite: None,
            resurrection_timer: None,
        };
        world.entities.push(enemy);
        world.energy.insert(999, 0);

        // Bump attack east
        let result = world.resolve_turn(PlayerAction {
            action_type: PlayerActionType::Move(Direction::E),
        });

        // Check attack event
        assert!(result.events.iter().any(|e| matches!(
            e,
            GameEvent::Attacked {
                attacker_id: 0,
                target_id: 999,
                ..
            }
        )));

        // Enemy should have less HP
        let enemy = world.get_entity(999).unwrap();
        assert!(enemy.health.as_ref().unwrap().current < 50);
    }

    #[test]
    fn enemy_death_grants_xp() {
        let mut world = World::new(42);

        let player_pos = world.get_entity(0).unwrap().position;
        let enemy_pos = Position::new(player_pos.x + 1, player_pos.y);

        // Make a weak enemy that will die in one hit
        let enemy = Entity {
            id: 999,
            name: "Weak Enemy".to_string(),
            position: enemy_pos,
            glyph: 0x67,
            render_order: RenderOrder::Enemy,
            blocks_movement: true,
            blocks_fov: false,
            health: Some(Health::new(1)),
            combat: Some(CombatStats {
                base_attack: 1,
                base_defense: 0,
                base_speed: 100,
                crit_chance: 0.0,
                dodge_chance: 0.0,
                ranged: None,
                on_hit: None,
            }),
            ai: Some(AIBehavior::Melee),
            inventory: None,
            equipment: None,
            item: None,
            status_effects: Vec::new(),
            fov: Some(FieldOfView::new(6)),
            door: None,
            trap: None,
            stair: None,
            loot_table: None,
            flavor_text: None,
            shop: None,
            interactive: None,
            elite: None,
            resurrection_timer: None,
        };
        world.entities.push(enemy);
        world.energy.insert(999, 0);

        let initial_xp = world.player_xp;

        // Kill it
        world.resolve_turn(PlayerAction {
            action_type: PlayerActionType::Move(Direction::E),
        });

        // XP should have increased (by enemy max_hp = 1)
        assert!(world.player_xp > initial_xp);

        // Enemy should be gone
        assert!(world.get_entity(999).is_none());
        assert_eq!(world.enemies_killed, 1);
    }

    #[test]
    fn score_calculation() {
        let mut world = World::new(42);
        world.floor = 5;
        world.enemies_killed = 10;
        world.bosses_killed = 1;
        world.player_level = 3;

        let score = world.calculate_score();
        // 5*100 + 10*10 + 1*500 + 3*50 = 500 + 100 + 500 + 150 = 1250
        assert_eq!(score, 1250);
    }

    #[test]
    fn seed_determinism_same_actions() {
        // Same seed + same actions = identical state
        let actions = vec![
            PlayerAction {
                action_type: PlayerActionType::Move(Direction::E),
            },
            PlayerAction {
                action_type: PlayerActionType::Move(Direction::S),
            },
            PlayerAction {
                action_type: PlayerActionType::Wait,
            },
            PlayerAction {
                action_type: PlayerActionType::Move(Direction::N),
            },
            PlayerAction {
                action_type: PlayerActionType::Move(Direction::W),
            },
        ];

        let mut world1 = World::new(12345);
        let mut world2 = World::new(12345);

        for action in &actions {
            world1.resolve_turn(action.clone());
            world2.resolve_turn(action.clone());
        }

        assert_eq!(world1.turn, world2.turn);
        assert_eq!(world1.floor, world2.floor);
        assert_eq!(world1.player_xp, world2.player_xp);
        assert_eq!(world1.enemies_killed, world2.enemies_killed);

        // Player positions should be identical
        let p1 = world1.get_entity(world1.player_id).unwrap();
        let p2 = world2.get_entity(world2.player_id).unwrap();
        assert_eq!(p1.position, p2.position);
    }

    #[test]
    fn different_seeds_produce_different_maps() {
        let world1 = World::new(1);
        let world2 = World::new(99999);

        // Maps should differ (extremely unlikely to be identical with different seeds)
        let tiles_match = world1
            .map
            .tiles
            .iter()
            .zip(world2.map.tiles.iter())
            .all(|(a, b)| a == b);
        assert!(
            !tiles_match,
            "Different seeds should produce different maps"
        );
    }

    #[test]
    fn world_serialization_round_trip() {
        let world = World::new(42);
        let json = serde_json::to_vec(&world).unwrap();
        let loaded: World = serde_json::from_slice(&json).unwrap();

        assert_eq!(loaded.seed, world.seed);
        assert_eq!(loaded.floor, world.floor);
        assert_eq!(loaded.turn, world.turn);
        assert_eq!(loaded.entities.len(), world.entities.len());
        assert_eq!(loaded.map.width, world.map.width);
        assert_eq!(loaded.map.height, world.map.height);
    }

    #[test]
    fn player_exists_after_creation() {
        let world = World::new(42);
        let player = world.get_entity(world.player_id);
        assert!(player.is_some());
        let player = player.unwrap();
        assert!(player.health.is_some());
        assert!(player.combat.is_some());
        assert!(player.fov.is_some());
        assert!(player.equipment.is_some());
        assert!(player.inventory.is_some());
    }

    #[test]
    fn enemies_spawned_on_floor() {
        let world = World::new(42);
        let enemy_count = world
            .entities
            .iter()
            .filter(|e| e.ai.is_some() && e.id != world.player_id)
            .count();
        assert!(enemy_count > 0, "Floor should have enemies");
    }

    fn place_interactable(world: &mut World, itype: InteractionType, items: Vec<String>) {
        let player_pos = world.get_entity(0).unwrap().position;
        let pos = Position::new(player_pos.x + 1, player_pos.y);
        // Ensure the tile is walkable
        let idx = world.map.idx(pos.x, pos.y);
        world.map.tiles[idx] = TileType::Floor;
        // Remove any entity at that position
        world.entities.retain(|e| e.position != pos || e.id == 0);

        let uses = match itype {
            InteractionType::Lever => None,
            _ => Some(1),
        };
        world.entities.push(Entity {
            id: 900,
            name: format!("{:?}", itype),
            position: pos,
            glyph: 0x6F,
            render_order: RenderOrder::Item,
            blocks_movement: itype == InteractionType::Barrel,
            blocks_fov: false,
            health: None,
            combat: None,
            ai: None,
            inventory: None,
            equipment: None,
            item: None,
            status_effects: Vec::new(),
            fov: None,
            door: None,
            trap: None,
            stair: None,
            loot_table: None,
            flavor_text: None,
            shop: None,
            interactive: Some(Interactive {
                interaction_type: itype,
                uses_remaining: uses,
                activated: false,
                contained_items: items,
            }),
            elite: None,
            resurrection_timer: None,
        });
    }

    #[test]
    fn interact_barrel_removes_entity() {
        let mut world = World::new(42);
        place_interactable(&mut world, InteractionType::Barrel, vec![]);
        assert!(world.get_entity(900).is_some());

        let result = world.resolve_turn(PlayerAction {
            action_type: PlayerActionType::Interact,
        });

        // Barrel should be removed
        assert!(world.get_entity(900).is_none());
        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, GameEvent::BarrelSmashed { .. })));
    }

    #[test]
    fn interact_fountain_uses_up() {
        let mut world = World::new(42);
        place_interactable(&mut world, InteractionType::Fountain, vec![]);

        let result = world.resolve_turn(PlayerAction {
            action_type: PlayerActionType::Interact,
        });

        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, GameEvent::FountainUsed { .. })));

        // Fountain should be used up
        let fountain = world.get_entity(900).unwrap();
        assert_eq!(
            fountain.interactive.as_ref().unwrap().uses_remaining,
            Some(0)
        );

        // Second use should fail
        let result2 = world.resolve_turn(PlayerAction {
            action_type: PlayerActionType::Interact,
        });
        // No FountainUsed event on second try
        assert!(!result2
            .events
            .iter()
            .any(|e| matches!(e, GameEvent::FountainUsed { .. })));
    }

    #[test]
    fn interact_chest_spawns_items() {
        let mut world = World::new(42);
        place_interactable(
            &mut world,
            InteractionType::Chest,
            vec!["Health Potion".to_string()],
        );

        let result = world.resolve_turn(PlayerAction {
            action_type: PlayerActionType::Interact,
        });

        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, GameEvent::ChestOpened { .. })));
        // Health Potion should be spawned on the map
        let potions: Vec<_> = world
            .entities
            .iter()
            .filter(|e| e.name == "Health Potion")
            .collect();
        assert!(!potions.is_empty(), "Chest should spawn contained items");
    }

    #[test]
    fn interact_altar_consumes_item_grants_stat() {
        let mut world = World::new(42);
        place_interactable(&mut world, InteractionType::Altar, vec![]);

        // Give player an item to sacrifice
        if let Some(player) = world.get_entity_mut(0) {
            if let Some(ref mut inv) = player.inventory {
                inv.items.push(Entity {
                    id: 800,
                    name: "Dagger".to_string(),
                    position: Position::new(0, 0),
                    glyph: 0x2F,
                    render_order: RenderOrder::Item,
                    blocks_movement: false,
                    blocks_fov: false,
                    health: None,
                    combat: None,
                    ai: None,
                    inventory: None,
                    equipment: None,
                    item: Some(ItemProperties {
                        item_type: ItemType::Weapon,
                        slot: Some(EquipSlot::MainHand),
                        power: 2,
                        speed_mod: 0,
                        effect: None,
                        charges: None,
                        energy_cost: 0,
                        ammo_type: None,
                        ranged: None,
                        hunger_restore: 0,
                        enchant_level: 0,
                        identified: true,
                    }),
                    status_effects: Vec::new(),
                    fov: None,
                    door: None,
                    trap: None,
                    stair: None,
                    loot_table: None,
                    flavor_text: None,
                    shop: None,
                    interactive: None,
                    elite: None,
                    resurrection_timer: None,
                });
            }
        }

        let result = world.resolve_turn(PlayerAction {
            action_type: PlayerActionType::Interact,
        });

        assert!(result
            .events
            .iter()
            .any(|e| matches!(e, GameEvent::AltarOffering { .. })));
        // Item should be consumed
        let player = world.get_entity(0).unwrap();
        let inv = player.inventory.as_ref().unwrap();
        assert!(
            inv.items.iter().all(|i| i.name != "Dagger"),
            "Altar should consume the offered item"
        );
    }

    #[test]
    fn interact_nothing_nearby() {
        let mut world = World::new(42);
        // Remove all interactables near player
        let player_pos = world.get_entity(0).unwrap().position;
        world
            .entities
            .retain(|e| e.interactive.is_none() || e.position.chebyshev_distance(&player_pos) > 1);

        let result = world.resolve_turn(PlayerAction {
            action_type: PlayerActionType::Interact,
        });

        // Should get "Nothing to interact with" message, no interaction events
        assert!(!result.events.iter().any(|e| matches!(
            e,
            GameEvent::BarrelSmashed { .. }
                | GameEvent::FountainUsed { .. }
                | GameEvent::ChestOpened { .. }
                | GameEvent::AltarOffering { .. }
                | GameEvent::LeverPulled { .. }
        )));
    }

    #[test]
    fn interactables_spawn_on_floor() {
        let world = World::new(42);
        let interactive_count = world
            .entities
            .iter()
            .filter(|e| e.interactive.is_some())
            .count();
        // Normal rooms get 0-2 barrels, so at least some should spawn
        assert!(
            interactive_count > 0,
            "Floor should have interactable entities"
        );
    }
}
