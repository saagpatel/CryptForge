use serde::{Deserialize, Serialize};

use super::entity::Position;

pub const MAP_WIDTH: usize = 80;
pub const MAP_HEIGHT: usize = 50;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TileType {
    Wall,
    Floor,
    DownStairs,
    UpStairs,
    DoorClosed,
    DoorOpen,
    SecretWall,
}

impl TileType {
    pub fn is_walkable(&self) -> bool {
        matches!(
            self,
            TileType::Floor | TileType::DownStairs | TileType::UpStairs | TileType::DoorOpen
        )
    }

    pub fn blocks_fov(&self) -> bool {
        matches!(
            self,
            TileType::Wall | TileType::DoorClosed | TileType::SecretWall
        )
    }

    pub fn as_str(&self) -> &'static str {
        match self {
            TileType::Wall => "Wall",
            TileType::Floor => "Floor",
            TileType::DownStairs => "DownStairs",
            TileType::UpStairs => "UpStairs",
            TileType::DoorClosed => "DoorClosed",
            TileType::DoorOpen => "DoorOpen",
            TileType::SecretWall => "Wall", // Renders as Wall to hide from player
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Map {
    pub width: usize,
    pub height: usize,
    pub tiles: Vec<TileType>,
    pub revealed: Vec<bool>,
    pub rooms: Vec<Room>,
    pub blocked: Vec<bool>,
}

impl Map {
    pub fn new(width: usize, height: usize) -> Self {
        let size = width * height;
        Self {
            width,
            height,
            tiles: vec![TileType::Wall; size],
            revealed: vec![false; size],
            rooms: Vec::new(),
            blocked: vec![false; size],
        }
    }

    pub fn default_map() -> Self {
        Self::new(MAP_WIDTH, MAP_HEIGHT)
    }

    pub fn idx(&self, x: i32, y: i32) -> usize {
        (y as usize) * self.width + (x as usize)
    }

    pub fn in_bounds(&self, x: i32, y: i32) -> bool {
        x >= 0 && y >= 0 && (x as usize) < self.width && (y as usize) < self.height
    }

    pub fn is_walkable(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return false;
        }
        let idx = self.idx(x, y);
        self.tiles[idx].is_walkable() && !self.blocked[idx]
    }

    pub fn is_opaque(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return true;
        }
        self.tiles[self.idx(x, y)].blocks_fov()
    }

    pub fn set_tile(&mut self, x: i32, y: i32, tile: TileType) {
        if self.in_bounds(x, y) {
            let idx = self.idx(x, y);
            self.tiles[idx] = tile;
        }
    }

    pub fn get_tile(&self, x: i32, y: i32) -> TileType {
        if !self.in_bounds(x, y) {
            return TileType::Wall;
        }
        self.tiles[self.idx(x, y)]
    }

    pub fn reveal(&mut self, x: i32, y: i32) {
        if self.in_bounds(x, y) {
            let idx = self.idx(x, y);
            self.revealed[idx] = true;
        }
    }

    pub fn is_revealed(&self, x: i32, y: i32) -> bool {
        if !self.in_bounds(x, y) {
            return false;
        }
        self.revealed[self.idx(x, y)]
    }

    pub fn pos_to_idx(&self, pos: &Position) -> usize {
        self.idx(pos.x, pos.y)
    }

    pub fn idx_to_pos(&self, idx: usize) -> Position {
        Position::new((idx % self.width) as i32, (idx / self.width) as i32)
    }

    pub fn refresh_blocked(&mut self) {
        for i in 0..self.blocked.len() {
            self.blocked[i] = !self.tiles[i].is_walkable();
        }
    }

    pub fn reveal_all(&mut self) {
        for r in self.revealed.iter_mut() {
            *r = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn new_map_all_walls() {
        let map = Map::new(10, 10);
        assert!(map.tiles.iter().all(|t| *t == TileType::Wall));
    }

    #[test]
    fn default_map_dimensions() {
        let map = Map::default_map();
        assert_eq!(map.width, MAP_WIDTH);
        assert_eq!(map.height, MAP_HEIGHT);
        assert_eq!(map.tiles.len(), MAP_WIDTH * MAP_HEIGHT);
    }

    #[test]
    fn in_bounds_checks() {
        let map = Map::new(10, 10);
        assert!(map.in_bounds(0, 0));
        assert!(map.in_bounds(9, 9));
        assert!(!map.in_bounds(-1, 0));
        assert!(!map.in_bounds(10, 0));
        assert!(!map.in_bounds(0, 10));
    }

    #[test]
    fn set_and_get_tile() {
        let mut map = Map::new(10, 10);
        map.set_tile(3, 4, TileType::Floor);
        assert_eq!(map.get_tile(3, 4), TileType::Floor);
        assert_eq!(map.get_tile(0, 0), TileType::Wall);
    }

    #[test]
    fn out_of_bounds_returns_wall() {
        let map = Map::new(10, 10);
        assert_eq!(map.get_tile(-1, 0), TileType::Wall);
        assert_eq!(map.get_tile(100, 0), TileType::Wall);
    }

    #[test]
    fn walkability() {
        let mut map = Map::new(10, 10);
        assert!(!map.is_walkable(5, 5)); // wall
        map.set_tile(5, 5, TileType::Floor);
        map.refresh_blocked();
        assert!(map.is_walkable(5, 5));
        map.set_tile(5, 5, TileType::DoorClosed);
        map.refresh_blocked();
        assert!(!map.is_walkable(5, 5));
        map.set_tile(5, 5, TileType::DoorOpen);
        map.refresh_blocked();
        assert!(map.is_walkable(5, 5));
    }

    #[test]
    fn reveal_and_check() {
        let mut map = Map::new(10, 10);
        assert!(!map.is_revealed(3, 3));
        map.reveal(3, 3);
        assert!(map.is_revealed(3, 3));
    }

    #[test]
    fn reveal_all_marks_everything() {
        let mut map = Map::new(5, 5);
        map.reveal_all();
        for y in 0..5i32 {
            for x in 0..5i32 {
                assert!(map.is_revealed(x, y));
            }
        }
    }

    #[test]
    fn idx_to_pos_round_trip() {
        let map = Map::new(80, 50);
        let pos = Position::new(15, 23);
        let idx = map.pos_to_idx(&pos);
        let back = map.idx_to_pos(idx);
        assert_eq!(back, pos);
    }

    #[test]
    fn tile_type_properties() {
        assert!(!TileType::Wall.is_walkable());
        assert!(TileType::Floor.is_walkable());
        assert!(TileType::DownStairs.is_walkable());
        assert!(TileType::UpStairs.is_walkable());
        assert!(!TileType::DoorClosed.is_walkable());
        assert!(TileType::DoorOpen.is_walkable());

        assert!(TileType::Wall.blocks_fov());
        assert!(!TileType::Floor.blocks_fov());
        assert!(TileType::DoorClosed.blocks_fov());
        assert!(!TileType::DoorOpen.blocks_fov());
    }

    #[test]
    fn room_center() {
        let room = Room::new(10, 20, 8, 6);
        let center = room.center();
        assert_eq!(center.x, 14);
        assert_eq!(center.y, 23);
    }

    #[test]
    fn room_contains() {
        let room = Room::new(5, 5, 4, 4);
        assert!(room.contains(&Position::new(5, 5)));
        assert!(room.contains(&Position::new(8, 8)));
        assert!(!room.contains(&Position::new(9, 5)));
        assert!(!room.contains(&Position::new(4, 5)));
    }

    #[test]
    fn room_intersects() {
        let room1 = Room::new(0, 0, 5, 5);
        let room2 = Room::new(3, 3, 5, 5);
        let room3 = Room::new(10, 10, 5, 5);
        assert!(room1.intersects(&room2));
        assert!(!room1.intersects(&room3));
    }

    #[test]
    fn room_inner_positions_excludes_border() {
        let room = Room::new(0, 0, 4, 4);
        let inner = room.inner_positions();
        // Inner positions: (1,1), (2,1), (1,2), (2,2) = 4 positions
        assert_eq!(inner.len(), 4);
        // None should be on the border
        for pos in &inner {
            assert!(pos.x > 0 && pos.x < 3);
            assert!(pos.y > 0 && pos.y < 3);
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Room {
    pub x: i32,
    pub y: i32,
    pub width: i32,
    pub height: i32,
    pub room_type: RoomType,
}

impl Room {
    pub fn new(x: i32, y: i32, width: i32, height: i32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            room_type: RoomType::Normal,
        }
    }

    pub fn center(&self) -> Position {
        Position::new(self.x + self.width / 2, self.y + self.height / 2)
    }

    pub fn intersects(&self, other: &Room) -> bool {
        self.x <= other.x + other.width
            && self.x + self.width >= other.x
            && self.y <= other.y + other.height
            && self.y + self.height >= other.y
    }

    pub fn contains(&self, pos: &Position) -> bool {
        pos.x >= self.x
            && pos.x < self.x + self.width
            && pos.y >= self.y
            && pos.y < self.y + self.height
    }

    pub fn inner_positions(&self) -> Vec<Position> {
        let mut positions = Vec::new();
        for y in (self.y + 1)..(self.y + self.height - 1) {
            for x in (self.x + 1)..(self.x + self.width - 1) {
                positions.push(Position::new(x, y));
            }
        }
        positions
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RoomType {
    Normal,
    Start,
    Treasure,
    Boss,
    Shrine,
    Library,
    Armory,
    Shop,
}
