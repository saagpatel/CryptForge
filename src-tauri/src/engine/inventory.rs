use super::combat;
use super::entity::*;

/// Calculate effective stats for a player entity including equipment.
pub fn calculate_effective_stats(entity: &Entity) -> EffectiveStats {
    EffectiveStats {
        attack: combat::effective_attack(entity),
        defense: combat::effective_defense(entity),
        speed: combat::effective_speed(entity),
    }
}

pub struct EffectiveStats {
    pub attack: i32,
    pub defense: i32,
    pub speed: i32,
}

/// Try to add an item to an entity's inventory. Returns false if full.
pub fn add_to_inventory(entity: &mut Entity, item: Entity) -> bool {
    if let Some(ref mut inv) = entity.inventory {
        if inv.is_full() {
            return false;
        }
        inv.items.push(item);
        true
    } else {
        false
    }
}

/// Remove an item from inventory by ID. Returns the removed item if found.
pub fn remove_from_inventory(entity: &mut Entity, item_id: EntityId) -> Option<Entity> {
    if let Some(ref mut inv) = entity.inventory {
        if let Some(idx) = inv.items.iter().position(|i| i.id == item_id) {
            return Some(inv.items.remove(idx));
        }
    }
    None
}

/// Get item from inventory by index (read-only).
pub fn get_inventory_item(entity: &Entity, index: usize) -> Option<&Entity> {
    entity.inventory.as_ref()?.items.get(index)
}

/// Equip an item to its appropriate slot. Returns the previously equipped item ID if any.
pub fn equip_item(
    entity: &mut Entity,
    item_id: EntityId,
) -> Result<Option<EntityId>, &'static str> {
    // Find the item in inventory
    let slot = {
        let inv = entity.inventory.as_ref().ok_or("No inventory")?;
        let item = inv
            .items
            .iter()
            .find(|i| i.id == item_id)
            .ok_or("Item not in inventory")?;
        let props = item.item.as_ref().ok_or("Not an item")?;
        props.slot.ok_or("Item can't be equipped")?
    };

    let equipment = entity.equipment.as_mut().ok_or("No equipment slots")?;
    let previous = equipment.get_slot(slot);
    equipment.set_slot(slot, Some(item_id));

    Ok(previous)
}

/// Unequip item from a specific slot. Returns the item ID that was in the slot.
pub fn unequip_slot(entity: &mut Entity, slot: EquipSlot) -> Option<EntityId> {
    let equipment = entity.equipment.as_mut()?;
    let previous = equipment.get_slot(slot);
    equipment.set_slot(slot, None);
    previous
}

/// Check if an item is currently equipped.
pub fn is_equipped(entity: &Entity, item_id: EntityId) -> bool {
    entity.equipment.as_ref().map_or(false, |equip| {
        [
            equip.main_hand,
            equip.off_hand,
            equip.head,
            equip.body,
            equip.ring,
            equip.amulet,
        ]
        .iter()
        .any(|slot| *slot == Some(item_id))
    })
}

/// Get the equipped item ID for a specific slot.
pub fn get_equipped(entity: &Entity, slot: EquipSlot) -> Option<EntityId> {
    entity.equipment.as_ref()?.get_slot(slot)
}

/// Check if inventory has room.
pub fn has_inventory_space(entity: &Entity) -> bool {
    entity
        .inventory
        .as_ref()
        .map_or(false, |inv| !inv.is_full())
}

/// Find an item in inventory by name.
pub fn find_item_by_name(entity: &Entity, name: &str) -> Option<EntityId> {
    entity
        .inventory
        .as_ref()?
        .items
        .iter()
        .find(|i| i.name == name)
        .map(|i| i.id)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_player() -> Entity {
        Entity {
            id: 0,
            name: "Player".to_string(),
            position: Position::new(5, 5),
            glyph: 0x40,
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

    fn make_weapon(id: EntityId, name: &str, power: i32) -> Entity {
        Entity {
            id,
            name: name.to_string(),
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
                power,
                speed_mod: 0,
                effect: None,
                charges: None,
                energy_cost: 100,
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
        }
    }

    #[test]
    fn add_item_to_inventory() {
        let mut player = make_player();
        let sword = make_weapon(10, "Sword", 5);

        assert!(add_to_inventory(&mut player, sword));
        assert_eq!(player.inventory.as_ref().unwrap().items.len(), 1);
    }

    #[test]
    fn full_inventory_rejects() {
        let mut player = make_player();
        player.inventory.as_mut().unwrap().max_size = 2;

        assert!(add_to_inventory(&mut player, make_weapon(10, "Sword", 5)));
        assert!(add_to_inventory(&mut player, make_weapon(11, "Axe", 7)));
        assert!(!add_to_inventory(&mut player, make_weapon(12, "Mace", 4)));
    }

    #[test]
    fn equip_weapon_increases_attack() {
        let mut player = make_player();
        let sword = make_weapon(10, "Sword", 5);
        add_to_inventory(&mut player, sword);
        equip_item(&mut player, 10).unwrap();

        let stats = calculate_effective_stats(&player);
        assert_eq!(stats.attack, 10); // 5 base + 5 weapon
    }

    #[test]
    fn unequip_removes_from_slot() {
        let mut player = make_player();
        let sword = make_weapon(10, "Sword", 5);
        add_to_inventory(&mut player, sword);
        equip_item(&mut player, 10).unwrap();

        let prev = unequip_slot(&mut player, EquipSlot::MainHand);
        assert_eq!(prev, Some(10));
        assert_eq!(get_equipped(&player, EquipSlot::MainHand), None);
    }

    #[test]
    fn is_equipped_check() {
        let mut player = make_player();
        let sword = make_weapon(10, "Sword", 5);
        add_to_inventory(&mut player, sword);
        equip_item(&mut player, 10).unwrap();

        assert!(is_equipped(&player, 10));
        assert!(!is_equipped(&player, 99));
    }

    #[test]
    fn find_item_by_name_works() {
        let mut player = make_player();
        add_to_inventory(&mut player, make_weapon(10, "Sword", 5));
        add_to_inventory(&mut player, make_weapon(11, "Dagger", 2));

        assert_eq!(find_item_by_name(&player, "Dagger"), Some(11));
        assert_eq!(find_item_by_name(&player, "Axe"), None);
    }
}
