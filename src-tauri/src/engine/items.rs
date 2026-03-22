use crate::engine::entity::*;

pub struct ItemTemplate {
    pub name: &'static str,
    pub glyph: u32,
    pub item_type: ItemType,
    pub slot: Option<EquipSlot>,
    pub power: i32,
    pub speed_mod: i32,
    pub effect: Option<ItemEffect>,
    pub charges: Option<u32>,
    pub energy_cost: i32,
    pub min_floor: u32,
    pub rarity: Rarity,
    pub ammo_type: Option<AmmoType>,
    pub ranged: Option<RangedStats>,
    pub hunger_restore: i32,
}

#[derive(Debug, Clone, Copy)]
pub enum Rarity {
    Common,
    Uncommon,
    Rare,
    VeryRare,
}

impl Rarity {
    pub fn weight(&self) -> u32 {
        match self {
            Rarity::Common => 10,
            Rarity::Uncommon => 5,
            Rarity::Rare => 2,
            Rarity::VeryRare => 1,
        }
    }
}

pub fn all_items() -> Vec<ItemTemplate> {
    vec![
        // Weapons
        ItemTemplate {
            name: "Dagger",
            glyph: 0x2F,
            item_type: ItemType::Weapon,
            slot: Some(EquipSlot::MainHand),
            power: 2,
            speed_mod: 20,
            effect: None,
            charges: None,
            energy_cost: 100,
            min_floor: 1,
            rarity: Rarity::Common,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Short Sword",
            glyph: 0x2F,
            item_type: ItemType::Weapon,
            slot: Some(EquipSlot::MainHand),
            power: 4,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 100,
            min_floor: 1,
            rarity: Rarity::Common,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Mace",
            glyph: 0x2F,
            item_type: ItemType::Weapon,
            slot: Some(EquipSlot::MainHand),
            power: 5,
            speed_mod: 0,
            effect: Some(ItemEffect::ApplyStatus {
                effect: StatusType::Stunned,
                duration: 1,
            }),
            charges: None,
            energy_cost: 100,
            min_floor: 2,
            rarity: Rarity::Uncommon,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Long Sword",
            glyph: 0x2F,
            item_type: ItemType::Weapon,
            slot: Some(EquipSlot::MainHand),
            power: 7,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 100,
            min_floor: 3,
            rarity: Rarity::Uncommon,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "War Axe",
            glyph: 0x2F,
            item_type: ItemType::Weapon,
            slot: Some(EquipSlot::MainHand),
            power: 9,
            speed_mod: -10,
            effect: None,
            charges: None,
            energy_cost: 100,
            min_floor: 5,
            rarity: Rarity::Rare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Great Sword",
            glyph: 0x2F,
            item_type: ItemType::Weapon,
            slot: Some(EquipSlot::MainHand),
            power: 11,
            speed_mod: -20,
            effect: None,
            charges: None,
            energy_cost: 100,
            min_floor: 7,
            rarity: Rarity::Rare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Poison Dagger",
            glyph: 0x2F,
            item_type: ItemType::Weapon,
            slot: Some(EquipSlot::MainHand),
            power: 3,
            speed_mod: 20,
            effect: Some(ItemEffect::ApplyStatus {
                effect: StatusType::Poison,
                duration: 4,
            }),
            charges: None,
            energy_cost: 100,
            min_floor: 4,
            rarity: Rarity::Rare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Flame Blade",
            glyph: 0x2F,
            item_type: ItemType::Weapon,
            slot: Some(EquipSlot::MainHand),
            power: 8,
            speed_mod: 0,
            effect: Some(ItemEffect::ApplyStatus {
                effect: StatusType::Burning,
                duration: 3,
            }),
            charges: None,
            energy_cost: 100,
            min_floor: 6,
            rarity: Rarity::VeryRare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Frost Brand",
            glyph: 0x2F,
            item_type: ItemType::Weapon,
            slot: Some(EquipSlot::MainHand),
            power: 8,
            speed_mod: 0,
            effect: Some(ItemEffect::ApplyStatus {
                effect: StatusType::Slowed,
                duration: 3,
            }),
            charges: None,
            energy_cost: 100,
            min_floor: 6,
            rarity: Rarity::VeryRare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        // Armor - Head
        ItemTemplate {
            name: "Leather Cap",
            glyph: 0x5E,
            item_type: ItemType::Armor,
            slot: Some(EquipSlot::Head),
            power: 1,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 50,
            min_floor: 1,
            rarity: Rarity::Common,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Iron Helm",
            glyph: 0x5E,
            item_type: ItemType::Armor,
            slot: Some(EquipSlot::Head),
            power: 3,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 50,
            min_floor: 4,
            rarity: Rarity::Uncommon,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        // Armor - Body
        ItemTemplate {
            name: "Leather Armor",
            glyph: 0x5B,
            item_type: ItemType::Armor,
            slot: Some(EquipSlot::Body),
            power: 2,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 50,
            min_floor: 1,
            rarity: Rarity::Common,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Chain Mail",
            glyph: 0x5B,
            item_type: ItemType::Armor,
            slot: Some(EquipSlot::Body),
            power: 4,
            speed_mod: -10,
            effect: None,
            charges: None,
            energy_cost: 50,
            min_floor: 3,
            rarity: Rarity::Uncommon,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Plate Armor",
            glyph: 0x5B,
            item_type: ItemType::Armor,
            slot: Some(EquipSlot::Body),
            power: 7,
            speed_mod: -20,
            effect: None,
            charges: None,
            energy_cost: 50,
            min_floor: 6,
            rarity: Rarity::Rare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        // Shields
        ItemTemplate {
            name: "Wooden Shield",
            glyph: 0x29,
            item_type: ItemType::Shield,
            slot: Some(EquipSlot::OffHand),
            power: 1,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 50,
            min_floor: 1,
            rarity: Rarity::Common,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Iron Shield",
            glyph: 0x29,
            item_type: ItemType::Shield,
            slot: Some(EquipSlot::OffHand),
            power: 3,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 50,
            min_floor: 3,
            rarity: Rarity::Uncommon,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Tower Shield",
            glyph: 0x29,
            item_type: ItemType::Shield,
            slot: Some(EquipSlot::OffHand),
            power: 5,
            speed_mod: -10,
            effect: None,
            charges: None,
            energy_cost: 50,
            min_floor: 6,
            rarity: Rarity::Rare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        // Accessories - Rings
        ItemTemplate {
            name: "Ring of Strength",
            glyph: 0x3D,
            item_type: ItemType::Ring,
            slot: Some(EquipSlot::Ring),
            power: 2,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 50,
            min_floor: 3,
            rarity: Rarity::Rare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Ring of Protection",
            glyph: 0x3D,
            item_type: ItemType::Ring,
            slot: Some(EquipSlot::Ring),
            power: 2,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 50,
            min_floor: 3,
            rarity: Rarity::Rare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Ring of Haste",
            glyph: 0x3D,
            item_type: ItemType::Ring,
            slot: Some(EquipSlot::Ring),
            power: 0,
            speed_mod: 20,
            effect: None,
            charges: None,
            energy_cost: 50,
            min_floor: 5,
            rarity: Rarity::VeryRare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Ring of Regeneration",
            glyph: 0x3D,
            item_type: ItemType::Ring,
            slot: Some(EquipSlot::Ring),
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::ApplyStatus {
                effect: StatusType::Regenerating,
                duration: 999,
            }),
            charges: None,
            energy_cost: 50,
            min_floor: 6,
            rarity: Rarity::VeryRare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        // Accessories - Amulets
        ItemTemplate {
            name: "Amulet of Health",
            glyph: 0x22,
            item_type: ItemType::Amulet,
            slot: Some(EquipSlot::Amulet),
            power: 20,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 50,
            min_floor: 3,
            rarity: Rarity::Rare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Amulet of Vision",
            glyph: 0x22,
            item_type: ItemType::Amulet,
            slot: Some(EquipSlot::Amulet),
            power: 3,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 50,
            min_floor: 4,
            rarity: Rarity::Rare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Amulet of Resistance",
            glyph: 0x22,
            item_type: ItemType::Amulet,
            slot: Some(EquipSlot::Amulet),
            power: 0,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 50,
            min_floor: 6,
            rarity: Rarity::VeryRare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        // Consumables - Potions
        ItemTemplate {
            name: "Health Potion",
            glyph: 0x21,
            item_type: ItemType::Potion,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::Heal(25)),
            charges: None,
            energy_cost: 100,
            min_floor: 1,
            rarity: Rarity::Common,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Greater Health Potion",
            glyph: 0x21,
            item_type: ItemType::Potion,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::Heal(50)),
            charges: None,
            energy_cost: 100,
            min_floor: 4,
            rarity: Rarity::Uncommon,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Potion of Strength",
            glyph: 0x21,
            item_type: ItemType::Potion,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::ApplyStatus {
                effect: StatusType::Strengthened,
                duration: 20,
            }),
            charges: None,
            energy_cost: 100,
            min_floor: 3,
            rarity: Rarity::Uncommon,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Potion of Speed",
            glyph: 0x21,
            item_type: ItemType::Potion,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::ApplyStatus {
                effect: StatusType::Hasted,
                duration: 15,
            }),
            charges: None,
            energy_cost: 100,
            min_floor: 3,
            rarity: Rarity::Uncommon,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Potion of Invisibility",
            glyph: 0x21,
            item_type: ItemType::Potion,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::ApplyStatus {
                effect: StatusType::Invisible,
                duration: 10,
            }),
            charges: None,
            energy_cost: 100,
            min_floor: 5,
            rarity: Rarity::Rare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Antidote",
            glyph: 0x21,
            item_type: ItemType::Potion,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::CureStatus),
            charges: None,
            energy_cost: 100,
            min_floor: 2,
            rarity: Rarity::Common,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        // Scrolls
        ItemTemplate {
            name: "Scroll of Reveal",
            glyph: 0x3F,
            item_type: ItemType::Scroll,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::RevealMap),
            charges: None,
            energy_cost: 100,
            min_floor: 2,
            rarity: Rarity::Uncommon,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Scroll of Teleport",
            glyph: 0x3F,
            item_type: ItemType::Scroll,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::Teleport),
            charges: None,
            energy_cost: 100,
            min_floor: 3,
            rarity: Rarity::Uncommon,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Scroll of Fireball",
            glyph: 0x3F,
            item_type: ItemType::Scroll,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::DamageArea {
                damage: 20,
                radius: 3,
            }),
            charges: None,
            energy_cost: 100,
            min_floor: 5,
            rarity: Rarity::Rare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Scroll of Confusion",
            glyph: 0x3F,
            item_type: ItemType::Scroll,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::ApplyStatus {
                effect: StatusType::Confused,
                duration: 5,
            }),
            charges: None,
            energy_cost: 100,
            min_floor: 4,
            rarity: Rarity::Uncommon,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Scroll of Detection",
            glyph: 0x3F,
            item_type: ItemType::Scroll,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::RevealSecrets),
            charges: None,
            energy_cost: 100,
            min_floor: 3,
            rarity: Rarity::Rare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        // Food
        ItemTemplate {
            name: "Food Ration",
            glyph: 0x25,
            item_type: ItemType::Food,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::Heal(15)),
            charges: None,
            energy_cost: 100,
            min_floor: 1,
            rarity: Rarity::Common,
            ammo_type: None,
            ranged: None,
            hunger_restore: 300,
        },
        ItemTemplate {
            name: "Dried Meat",
            glyph: 0x25,
            item_type: ItemType::Food,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::Heal(5)),
            charges: None,
            energy_cost: 100,
            min_floor: 1,
            rarity: Rarity::Common,
            ammo_type: None,
            ranged: None,
            hunger_restore: 200,
        },
        ItemTemplate {
            name: "Feast",
            glyph: 0x25,
            item_type: ItemType::Food,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::Heal(30)),
            charges: None,
            energy_cost: 100,
            min_floor: 5,
            rarity: Rarity::Rare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 500,
        },
        // Wands
        ItemTemplate {
            name: "Wand of Fire",
            glyph: 0x7C,
            item_type: ItemType::Wand,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::RangedAttack {
                damage: 8,
                status: Some((StatusType::Burning, 3)),
            }),
            charges: Some(8),
            energy_cost: 100,
            min_floor: 4,
            rarity: Rarity::Rare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Wand of Ice",
            glyph: 0x7C,
            item_type: ItemType::Wand,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::RangedAttack {
                damage: 8,
                status: Some((StatusType::Slowed, 3)),
            }),
            charges: Some(8),
            energy_cost: 100,
            min_floor: 4,
            rarity: Rarity::Rare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Wand of Lightning",
            glyph: 0x7C,
            item_type: ItemType::Wand,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: Some(ItemEffect::RangedAttack {
                damage: 12,
                status: None,
            }),
            charges: Some(5),
            energy_cost: 100,
            min_floor: 6,
            rarity: Rarity::VeryRare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        // Ranged Weapons
        ItemTemplate {
            name: "Shortbow",
            glyph: 0x7D,
            item_type: ItemType::Weapon,
            slot: Some(EquipSlot::MainHand),
            power: 2,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 100,
            min_floor: 2,
            rarity: Rarity::Uncommon,
            ammo_type: Some(AmmoType::Arrow),
            ranged: Some(RangedStats {
                range: 5,
                damage_bonus: 1,
            }),
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Longbow",
            glyph: 0x7D,
            item_type: ItemType::Weapon,
            slot: Some(EquipSlot::MainHand),
            power: 3,
            speed_mod: -10,
            effect: None,
            charges: None,
            energy_cost: 100,
            min_floor: 4,
            rarity: Rarity::Rare,
            ammo_type: Some(AmmoType::Arrow),
            ranged: Some(RangedStats {
                range: 8,
                damage_bonus: 2,
            }),
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Crossbow",
            glyph: 0x7D,
            item_type: ItemType::Weapon,
            slot: Some(EquipSlot::MainHand),
            power: 4,
            speed_mod: -20,
            effect: None,
            charges: None,
            energy_cost: 100,
            min_floor: 5,
            rarity: Rarity::Rare,
            ammo_type: Some(AmmoType::Bolt),
            ranged: Some(RangedStats {
                range: 6,
                damage_bonus: 4,
            }),
            hunger_restore: 0,
        },
        // Ammunition
        ItemTemplate {
            name: "Arrow",
            glyph: 0x2D,
            item_type: ItemType::Projectile,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: None,
            charges: Some(10),
            energy_cost: 0,
            min_floor: 2,
            rarity: Rarity::Common,
            ammo_type: Some(AmmoType::Arrow),
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Bolt",
            glyph: 0x2D,
            item_type: ItemType::Projectile,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: None,
            charges: Some(8),
            energy_cost: 0,
            min_floor: 5,
            rarity: Rarity::Uncommon,
            ammo_type: Some(AmmoType::Bolt),
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Throwing Knife",
            glyph: 0x2D,
            item_type: ItemType::Projectile,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: None,
            charges: Some(5),
            energy_cost: 100,
            min_floor: 3,
            rarity: Rarity::Uncommon,
            ammo_type: Some(AmmoType::ThrowingKnife),
            ranged: Some(RangedStats {
                range: 4,
                damage_bonus: 3,
            }),
            hunger_restore: 0,
        },
        // Keys
        ItemTemplate {
            name: "Iron Key",
            glyph: 0x7E,
            item_type: ItemType::Key,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 50,
            min_floor: 1,
            rarity: Rarity::Common,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Boss Key",
            glyph: 0x7E,
            item_type: ItemType::Key,
            slot: None,
            power: 0,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 50,
            min_floor: 3,
            rarity: Rarity::Common,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        // Unlockable reward items (not in random loot pool due to min_floor: 99)
        ItemTemplate {
            name: "Blessed Sword",
            glyph: 0x2F,
            item_type: ItemType::Weapon,
            slot: Some(EquipSlot::MainHand),
            power: 5,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 100,
            min_floor: 99,
            rarity: Rarity::VeryRare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Veteran's Ring",
            glyph: 0x6F,
            item_type: ItemType::Ring,
            slot: Some(EquipSlot::Ring),
            power: 2,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 100,
            min_floor: 99,
            rarity: Rarity::VeryRare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Abyss Cloak",
            glyph: 0x5B,
            item_type: ItemType::Armor,
            slot: Some(EquipSlot::Body),
            power: 3,
            speed_mod: 0,
            effect: None,
            charges: None,
            energy_cost: 100,
            min_floor: 99,
            rarity: Rarity::VeryRare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
        ItemTemplate {
            name: "Speed Boots",
            glyph: 0x62,
            item_type: ItemType::Armor,
            slot: Some(EquipSlot::Body),
            power: 0,
            speed_mod: 20,
            effect: None,
            charges: None,
            energy_cost: 100,
            min_floor: 99,
            rarity: Rarity::VeryRare,
            ammo_type: None,
            ranged: None,
            hunger_restore: 0,
        },
    ]
}

pub fn get_loot_pool(floor: u32) -> Vec<&'static str> {
    let all = all_items();
    all.iter()
        .filter(|t| t.min_floor <= floor && t.item_type != ItemType::Key)
        .map(|t| t.name)
        .collect()
}

pub fn find_template(name: &str) -> Option<ItemTemplate> {
    all_items().into_iter().find(|t| t.name == name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_items_non_empty() {
        let items = all_items();
        assert!(
            items.len() > 30,
            "Should have 30+ items, got {}",
            items.len()
        );
    }

    #[test]
    fn items_have_valid_types() {
        for item in all_items() {
            // Equipment items should have slots
            match item.item_type {
                ItemType::Weapon
                | ItemType::Armor
                | ItemType::Shield
                | ItemType::Ring
                | ItemType::Amulet => {
                    assert!(
                        item.slot.is_some(),
                        "{} is equipment but has no slot",
                        item.name
                    );
                }
                ItemType::Potion
                | ItemType::Scroll
                | ItemType::Food
                | ItemType::Wand
                | ItemType::Key
                | ItemType::Projectile => {
                    assert!(
                        item.slot.is_none(),
                        "{} is consumable but has a slot",
                        item.name
                    );
                }
            }
        }
    }

    #[test]
    fn consumables_have_effects() {
        for item in all_items() {
            match item.item_type {
                ItemType::Potion | ItemType::Scroll | ItemType::Food | ItemType::Wand => {
                    assert!(
                        item.effect.is_some(),
                        "{} is consumable but has no effect",
                        item.name
                    );
                }
                _ => {}
            }
        }
    }

    #[test]
    fn find_template_works() {
        let dagger = find_template("Dagger");
        assert!(dagger.is_some());
        assert_eq!(dagger.unwrap().name, "Dagger");
    }

    #[test]
    fn find_template_missing() {
        assert!(find_template("Nonexistent Item").is_none());
    }

    #[test]
    fn loot_pool_scales_with_floor() {
        let pool_f1 = get_loot_pool(1);
        let pool_f10 = get_loot_pool(10);
        assert!(
            pool_f10.len() >= pool_f1.len(),
            "Higher floors should have equal or more items"
        );
    }

    #[test]
    fn loot_pool_excludes_keys() {
        for floor in 1..=10 {
            let pool = get_loot_pool(floor);
            assert!(
                !pool.contains(&"Iron Key"),
                "Loot pool should not contain keys"
            );
            assert!(
                !pool.contains(&"Boss Key"),
                "Loot pool should not contain boss keys"
            );
        }
    }

    #[test]
    fn rarity_weights() {
        assert!(Rarity::Common.weight() > Rarity::Uncommon.weight());
        assert!(Rarity::Uncommon.weight() > Rarity::Rare.weight());
        assert!(Rarity::Rare.weight() > Rarity::VeryRare.weight());
    }

    #[test]
    fn unique_item_names() {
        let items = all_items();
        let mut names: Vec<&str> = items.iter().map(|i| i.name).collect();
        let len_before = names.len();
        names.sort();
        names.dedup();
        assert_eq!(names.len(), len_before, "Item names should be unique");
    }
}
