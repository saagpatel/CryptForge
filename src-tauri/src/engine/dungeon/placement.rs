use rand::Rng;
use std::collections::HashSet;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::engine::enemies::{
    all_enemies, apply_endless_scaling, boss_templates, get_boss_for_floor, get_enemy_pool,
    EnemySpecial,
};
use crate::engine::entity::*;
use crate::engine::items::all_items;
use crate::engine::map::{Map, Room, RoomType, TileType};

fn map_special_to_on_hit(special: &Option<EnemySpecial>) -> Option<OnHitEffect> {
    match special {
        Some(EnemySpecial::PoisonOnHit { damage, duration }) => Some(OnHitEffect::Poison {
            damage: *damage,
            duration: *duration,
        }),
        Some(EnemySpecial::BurningOnHit { damage, duration }) => Some(OnHitEffect::Burn {
            damage: *damage,
            duration: *duration,
        }),
        Some(EnemySpecial::SlowOnHit {
            magnitude,
            duration,
        }) => Some(OnHitEffect::Slow {
            magnitude: *magnitude,
            duration: *duration,
        }),
        Some(EnemySpecial::ConfuseOnHit { duration }) => Some(OnHitEffect::Confuse {
            duration: *duration,
        }),
        Some(EnemySpecial::LifeSteal) => Some(OnHitEffect::LifeSteal),
        Some(EnemySpecial::DrainMaxHp) => Some(OnHitEffect::DrainMaxHp),
        _ => None,
    }
}

static NEXT_ENTITY_ID: AtomicU32 = AtomicU32::new(1);

pub fn next_id() -> EntityId {
    NEXT_ENTITY_ID.fetch_add(1, Ordering::Relaxed)
}

pub fn spawn_player(pos: Position) -> Entity {
    Entity {
        id: 0, // Player always ID 0
        name: "Player".to_string(),
        position: pos,
        glyph: 0x40, // @
        render_order: RenderOrder::Player,
        blocks_movement: true,
        blocks_fov: false,
        health: Some(Health::new(50)),
        combat: Some(CombatStats {
            base_attack: 5,
            base_defense: 2,
            base_speed: 100,
            crit_chance: 0.05,
            dodge_chance: 0.0,
            ranged: None,
            on_hit: None,
        }),
        ai: None,
        inventory: Some(Inventory::new(20)),
        equipment: Some(EquipmentSlots::empty()),
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

pub fn spawn_player_with_class(
    pos: Position,
    template: &crate::engine::classes::ClassTemplate,
) -> Entity {
    Entity {
        id: 0,
        name: "Player".to_string(),
        position: pos,
        glyph: 0x40,
        render_order: RenderOrder::Player,
        blocks_movement: true,
        blocks_fov: false,
        health: Some(Health::new(template.hp)),
        combat: Some(CombatStats {
            base_attack: template.attack,
            base_defense: template.defense,
            base_speed: template.speed,
            crit_chance: template.crit_chance,
            dodge_chance: template.dodge_chance,
            ranged: None,
            on_hit: None,
        }),
        ai: None,
        inventory: Some(Inventory::new(20)),
        equipment: Some(EquipmentSlots::empty()),
        item: None,
        status_effects: Vec::new(),
        fov: Some(FieldOfView::new(template.fov_radius)),
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

pub fn spawn_entities(map: &Map, floor: u32, rng: &mut impl Rng) -> Vec<Entity> {
    let mut entities = Vec::new();
    let mut occupied: HashSet<Position> = HashSet::new();

    let enemy_pool = get_enemy_pool(floor);
    let all_enemy_templates = all_enemies();
    let all_boss_templates = boss_templates();
    let all_item_templates = all_items();
    let boss_name = get_boss_for_floor(floor);

    for room in &map.rooms {
        let positions = get_floor_positions(map, room);
        if positions.is_empty() {
            continue;
        }

        match room.room_type {
            RoomType::Start => {
                // Place a health potion in the start room on floor 1
                if floor == 1 {
                    if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                        entities.push(create_item("Health Potion", pos, &all_item_templates));
                        occupied.insert(pos);
                    }
                    if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                        entities.push(create_item("Dagger", pos, &all_item_templates));
                        occupied.insert(pos);
                    }
                }
            }
            RoomType::Boss => {
                // Boss + 1-2 minions
                if let Some(name) = boss_name {
                    if let Some(template) = all_boss_templates.iter().find(|t| t.name == name) {
                        if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                            entities.push(create_enemy_from_template(template, pos, floor, rng));
                            occupied.insert(pos);
                        }
                    }
                    // 1-2 minions
                    let minion_count = rng.gen_range(1..=2);
                    for _ in 0..minion_count {
                        if enemy_pool.is_empty() {
                            break;
                        }
                        if let Some(enemy_name) = enemy_pool.get(rng.gen_range(0..enemy_pool.len()))
                        {
                            if let Some(template) =
                                all_enemy_templates.iter().find(|t| t.name == *enemy_name)
                            {
                                if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                                    entities.push(create_enemy_from_template(
                                        template, pos, floor, rng,
                                    ));
                                    occupied.insert(pos);
                                }
                            }
                        }
                    }
                }
                // Place locked door (handled separately in map)
                // Place a lever in the boss room
                if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                    entities.push(create_interactable(InteractionType::Lever, pos, None));
                    occupied.insert(pos);
                }
            }
            RoomType::Treasure => {
                // 1 chest with 1-2 items
                if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                    let item_count = rng.gen_range(1..=2);
                    let mut chest_items = Vec::new();
                    for _ in 0..item_count {
                        if let Some(item) = pick_weighted_item(floor, rng, &all_item_templates) {
                            chest_items.push(item.name.clone());
                        }
                    }
                    entities.push(create_interactable(
                        InteractionType::Chest,
                        pos,
                        Some(chest_items),
                    ));
                    occupied.insert(pos);
                }
                // 1-3 loose items
                let item_count = rng.gen_range(1..=3);
                for _ in 0..item_count {
                    if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                        if let Some(mut item) = pick_weighted_item(floor, rng, &all_item_templates)
                        {
                            item.position = pos;
                            entities.push(item);
                            occupied.insert(pos);
                        }
                    }
                }
                // 1 enemy guarding
                if !enemy_pool.is_empty() {
                    if let Some(enemy_name) = enemy_pool.get(rng.gen_range(0..enemy_pool.len())) {
                        if let Some(template) =
                            all_enemy_templates.iter().find(|t| t.name == *enemy_name)
                        {
                            if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                                entities
                                    .push(create_enemy_from_template(template, pos, floor, rng));
                                occupied.insert(pos);
                            }
                        }
                    }
                }
            }
            RoomType::Library | RoomType::Armory => {
                // 2-3 items
                let item_count = rng.gen_range(2..=3);
                for _ in 0..item_count {
                    if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                        if let Some(mut item) = pick_weighted_item(floor, rng, &all_item_templates)
                        {
                            item.position = pos;
                            entities.push(item);
                            occupied.insert(pos);
                        }
                    }
                }
            }
            RoomType::Shrine => {
                // 1 fountain, altar, or anvil
                if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                    let roll = rng.gen::<f32>();
                    if roll < 0.33 {
                        entities.push(create_interactable(InteractionType::Fountain, pos, None));
                    } else if roll < 0.66 {
                        entities.push(create_interactable(InteractionType::Altar, pos, None));
                    } else {
                        entities.push(create_interactable(InteractionType::Anvil, pos, None));
                    }
                    occupied.insert(pos);
                }
                // 1 good item
                if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                    if let Some(mut item) = pick_weighted_item(floor, rng, &all_item_templates) {
                        item.position = pos;
                        entities.push(item);
                        occupied.insert(pos);
                    }
                }
            }
            RoomType::Shop => {
                // Shopkeeper with randomized inventory
                if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                    let shop_items = generate_shop_inventory(floor, rng, &all_item_templates);
                    entities.push(create_shopkeeper(pos, shop_items));
                    occupied.insert(pos);
                }
            }
            RoomType::Normal => {
                // Enemies: floor/2 + rng(1,3)
                let enemy_count =
                    (floor as i32 / 2 + rng.gen_range(1..=3)).min(positions.len() as i32 / 2);
                for _ in 0..enemy_count {
                    if enemy_pool.is_empty() {
                        break;
                    }
                    if let Some(enemy_name) = enemy_pool.get(rng.gen_range(0..enemy_pool.len())) {
                        if let Some(template) =
                            all_enemy_templates.iter().find(|t| t.name == *enemy_name)
                        {
                            if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                                entities
                                    .push(create_enemy_from_template(template, pos, floor, rng));
                                occupied.insert(pos);
                            }
                        }
                    }
                }
                // 30% chance of an item
                if rng.gen::<f32>() < 0.30 {
                    if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                        if let Some(mut item) = pick_weighted_item(floor, rng, &all_item_templates)
                        {
                            item.position = pos;
                            entities.push(item);
                            occupied.insert(pos);
                        }
                    }
                }
                // 0-2 barrels
                let barrel_count = rng.gen_range(0..=2);
                for _ in 0..barrel_count {
                    if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                        entities.push(create_interactable(InteractionType::Barrel, pos, None));
                        occupied.insert(pos);
                    }
                }
            }
        }
    }

    // Place 1-3 traps
    let trap_count = rng.gen_range(1..=3);
    let all_floor_positions: Vec<Position> = (0..map.width as i32)
        .flat_map(|x| (0..map.height as i32).map(move |y| Position::new(x, y)))
        .filter(|p| map.get_tile(p.x, p.y) == TileType::Floor && !occupied.contains(p))
        .collect();

    for _ in 0..trap_count {
        if let Some(pos) = pick_free_pos(&all_floor_positions, &occupied, rng) {
            let trap_type = match rng.gen_range(0..4) {
                0 => TrapType::Spike {
                    damage: 5 + floor as i32,
                },
                1 => TrapType::Poison {
                    damage: 2,
                    duration: 3,
                },
                2 => TrapType::Teleport,
                _ => TrapType::Alarm,
            };
            entities.push(Entity {
                id: next_id(),
                name: "Trap".to_string(),
                position: pos,
                glyph: 0x5E,
                render_order: RenderOrder::Trap,
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
                trap: Some(TrapProperties {
                    trap_type,
                    revealed: false,
                    triggered: false,
                }),
                stair: None,
                loot_table: None,
                flavor_text: None,
                shop: None,
                interactive: None,
                elite: None,
                resurrection_timer: None,
            });
            occupied.insert(pos);
        }
    }

    // 15% chance per non-boss floor: spawn an NPC ally (Prisoner)
    if boss_name.is_none() && rng.gen::<f32>() < 0.15 {
        let normal_rooms: Vec<&Room> = map
            .rooms
            .iter()
            .filter(|r| r.room_type == RoomType::Normal)
            .collect();
        if let Some(ally_room) = normal_rooms.get(rng.gen_range(0..normal_rooms.len().max(1))) {
            let positions = get_floor_positions(map, ally_room);
            if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                entities.push(create_ally(rng, pos));
                occupied.insert(pos);
            }
        }
    }

    // Place boss key on boss floors (in a non-boss, non-start room)
    if boss_name.is_some() {
        let key_rooms: Vec<&Room> = map
            .rooms
            .iter()
            .filter(|r| r.room_type != RoomType::Boss && r.room_type != RoomType::Start)
            .collect();
        if let Some(key_room) = key_rooms.get(rng.gen_range(0..key_rooms.len().max(1))) {
            let positions = get_floor_positions(map, key_room);
            if let Some(pos) = pick_free_pos(&positions, &occupied, rng) {
                entities.push(create_item("Boss Key", pos, &all_item_templates));
                occupied.insert(pos);
            }
        }
    }

    entities
}

fn get_floor_positions(map: &Map, room: &Room) -> Vec<Position> {
    let mut positions = Vec::new();
    for y in room.y..(room.y + room.height) {
        for x in room.x..(room.x + room.width) {
            if map.in_bounds(x, y) && map.get_tile(x, y) == TileType::Floor {
                positions.push(Position::new(x, y));
            }
        }
    }
    positions
}

fn pick_free_pos(
    positions: &[Position],
    occupied: &HashSet<Position>,
    rng: &mut impl Rng,
) -> Option<Position> {
    let free: Vec<Position> = positions
        .iter()
        .filter(|p| !occupied.contains(p))
        .cloned()
        .collect();
    if free.is_empty() {
        return None;
    }
    Some(free[rng.gen_range(0..free.len())])
}

fn create_enemy_from_template(
    template: &crate::engine::enemies::EnemyTemplate,
    pos: Position,
    floor: u32,
    rng: &mut impl Rng,
) -> Entity {
    let (hp, attack, defense) = apply_endless_scaling(template, floor);
    let is_boss = matches!(template.ai, AIBehavior::Boss(_));

    // 12% chance for non-boss enemies to become elite
    let elite_prefix = if !is_boss && rng.gen::<f32>() < 0.12 {
        Some(match rng.gen_range(0..4) {
            0 => ElitePrefix::Frenzied,
            1 => ElitePrefix::Armored,
            2 => ElitePrefix::Venomous,
            _ => ElitePrefix::Spectral,
        })
    } else {
        None
    };

    // Apply elite stat modifications
    let (hp, attack, defense, speed, crit_chance, on_hit) = match elite_prefix {
        Some(ElitePrefix::Frenzied) => (
            hp,
            (attack as f32 * 1.5) as i32,
            defense,
            template.speed + 30,
            template.crit_chance,
            map_special_to_on_hit(&template.special),
        ),
        Some(ElitePrefix::Armored) => (
            (hp as f32 * 1.5) as i32,
            attack,
            defense * 2,
            template.speed,
            template.crit_chance,
            map_special_to_on_hit(&template.special),
        ),
        Some(ElitePrefix::Venomous) => (
            hp,
            attack,
            defense,
            template.speed,
            template.crit_chance,
            Some(OnHitEffect::Poison {
                damage: 2,
                duration: 3,
            }),
        ),
        Some(ElitePrefix::Spectral) => (
            hp,
            attack,
            defense,
            template.speed,
            0.30,
            map_special_to_on_hit(&template.special),
        ),
        None => (
            hp,
            attack,
            defense,
            template.speed,
            template.crit_chance,
            map_special_to_on_hit(&template.special),
        ),
    };

    let name = match &elite_prefix {
        Some(prefix) => {
            let prefix_str = match prefix {
                ElitePrefix::Frenzied => "Frenzied",
                ElitePrefix::Armored => "Armored",
                ElitePrefix::Venomous => "Venomous",
                ElitePrefix::Spectral => "Spectral",
            };
            format!("{} {}", prefix_str, template.name)
        }
        None => template.name.to_string(),
    };

    Entity {
        id: next_id(),
        name,
        position: pos,
        glyph: template.glyph,
        render_order: RenderOrder::Enemy,
        blocks_movement: true,
        blocks_fov: false,
        health: Some(Health::new(hp)),
        combat: Some(CombatStats {
            base_attack: attack,
            base_defense: defense,
            base_speed: speed,
            crit_chance,
            dodge_chance: 0.0,
            ranged: None,
            on_hit,
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
        elite: elite_prefix,
        resurrection_timer: None,
    }
}

fn create_item(
    name: &str,
    pos: Position,
    templates: &[crate::engine::items::ItemTemplate],
) -> Entity {
    let t = templates
        .iter()
        .find(|t| t.name == name)
        .unwrap_or_else(|| panic!("Item template '{}' not found", name));
    Entity {
        id: next_id(),
        name: t.name.to_string(),
        position: pos,
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
    }
}

pub(crate) fn pick_weighted_item(
    floor: u32,
    rng: &mut impl Rng,
    templates: &[crate::engine::items::ItemTemplate],
) -> Option<Entity> {
    let eligible: Vec<&crate::engine::items::ItemTemplate> = templates
        .iter()
        .filter(|t| t.min_floor <= floor && t.item_type != ItemType::Key)
        .collect();

    if eligible.is_empty() {
        return None;
    }

    let total_weight: u32 = eligible.iter().map(|t| t.rarity.weight()).sum();
    let mut roll = rng.gen_range(0..total_weight);

    for t in &eligible {
        let w = t.rarity.weight();
        if roll < w {
            let entity = Entity {
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
            return Some(entity);
        }
        roll -= w;
    }
    None
}

fn generate_shop_inventory(
    floor: u32,
    rng: &mut impl Rng,
    templates: &[crate::engine::items::ItemTemplate],
) -> Vec<ShopItem> {
    let eligible: Vec<&crate::engine::items::ItemTemplate> = templates
        .iter()
        .filter(|t| {
            t.min_floor <= floor + 1
                && t.item_type != ItemType::Key
                && t.item_type != ItemType::Projectile
        })
        .collect();

    let count = rng.gen_range(4..=6).min(eligible.len());
    let mut items = Vec::new();
    let mut used_names: HashSet<String> = HashSet::new();

    for _ in 0..count * 3 {
        if items.len() >= count {
            break;
        }
        let idx = rng.gen_range(0..eligible.len());
        let t = eligible[idx];
        if used_names.contains(t.name) {
            continue;
        }
        let price = match t.rarity {
            crate::engine::items::Rarity::Common => 5,
            crate::engine::items::Rarity::Uncommon => 12,
            crate::engine::items::Rarity::Rare => 25,
            crate::engine::items::Rarity::VeryRare => 50,
        } + floor / 2;

        items.push(ShopItem {
            name: t.name.to_string(),
            price,
            item_type: t.item_type,
            slot: t.slot,
        });
        used_names.insert(t.name.to_string());
    }

    items
}

fn create_shopkeeper(pos: Position, shop_items: Vec<ShopItem>) -> Entity {
    Entity {
        id: next_id(),
        name: "Shopkeeper".to_string(),
        position: pos,
        glyph: 0x24, // '$' symbol
        render_order: RenderOrder::Enemy,
        blocks_movement: true,
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
        flavor_text: Some("A weathered merchant. Bump to trade.".to_string()),
        shop: Some(ShopInventory {
            items: shop_items,
            buy_multiplier: 1.0,
        }),
        interactive: None,
        elite: None,
        resurrection_timer: None,
    }
}

fn create_interactable(
    interaction_type: InteractionType,
    pos: Position,
    contained_items: Option<Vec<String>>,
) -> Entity {
    let (name, glyph, blocks_movement) = match interaction_type {
        InteractionType::Barrel => ("Barrel", 0x6F, true), // 'o'
        InteractionType::Lever => ("Lever", 0x2F, false),  // '/'
        InteractionType::Fountain => ("Fountain", 0x7E, false), // '~'
        InteractionType::Altar => ("Altar", 0x2B, false),  // '+'
        InteractionType::Chest => ("Chest", 0x3D, false),  // '='
        InteractionType::Anvil => ("Anvil", 0x26, false),  // '&'
    };

    let uses = match interaction_type {
        InteractionType::Fountain | InteractionType::Altar | InteractionType::Chest => Some(1),
        InteractionType::Barrel => Some(1),
        InteractionType::Lever | InteractionType::Anvil => None,
    };

    Entity {
        id: next_id(),
        name: name.to_string(),
        position: pos,
        glyph,
        render_order: RenderOrder::Item,
        blocks_movement,
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
            interaction_type,
            uses_remaining: uses,
            activated: false,
            contained_items: contained_items.unwrap_or_default(),
        }),
        elite: None,
        resurrection_timer: None,
    }
}

fn create_ally(rng: &mut impl Rng, pos: Position) -> Entity {
    // Three ally variants: Sellsword (melee), Healer (low atk), Scout (ranged-ish)
    let variant = rng.gen_range(0..3);
    let (name, hp, attack, defense, speed) = match variant {
        0 => ("Sellsword", 30, 5, 2, 100),
        1 => ("Healer", 20, 2, 1, 90),
        _ => ("Scout", 25, 4, 1, 110),
    };

    Entity {
        id: next_id(),
        name: name.to_string(),
        position: pos,
        glyph: 0x41,                      // 'A'
        render_order: RenderOrder::Enemy, // Same layer as actors
        blocks_movement: true,
        blocks_fov: false,
        health: Some(Health::new(hp)),
        combat: Some(CombatStats {
            base_attack: attack,
            base_defense: defense,
            base_speed: speed,
            crit_chance: 0.05,
            dodge_chance: 0.0,
            ranged: None,
            on_hit: None,
        }),
        ai: Some(AIBehavior::Ally { follow_distance: 3 }),
        inventory: None,
        equipment: None,
        item: None,
        status_effects: Vec::new(),
        fov: Some(FieldOfView::new(6)),
        door: None,
        trap: None,
        stair: None,
        loot_table: None,
        flavor_text: Some(format!(
            "A rescued {}. They will fight alongside you.",
            name.to_lowercase()
        )),
        shop: None,
        interactive: None,
        elite: None,
        resurrection_timer: None,
    }
}
