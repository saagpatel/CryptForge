use rand::Rng;

use crate::engine::map::{Map, Room, TileType, MAP_HEIGHT, MAP_WIDTH};

use super::corridor::carve_l_corridor;

const MIN_NODE_W: i32 = 12;
const MIN_NODE_H: i32 = 10;
const MIN_ROOM_W: i32 = 4;
const MIN_ROOM_H: i32 = 4;
const MAX_ROOM_W: i32 = 12;
const MAX_ROOM_H: i32 = 10;

struct BspNode {
    x: i32,
    y: i32,
    w: i32,
    h: i32,
    left: Option<Box<BspNode>>,
    right: Option<Box<BspNode>>,
    room: Option<Room>,
}

impl BspNode {
    fn new(x: i32, y: i32, w: i32, h: i32) -> Self {
        Self {
            x,
            y,
            w,
            h,
            left: None,
            right: None,
            room: None,
        }
    }

    fn split(&mut self, rng: &mut impl Rng, depth: usize, max_depth: usize) {
        if depth >= max_depth {
            return;
        }

        let can_split_h = self.w >= MIN_NODE_W * 2;
        let can_split_v = self.h >= MIN_NODE_H * 2;

        if !can_split_h && !can_split_v {
            return;
        }

        let split_horizontal = if can_split_h && can_split_v {
            // Prefer splitting the longer dimension
            if self.w as f32 / self.h as f32 > 1.25 {
                true
            } else if self.h as f32 / self.w as f32 > 1.25 {
                false
            } else {
                rng.gen_bool(0.5)
            }
        } else {
            can_split_h
        };

        if split_horizontal {
            let split = rng.gen_range(MIN_NODE_W..=(self.w - MIN_NODE_W));
            let mut left = BspNode::new(self.x, self.y, split, self.h);
            let mut right = BspNode::new(self.x + split, self.y, self.w - split, self.h);
            left.split(rng, depth + 1, max_depth);
            right.split(rng, depth + 1, max_depth);
            self.left = Some(Box::new(left));
            self.right = Some(Box::new(right));
        } else {
            let split = rng.gen_range(MIN_NODE_H..=(self.h - MIN_NODE_H));
            let mut left = BspNode::new(self.x, self.y, self.w, split);
            let mut right = BspNode::new(self.x, self.y + split, self.w, self.h - split);
            left.split(rng, depth + 1, max_depth);
            right.split(rng, depth + 1, max_depth);
            self.left = Some(Box::new(left));
            self.right = Some(Box::new(right));
        }
    }

    fn create_rooms(&mut self, rng: &mut impl Rng) {
        if let (Some(ref mut left), Some(ref mut right)) = (&mut self.left, &mut self.right) {
            left.create_rooms(rng);
            right.create_rooms(rng);
        } else {
            // Leaf node — create a room
            let room_w = rng.gen_range(MIN_ROOM_W..=MAX_ROOM_W.min(self.w - 2));
            let room_h = rng.gen_range(MIN_ROOM_H..=MAX_ROOM_H.min(self.h - 2));
            let room_x = self.x + rng.gen_range(1..=(self.w - room_w - 1).max(1));
            let room_y = self.y + rng.gen_range(1..=(self.h - room_h - 1).max(1));
            self.room = Some(Room::new(room_x, room_y, room_w, room_h));
        }
    }

    fn get_room(&self) -> Option<&Room> {
        if self.room.is_some() {
            return self.room.as_ref();
        }
        // Recurse to find a room in children
        if let Some(ref left) = self.left {
            if let Some(room) = left.get_room() {
                return Some(room);
            }
        }
        if let Some(ref right) = self.right {
            if let Some(room) = right.get_room() {
                return Some(room);
            }
        }
        None
    }

    fn collect_rooms(&self, rooms: &mut Vec<Room>) {
        if let Some(ref room) = self.room {
            rooms.push(room.clone());
        }
        if let Some(ref left) = self.left {
            left.collect_rooms(rooms);
        }
        if let Some(ref right) = self.right {
            right.collect_rooms(rooms);
        }
    }

    fn create_corridors(&self, map: &mut Map, rng: &mut impl Rng) {
        if let (Some(ref left), Some(ref right)) = (&self.left, &self.right) {
            left.create_corridors(map, rng);
            right.create_corridors(map, rng);

            // Connect a room from left subtree to a room from right subtree
            if let (Some(left_room), Some(right_room)) = (left.get_room(), right.get_room()) {
                carve_l_corridor(map, left_room.center(), right_room.center(), rng);
            }
        }
    }
}

pub fn generate_bsp(rng: &mut impl Rng) -> Map {
    let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);
    let mut root = BspNode::new(0, 0, MAP_WIDTH as i32, MAP_HEIGHT as i32);

    // Split to get 5-12 leaves (depth 3-4 typically)
    root.split(rng, 0, 4);
    root.create_rooms(rng);

    // Carve rooms into map
    let mut rooms = Vec::new();
    root.collect_rooms(&mut rooms);

    for room in &rooms {
        carve_room(&mut map, room);
    }

    // Carve corridors
    root.create_corridors(&mut map, rng);

    map.rooms = rooms;
    map.refresh_blocked();
    map
}

fn carve_room(map: &mut Map, room: &Room) {
    for y in (room.y + 1)..(room.y + room.height - 1) {
        for x in (room.x + 1)..(room.x + room.width - 1) {
            map.set_tile(x, y, TileType::Floor);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn bsp_generates_connected_maps() {
        for seed in 0..100u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            let map = generate_bsp(&mut rng);

            assert!(!map.rooms.is_empty(), "seed {seed}: no rooms generated");
            assert!(
                map.rooms.len() >= 3,
                "seed {seed}: too few rooms ({})",
                map.rooms.len()
            );

            // Verify all rooms have floor tiles
            for room in &map.rooms {
                let cx = room.center().x;
                let cy = room.center().y;
                assert!(
                    map.in_bounds(cx, cy),
                    "seed {seed}: room center out of bounds"
                );
                assert_eq!(
                    map.get_tile(cx, cy),
                    TileType::Floor,
                    "seed {seed}: room center is not floor"
                );
            }

            // Flood fill connectivity from first room center
            let start = map.rooms[0].center();
            let reachable = flood_fill(&map, start.x, start.y);

            for (i, room) in map.rooms.iter().enumerate() {
                let c = room.center();
                assert!(
                    reachable.contains(&(c.x, c.y)),
                    "seed {seed}: room {i} not reachable from room 0"
                );
            }
        }
    }

    fn flood_fill(map: &Map, start_x: i32, start_y: i32) -> std::collections::HashSet<(i32, i32)> {
        let mut visited = std::collections::HashSet::new();
        let mut queue = std::collections::VecDeque::new();
        queue.push_back((start_x, start_y));
        visited.insert((start_x, start_y));

        while let Some((x, y)) = queue.pop_front() {
            for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
                let nx = x + dx;
                let ny = y + dy;
                if map.in_bounds(nx, ny)
                    && !visited.contains(&(nx, ny))
                    && map.get_tile(nx, ny).is_walkable()
                {
                    visited.insert((nx, ny));
                    queue.push_back((nx, ny));
                }
            }
        }
        visited
    }
}
