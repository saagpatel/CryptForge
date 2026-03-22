use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub type EntityId = u32;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    pub id: EntityId,
    pub name: String,
    pub position: Position,
    pub glyph: u32,
    pub render_order: RenderOrder,
    pub blocks_movement: bool,
    pub blocks_fov: bool,
    pub health: Option<Health>,
    pub combat: Option<CombatStats>,
    pub ai: Option<AIBehavior>,
    pub inventory: Option<Inventory>,
    pub equipment: Option<EquipmentSlots>,
    pub item: Option<ItemProperties>,
    pub status_effects: Vec<StatusEffect>,
    pub fov: Option<FieldOfView>,
    pub door: Option<DoorState>,
    pub trap: Option<TrapProperties>,
    pub stair: Option<StairDirection>,
    pub loot_table: Option<LootTable>,
    pub flavor_text: Option<String>,
    pub shop: Option<ShopInventory>,
    pub interactive: Option<Interactive>,
    pub elite: Option<ElitePrefix>,
    pub resurrection_timer: Option<u32>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Position {
    pub x: i32,
    pub y: i32,
}

impl Position {
    pub fn new(x: i32, y: i32) -> Self {
        Self { x, y }
    }

    pub fn distance_to(&self, other: &Position) -> f64 {
        let dx = (self.x - other.x) as f64;
        let dy = (self.y - other.y) as f64;
        (dx * dx + dy * dy).sqrt()
    }

    pub fn chebyshev_distance(&self, other: &Position) -> i32 {
        (self.x - other.x).abs().max((self.y - other.y).abs())
    }

    pub fn apply_direction(&self, dir: Direction) -> Position {
        let (dx, dy) = dir.delta();
        Position::new(self.x + dx, self.y + dy)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Direction {
    N,
    S,
    E,
    W,
    NE,
    NW,
    SE,
    SW,
}

impl Direction {
    pub fn delta(&self) -> (i32, i32) {
        match self {
            Direction::N => (0, -1),
            Direction::S => (0, 1),
            Direction::E => (1, 0),
            Direction::W => (-1, 0),
            Direction::NE => (1, -1),
            Direction::NW => (-1, -1),
            Direction::SE => (1, 1),
            Direction::SW => (-1, 1),
        }
    }

    pub const ALL: [Direction; 8] = [
        Direction::N,
        Direction::S,
        Direction::E,
        Direction::W,
        Direction::NE,
        Direction::NW,
        Direction::SE,
        Direction::SW,
    ];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum RenderOrder {
    Background = 0,
    Trap = 1,
    Item = 2,
    Door = 3,
    Enemy = 4,
    Player = 5,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    pub fn is_dead(&self) -> bool {
        self.current <= 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CombatStats {
    pub base_attack: i32,
    pub base_defense: i32,
    pub base_speed: i32,
    pub crit_chance: f32,
    pub dodge_chance: f32,
    pub ranged: Option<RangedStats>,
    pub on_hit: Option<OnHitEffect>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OnHitEffect {
    Poison { damage: i32, duration: u32 },
    Burn { damage: i32, duration: u32 },
    Slow { magnitude: i32, duration: u32 },
    Confuse { duration: u32 },
    LifeSteal,
    DrainMaxHp,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct RangedStats {
    pub range: i32,
    pub damage_bonus: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AIBehavior {
    Melee,
    Ranged { range: i32, preferred_distance: i32 },
    Passive,
    Fleeing,
    Boss(BossPhase),
    Ally { follow_distance: i32 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum BossPhase {
    Phase1,
    Phase2,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Inventory {
    pub items: Vec<Entity>,
    pub max_size: usize,
}

impl Inventory {
    pub fn new(max_size: usize) -> Self {
        Self {
            items: Vec::new(),
            max_size,
        }
    }

    pub fn is_full(&self) -> bool {
        self.items.len() >= self.max_size
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentSlots {
    pub main_hand: Option<EntityId>,
    pub off_hand: Option<EntityId>,
    pub head: Option<EntityId>,
    pub body: Option<EntityId>,
    pub ring: Option<EntityId>,
    pub amulet: Option<EntityId>,
}

impl EquipmentSlots {
    pub fn empty() -> Self {
        Self {
            main_hand: None,
            off_hand: None,
            head: None,
            body: None,
            ring: None,
            amulet: None,
        }
    }

    pub fn get_slot(&self, slot: EquipSlot) -> Option<EntityId> {
        match slot {
            EquipSlot::MainHand => self.main_hand,
            EquipSlot::OffHand => self.off_hand,
            EquipSlot::Head => self.head,
            EquipSlot::Body => self.body,
            EquipSlot::Ring => self.ring,
            EquipSlot::Amulet => self.amulet,
        }
    }

    pub fn set_slot(&mut self, slot: EquipSlot, id: Option<EntityId>) {
        match slot {
            EquipSlot::MainHand => self.main_hand = id,
            EquipSlot::OffHand => self.off_hand = id,
            EquipSlot::Head => self.head = id,
            EquipSlot::Body => self.body = id,
            EquipSlot::Ring => self.ring = id,
            EquipSlot::Amulet => self.amulet = id,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EquipSlot {
    MainHand,
    OffHand,
    Head,
    Body,
    Ring,
    Amulet,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemProperties {
    pub item_type: ItemType,
    pub slot: Option<EquipSlot>,
    pub power: i32,
    pub speed_mod: i32,
    pub effect: Option<ItemEffect>,
    pub charges: Option<u32>,
    pub energy_cost: i32,
    pub ammo_type: Option<AmmoType>,
    pub ranged: Option<RangedStats>,
    pub hunger_restore: i32,
    pub enchant_level: i32,
    pub identified: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemType {
    Weapon,
    Armor,
    Shield,
    Ring,
    Amulet,
    Potion,
    Scroll,
    Wand,
    Key,
    Food,
    Projectile,
}

impl ItemType {
    pub fn is_consumable(&self) -> bool {
        matches!(self, ItemType::Potion | ItemType::Scroll | ItemType::Food)
    }

    pub fn is_equipment(&self) -> bool {
        matches!(
            self,
            ItemType::Weapon
                | ItemType::Armor
                | ItemType::Shield
                | ItemType::Ring
                | ItemType::Amulet
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AmmoType {
    Arrow,
    Bolt,
    ThrowingKnife,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ItemEffect {
    Heal(i32),
    DamageArea {
        damage: i32,
        radius: i32,
    },
    ApplyStatus {
        effect: StatusType,
        duration: u32,
    },
    RevealMap,
    RevealSecrets,
    Teleport,
    CureStatus,
    RangedAttack {
        damage: i32,
        status: Option<(StatusType, u32)>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DoorState {
    pub open: bool,
    pub locked: bool,
    pub key_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrapProperties {
    pub trap_type: TrapType,
    pub revealed: bool,
    pub triggered: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrapType {
    Spike { damage: i32 },
    Poison { damage: i32, duration: u32 },
    Teleport,
    Alarm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum StairDirection {
    Down,
    Up,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldOfView {
    pub radius: i32,
    pub visible_tiles: HashSet<Position>,
    pub dirty: bool,
}

impl FieldOfView {
    pub fn new(radius: i32) -> Self {
        Self {
            radius,
            visible_tiles: HashSet::new(),
            dirty: true,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusEffect {
    pub effect_type: StatusType,
    pub duration: u32,
    pub magnitude: i32,
    pub source: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StatusType {
    Poison,
    Burning,
    Stunned,
    Confused,
    Weakened,
    Strengthened,
    Blinded,
    Regenerating,
    Hasted,
    Slowed,
    Shielded,
    Invisible,
}

impl StatusType {
    pub fn is_negative(&self) -> bool {
        matches!(
            self,
            StatusType::Poison
                | StatusType::Burning
                | StatusType::Stunned
                | StatusType::Confused
                | StatusType::Weakened
                | StatusType::Blinded
                | StatusType::Slowed
        )
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootTable {
    pub entries: Vec<LootEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LootEntry {
    pub item_name: String,
    pub weight: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopInventory {
    pub items: Vec<ShopItem>,
    pub buy_multiplier: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopItem {
    pub name: String,
    pub price: u32,
    pub item_type: ItemType,
    pub slot: Option<EquipSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopView {
    pub shop_id: u32,
    pub name: String,
    pub items: Vec<ShopItemView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShopItemView {
    pub name: String,
    pub price: u32,
    pub item_type: ItemType,
    pub slot: Option<EquipSlot>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interactive {
    pub interaction_type: InteractionType,
    pub uses_remaining: Option<u32>,
    pub activated: bool,
    /// Items contained in chests
    pub contained_items: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum InteractionType {
    Barrel,
    Lever,
    Fountain,
    Altar,
    Chest,
    Anvil,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Biome {
    Dungeon,
    Crypt,
    Caves,
    Inferno,
    Abyss,
}

impl Biome {
    pub fn for_floor(floor: u32) -> Self {
        match ((floor - 1) % 10) + 1 {
            1 | 2 => Biome::Dungeon,
            3 | 4 => Biome::Crypt,
            5 | 6 => Biome::Caves,
            7 | 8 => Biome::Inferno,
            _ => Biome::Abyss,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum EntityType {
    Player,
    Enemy,
    Item,
    Door,
    Trap,
    Stairs,
    Interactive,
}

// IPC view types — sent to frontend

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerAction {
    pub action_type: PlayerActionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PlayerActionType {
    Move(Direction),
    Wait,
    PickUp,
    UseStairs,
    UseItem(u32),
    DropItem(u32),
    EquipItem(u32),
    UnequipSlot(EquipSlot),
    LevelUpChoice(LevelUpChoice),
    ClickMove {
        x: i32,
        y: i32,
    },
    AutoExplore,
    RangedAttack {
        target_id: u32,
    },
    BuyItem {
        shop_id: u32,
        index: usize,
    },
    SellItem {
        index: usize,
        shop_id: u32,
    },
    Interact,
    UseAbility {
        ability_id: String,
        target: Option<Position>,
    },
    Craft {
        weapon_idx: u32,
        scroll_idx: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LevelUpChoice {
    MaxHp,
    Attack,
    Defense,
    Speed,
    Cleave,
    Fortify,
    Backstab,
    Evasion,
    SpellPower,
    ManaRegen,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PlayerClass {
    Warrior,
    Rogue,
    Mage,
}

impl Default for PlayerClass {
    fn default() -> Self {
        PlayerClass::Warrior
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ElitePrefix {
    Frenzied,
    Armored,
    Venomous,
    Spectral,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RunModifier {
    GlassCannon,
    Marathon,
    Pacifist,
    Cursed,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum AutoExploreInterrupt {
    EnemySpotted,
    TookDamage,
    ItemFound,
    StairsReached,
    FullyExplored,
    NothingReachable,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TurnResult {
    pub state: GameState,
    pub events: Vec<GameEvent>,
    pub game_over: Option<GameOverInfo>,
    pub auto_explore_interrupt: Option<AutoExploreInterrupt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameState {
    pub player: PlayerState,
    pub visible_tiles: Vec<VisibleTile>,
    pub visible_entities: Vec<EntityView>,
    pub floor: u32,
    pub turn: u32,
    pub messages: Vec<LogMessage>,
    pub minimap: MinimapData,
    pub pending_level_up: bool,
    pub biome: Biome,
    pub seed: u64,
    pub level_up_choices: Vec<LevelUpChoice>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlayerState {
    pub position: Position,
    pub hp: i32,
    pub max_hp: i32,
    pub attack: i32,
    pub defense: i32,
    pub speed: i32,
    pub level: u32,
    pub xp: u32,
    pub xp_to_next: u32,
    pub gold: u32,
    pub inventory: Vec<ItemView>,
    pub equipment: EquipmentView,
    pub status_effects: Vec<StatusView>,
    pub player_class: PlayerClass,
    pub mana: i32,
    pub max_mana: i32,
    pub abilities: Vec<AbilityView>,
    pub hunger: i32,
    pub max_hunger: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbilityView {
    pub id: String,
    pub name: String,
    pub mana_cost: i32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VisibleTile {
    pub x: i32,
    pub y: i32,
    pub tile_type: String,
    pub explored: bool,
    pub visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityView {
    pub id: u32,
    pub name: String,
    pub position: Position,
    pub entity_type: EntityType,
    pub glyph: u32,
    pub hp: Option<(i32, i32)>,
    pub flavor_text: Option<String>,
    pub status_effects: Vec<StatusView>,
    pub elite: Option<String>,
    pub is_ally: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ItemView {
    pub id: u32,
    pub name: String,
    pub item_type: ItemType,
    pub slot: Option<EquipSlot>,
    pub charges: Option<u32>,
    pub identified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EquipmentView {
    pub main_hand: Option<ItemView>,
    pub off_hand: Option<ItemView>,
    pub head: Option<ItemView>,
    pub body: Option<ItemView>,
    pub ring: Option<ItemView>,
    pub amulet: Option<ItemView>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StatusView {
    pub effect_type: StatusType,
    pub duration: u32,
    pub magnitude: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LogMessage {
    pub text: String,
    pub turn: u32,
    pub severity: LogSeverity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LogSeverity {
    Info,
    Warning,
    Danger,
    Good,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinimapData {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<u8>, // 0=unknown, 1=wall, 2=floor, 3=stairs
    pub player_x: i32,
    pub player_y: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GameEvent {
    Moved {
        entity_id: u32,
        from: Position,
        to: Position,
    },
    Attacked {
        attacker_id: u32,
        target_id: u32,
        damage: i32,
        killed: bool,
        damage_type: String,
        dodged: bool,
    },
    DamageTaken {
        entity_id: u32,
        amount: i32,
        source: String,
    },
    Healed {
        entity_id: u32,
        amount: i32,
    },
    ItemPickedUp {
        item: ItemView,
    },
    ItemUsed {
        item: ItemView,
        effect: String,
    },
    ItemDropped {
        item: ItemView,
    },
    ItemEquipped {
        item: ItemView,
        slot: EquipSlot,
    },
    StatusApplied {
        entity_id: u32,
        effect: StatusType,
        duration: u32,
    },
    StatusExpired {
        entity_id: u32,
        effect: StatusType,
    },
    DoorOpened {
        position: Position,
    },
    TrapTriggered {
        position: Position,
        trap_type: String,
        damage: i32,
    },
    StairsDescended {
        new_floor: u32,
    },
    EnemySpotted {
        entity_id: u32,
        name: String,
    },
    LevelUp {
        new_level: u32,
    },
    FlavorText {
        text: String,
    },
    PlayerDied {
        cause: String,
    },
    BossDefeated {
        name: String,
        floor: u32,
    },
    ProjectileFired {
        from: Position,
        to: Position,
        hit: bool,
    },
    ItemBought {
        name: String,
        price: u32,
    },
    ItemSold {
        name: String,
        price: u32,
    },
    GoldGained {
        amount: u32,
    },
    BarrelSmashed {
        position: Position,
        dropped_item: Option<String>,
    },
    LeverPulled {
        position: Position,
    },
    FountainUsed {
        position: Position,
        effect: String,
    },
    ChestOpened {
        position: Position,
        items: Vec<String>,
        trapped: bool,
    },
    AltarOffering {
        item_name: String,
        stat_gained: String,
    },
    AchievementUnlocked {
        name: String,
    },
    AbilityUsed {
        name: String,
        position: Position,
        targets: Vec<Position>,
    },
    ManaChanged {
        amount: i32,
    },
    HungerChanged {
        level: i32,
    },
    SecretRoomFound {
        position: Position,
    },
    ItemEnchanted {
        item_name: String,
        new_level: i32,
    },
    BossSummon {
        boss_name: String,
        summoned: Vec<String>,
    },
    BossCharge {
        boss_id: u32,
        from: Position,
        to: Position,
    },
    Victory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GameOverInfo {
    pub cause_of_death: String,
    pub epitaph: Option<String>,
    pub final_score: u32,
    pub run_summary: RunSummary,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunSummary {
    pub seed: String,
    pub floor_reached: u32,
    pub enemies_killed: u32,
    pub bosses_killed: u32,
    pub level_reached: u32,
    pub turns_taken: u32,
    pub score: u32,
    pub cause_of_death: Option<String>,
    pub victory: bool,
    pub timestamp: String,
    pub class: String,
    pub modifiers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HighScore {
    pub rank: u32,
    pub score: u32,
    pub floor_reached: u32,
    pub seed: String,
    pub timestamp: String,
    pub victory: bool,
    pub class: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDetail {
    pub id: u32,
    pub name: String,
    pub entity_type: EntityType,
    pub hp: Option<(i32, i32)>,
    pub attack: Option<i32>,
    pub defense: Option<i32>,
    pub status_effects: Vec<StatusView>,
    pub flavor_text: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub tile_size: u32,
    pub master_volume: u32,
    pub sfx_volume: u32,
    pub ambient_volume: u32,
    pub fullscreen: bool,
    pub ollama_enabled: bool,
    pub ollama_url: String,
    pub ollama_model: String,
    pub ollama_timeout: u32,
    pub tileset_mode: String,
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            tile_size: 32,
            master_volume: 80,
            sfx_volume: 80,
            ambient_volume: 50,
            fullscreen: false,
            ollama_enabled: false,
            ollama_url: "http://localhost:11434".to_string(),
            ollama_model: "llama3.2".to_string(),
            ollama_timeout: 3,
            tileset_mode: "ascii".to_string(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OllamaStatus {
    pub available: bool,
    pub model_loaded: bool,
    pub url: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LifetimeStats {
    pub total_runs: u32,
    pub total_kills: u32,
    pub total_deaths: u32,
    pub total_victories: u32,
    pub floors_explored: u32,
    pub favorite_class: String,
    pub average_score: f64,
    pub win_rate: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyStatus {
    pub date: String,
    pub played: bool,
    pub score: Option<u32>,
    pub floor_reached: Option<u32>,
}
