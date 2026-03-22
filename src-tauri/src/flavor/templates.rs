use rand::Rng;

const ITEM_PREFIXES: &[&str] = &[
    "Ancient",
    "Blessed",
    "Cursed",
    "Enchanted",
    "Forgotten",
    "Gleaming",
    "Haunted",
    "Infernal",
    "Jagged",
    "Keen",
    "Lost",
    "Mystic",
    "Noble",
    "Ornate",
    "Primal",
    "Radiant",
    "Sacred",
    "Twisted",
    "Unholy",
    "Wicked",
];

const ITEM_SUFFIXES: &[&str] = &[
    "of the Abyss",
    "of Valor",
    "of Shadows",
    "of the Fallen",
    "of Flame",
    "of Ice",
    "of Thunder",
    "of the Deep",
    "of Ruin",
    "of Grace",
    "of Wrath",
    "of the Void",
    "of Secrets",
    "of Despair",
    "of Glory",
    "of the Ancients",
];

const ITEM_DESCRIPTIONS: &[&str] = &[
    "A finely crafted weapon with runes etched along the blade.",
    "This item radiates a faint warmth when held.",
    "Worn smooth by countless hands before yours.",
    "The metal has a strange, iridescent sheen.",
    "Faint whispers seem to emanate from within.",
    "Cold to the touch, even near flame.",
    "Covered in intricate patterns that shift when not observed directly.",
    "Surprisingly light for its apparent heft.",
    "The craftsmanship is beyond anything you've seen on this floor.",
    "Pulsing with barely contained energy.",
    "Fragments of an inscription remain: '...last of the forge...'",
    "The edges gleam despite the dungeon's damp air.",
    "Bloodstains suggest its previous owner met a violent end.",
    "A gemstone at the hilt glows with inner fire.",
    "Masterwork construction from an era long past.",
    "Still sharp despite apparent age.",
    "Wrapped in leather bindings that never seem to slip.",
    "Etched with protective wards.",
    "The air around it hums with latent power.",
    "Ornamental but deadly — form and function united.",
];

const ENEMY_DESCRIPTIONS: &[&str] = &[
    "Its eyes burn with malevolent intelligence.",
    "Scars crisscross its body — a veteran of many battles.",
    "It moves with predatory grace.",
    "A low growl reverberates through the chamber.",
    "Its presence fills the room with dread.",
    "Muscles coil beneath its skin, ready to strike.",
    "It regards you with cold calculation.",
    "The stench of death clings to it.",
    "Its form flickers at the edges, as if not fully present.",
    "Battle-hardened and hungry.",
    "It blocks your path with clear intent.",
    "Ancient hatred gleams in its eyes.",
    "It has clearly made this floor its territory.",
    "Wounds that should be fatal don't seem to slow it.",
    "It watches you with an unsettling patience.",
    "The darkness seems to bend toward it.",
    "Trophy bones rattle from its belt.",
    "Its breath comes in ragged, eager gasps.",
    "A creature born from the dungeon's darkest depths.",
    "It's been waiting for someone like you.",
];

const ROOM_DESCRIPTIONS: &[&str] = &[
    "Dust motes dance in the flickering torchlight.",
    "Water drips steadily from the ceiling.",
    "The walls are covered in faded murals of forgotten battles.",
    "A cold draft whistles through cracks in the stonework.",
    "The remnants of a campfire suggest someone was here recently.",
    "Mushrooms grow in clusters along the damp walls.",
    "Chains hang from the ceiling, their purpose long forgotten.",
    "The floor is worn smooth by centuries of footsteps.",
    "A faint humming fills the air from an unknown source.",
    "Cobwebs thick as curtains drape the corners.",
    "Claw marks score the walls at varying heights.",
    "The air tastes metallic, like old blood.",
    "Broken furniture lies scattered across the floor.",
    "Crystalline formations jut from the walls, casting prismatic light.",
    "The ceiling arches high above, lost in shadow.",
    "Ancient runes pulse dimly on the floor tiles.",
    "A mosaic floor depicts scenes of the dungeon's construction.",
    "The room smells of damp earth and old stone.",
    "Alcoves in the walls once held statues, now empty.",
    "The temperature drops noticeably as you enter.",
];

const DEATH_EPITAPHS: &[&str] = &[
    "They ventured too deep and paid the price.",
    "A brave soul lost to the darkness below.",
    "The dungeon claims another.",
    "Their torch flickered and died — and so did they.",
    "Overconfidence was a slow and insidious killer.",
    "They fought bravely. It wasn't enough.",
    "The depths remember all who fall within.",
    "Another name added to the dungeon's endless tally.",
    "They sought glory and found only stone and silence.",
    "The last thing they heard was their own heartbeat.",
    "Ambition exceeded preparation.",
    "The monsters feasted well that day.",
    "They never did find those stairs down.",
    "A moment's hesitation proved fatal.",
    "The dungeon does not forgive mistakes.",
    "Perhaps the next adventurer will fare better.",
    "They died as they lived — surrounded by enemies.",
    "Not even their bones will see the surface again.",
    "The weight of the mountain pressed down upon them, final.",
    "In the end, the dungeon always wins.",
];

/// Generate a fallback flavor description for an item.
pub fn fallback_item_description(_item_name: &str, rng: &mut impl Rng) -> String {
    let desc = ITEM_DESCRIPTIONS[rng.gen_range(0..ITEM_DESCRIPTIONS.len())];
    format!("{}", desc)
}

/// Generate a fallback flavor name for an item.
pub fn fallback_item_name(base_name: &str, rng: &mut impl Rng) -> String {
    let prefix = ITEM_PREFIXES[rng.gen_range(0..ITEM_PREFIXES.len())];
    let suffix = ITEM_SUFFIXES[rng.gen_range(0..ITEM_SUFFIXES.len())];
    format!("{} {} {}", prefix, base_name, suffix)
}

/// Generate a fallback flavor description for an enemy.
pub fn fallback_enemy_description(_enemy_name: &str, rng: &mut impl Rng) -> String {
    let desc = ENEMY_DESCRIPTIONS[rng.gen_range(0..ENEMY_DESCRIPTIONS.len())];
    format!("{}", desc)
}

/// Generate a fallback room description.
pub fn fallback_room_description(rng: &mut impl Rng) -> String {
    ROOM_DESCRIPTIONS[rng.gen_range(0..ROOM_DESCRIPTIONS.len())].to_string()
}

/// Generate a fallback death epitaph.
pub fn fallback_death_epitaph(rng: &mut impl Rng) -> String {
    DEATH_EPITAPHS[rng.gen_range(0..DEATH_EPITAPHS.len())].to_string()
}
