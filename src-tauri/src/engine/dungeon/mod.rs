pub mod bsp;
pub mod cellular;
pub mod corridor;
pub mod placement;
pub mod room;

use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::engine::map::{Map, TileType};

use self::room::{assign_room_types, get_stairs_room_idx};

const FLOOR_SEED_MULTIPLIER: u64 = 0x9E3779B97F4A7C15;

pub fn generate_floor(seed: u64, floor: u32) -> Map {
    let floor_seed = seed.wrapping_add((floor as u64).wrapping_mul(FLOOR_SEED_MULTIPLIER));
    let mut rng = StdRng::seed_from_u64(floor_seed);

    let is_boss_floor = matches!(floor, 3 | 6 | 10) || (floor > 10 && floor % 5 == 0);

    let mut map = match floor {
        1..=3 => bsp::generate_bsp(&mut rng),
        4 => {
            // 60% BSP, 40% cellular
            if rng.gen::<f64>() < 0.6 {
                bsp::generate_bsp(&mut rng)
            } else {
                cellular::generate_cellular(&mut rng)
            }
        }
        5 => {
            // 40% BSP, 60% cellular
            if rng.gen::<f64>() < 0.4 {
                bsp::generate_bsp(&mut rng)
            } else {
                cellular::generate_cellular(&mut rng)
            }
        }
        6 => {
            // 20% BSP, 80% cellular (end of mixed range)
            if rng.gen::<f64>() < 0.2 {
                bsp::generate_bsp(&mut rng)
            } else {
                cellular::generate_cellular(&mut rng)
            }
        }
        7..=9 => cellular::generate_cellular(&mut rng),
        10 => generate_arena(&mut rng),
        _ => {
            // Endless mode: cycle between types
            let cycle = (floor - 11) % 3;
            match cycle {
                0 => bsp::generate_bsp(&mut rng),
                1 => cellular::generate_cellular(&mut rng),
                _ => generate_arena(&mut rng),
            }
        }
    };

    // Assign room types
    assign_room_types(&mut map.rooms, &mut rng, is_boss_floor, floor);

    // 50% chance to carve a secret room adjacent to a Normal room
    if !is_boss_floor && rng.gen::<f32>() < 0.5 {
        carve_secret_room(&mut map, &mut rng);
    }

    // Place stairs in the furthest room from start
    if let Some(stairs_idx) = get_stairs_room_idx(&map.rooms) {
        let center = map.rooms[stairs_idx].center();
        // Find nearest floor tile (center may be wall in cellular automata rooms)
        let pos = find_nearest_floor(&map, center);
        map.set_tile(pos.x, pos.y, TileType::DownStairs);
    }

    // Place up stairs in start room (except floor 1)
    if floor > 1 {
        if let Some(start_room) = map
            .rooms
            .iter()
            .find(|r| r.room_type == crate::engine::map::RoomType::Start)
        {
            let center = start_room.center();
            // Offset by 1 so up/down stairs don't collide; use find_nearest_floor
            // to handle cellular automata rooms where center+1 may be a wall
            let offset = crate::engine::entity::Position::new(center.x + 1, center.y);
            let up_pos = find_nearest_floor(&map, offset);
            // Don't place on top of existing down stairs
            if map.get_tile(up_pos.x, up_pos.y) != TileType::DownStairs {
                map.set_tile(up_pos.x, up_pos.y, TileType::UpStairs);
            }
        }
    }

    map
}

/// Find the nearest floor tile to a position (BFS spiral outward).
fn find_nearest_floor(
    map: &Map,
    start: crate::engine::entity::Position,
) -> crate::engine::entity::Position {
    use crate::engine::entity::Position;
    if map.in_bounds(start.x, start.y) && map.get_tile(start.x, start.y) == TileType::Floor {
        return start;
    }
    for radius in 1i32..20 {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() != radius && dy.abs() != radius {
                    continue; // Only check the ring at this radius
                }
                let x = start.x + dx;
                let y = start.y + dy;
                if map.in_bounds(x, y) && map.get_tile(x, y) == TileType::Floor {
                    return Position::new(x, y);
                }
            }
        }
    }
    start // fallback: shouldn't happen on a valid map
}

/// Carve a 3x3 secret room behind a wall adjacent to a Normal room.
/// The connecting wall tile becomes SecretWall (bumping reveals it).
fn carve_secret_room(map: &mut Map, rng: &mut impl Rng) {
    use crate::engine::map::{Room, RoomType, MAP_HEIGHT, MAP_WIDTH};

    // Find Normal rooms to attach secret room to
    let normal_indices: Vec<usize> = map
        .rooms
        .iter()
        .enumerate()
        .filter(|(_, r)| r.room_type == RoomType::Normal)
        .map(|(i, _)| i)
        .collect();

    if normal_indices.is_empty() {
        return;
    }

    let room_idx = normal_indices[rng.gen_range(0..normal_indices.len())];
    let room = &map.rooms[room_idx];

    // Try each wall of the room to find space for a 3x3 secret chamber
    // Directions: 0=north, 1=south, 2=west, 3=east
    let mut dirs: Vec<u8> = vec![0, 1, 2, 3];
    // Shuffle directions
    for i in (1..dirs.len()).rev() {
        let j = rng.gen_range(0..=i);
        dirs.swap(i, j);
    }

    for dir in dirs {
        // Pick a connection point on the room's wall and compute secret room position
        let (secret_x, secret_y, connect_x, connect_y) = match dir {
            0 => {
                // North wall: secret room is above
                let cx = room.x + 1 + rng.gen_range(0..(room.width - 2).max(1));
                let cy = room.y - 1; // wall tile
                (cx - 1, room.y - 4, cx, cy) // 3x3 room starts 3 tiles above wall
            }
            1 => {
                // South wall: secret room is below
                let cx = room.x + 1 + rng.gen_range(0..(room.width - 2).max(1));
                let cy = room.y + room.height; // wall tile
                (cx - 1, room.y + room.height + 1, cx, cy)
            }
            2 => {
                // West wall: secret room is left
                let cy = room.y + 1 + rng.gen_range(0..(room.height - 2).max(1));
                let cx = room.x - 1; // wall tile
                (room.x - 4, cy - 1, cx, cy)
            }
            _ => {
                // East wall: secret room is right
                let cy = room.y + 1 + rng.gen_range(0..(room.height - 2).max(1));
                let cx = room.x + room.width; // wall tile
                (room.x + room.width + 1, cy - 1, cx, cy)
            }
        };

        // Check bounds: secret room (3x3) must fit within map with 1-tile border
        if secret_x < 1
            || secret_y < 1
            || secret_x + 3 >= MAP_WIDTH as i32 - 1
            || secret_y + 3 >= MAP_HEIGHT as i32 - 1
        {
            continue;
        }

        // Check that the secret room area is all walls (don't carve into existing rooms)
        let mut area_clear = true;
        for dy in 0..3 {
            for dx in 0..3 {
                let tile = map.get_tile(secret_x + dx, secret_y + dy);
                if tile != TileType::Wall {
                    area_clear = false;
                    break;
                }
            }
            if !area_clear {
                break;
            }
        }
        if !area_clear {
            continue;
        }

        // Also check that the connection tile is currently a Wall
        if map.get_tile(connect_x, connect_y) != TileType::Wall {
            continue;
        }

        // Carve the 3x3 secret room interior
        for dy in 0..3 {
            for dx in 0..3 {
                map.set_tile(secret_x + dx, secret_y + dy, TileType::Floor);
            }
        }

        // Set the connection tile to SecretWall
        map.set_tile(connect_x, connect_y, TileType::SecretWall);

        // Add the secret room to the room list (for item placement)
        let mut secret_room = Room::new(secret_x, secret_y, 3, 3);
        secret_room.room_type = RoomType::Treasure; // Rare loot
        map.rooms.push(secret_room);

        map.refresh_blocked();
        return; // Only one secret room per floor
    }
}

fn generate_arena(_rng: &mut impl rand::Rng) -> Map {
    use crate::engine::map::{Map, Room, MAP_HEIGHT, MAP_WIDTH};

    let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);

    // Large central arena
    let arena_x = 20;
    let arena_y = 10;
    let arena_w = 40;
    let arena_h = 30;

    // Carve arena
    for y in arena_y..(arena_y + arena_h) {
        for x in arena_x..(arena_x + arena_w) {
            map.set_tile(x, y, TileType::Floor);
        }
    }

    // Four corner rooms
    let corner_rooms = [
        (5, 5, 10, 8),
        (65, 5, 10, 8),
        (5, 37, 10, 8),
        (65, 37, 10, 8),
    ];

    for &(rx, ry, rw, rh) in &corner_rooms {
        for y in (ry + 1)..(ry + rh - 1) {
            for x in (rx + 1)..(rx + rw - 1) {
                map.set_tile(x, y, TileType::Floor);
            }
        }
    }

    // Corridors connecting corner rooms to arena
    // Top corridors at y=10 (inside both top room interiors and arena top edge)
    for x in 14..=20 {
        map.set_tile(x, 10, TileType::Floor); // top-left room → arena
    }
    for x in 59..=66 {
        map.set_tile(x, 10, TileType::Floor); // arena → top-right room
    }
    // Bottom corridors at y=39 (inside both bottom room interiors and arena bottom)
    for x in 14..=20 {
        map.set_tile(x, 39, TileType::Floor); // bottom-left room → arena
    }
    for x in 59..=66 {
        map.set_tile(x, 39, TileType::Floor); // arena → bottom-right room
    }

    // Build rooms list
    let mut rooms = vec![Room::new(arena_x, arena_y, arena_w, arena_h)];
    for &(rx, ry, rw, rh) in &corner_rooms {
        rooms.push(Room::new(rx, ry, rw, rh));
    }

    map.rooms = rooms;
    map.refresh_blocked();
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::map::RoomType;

    #[test]
    fn floor_generation_produces_valid_maps() {
        let seed = 12345u64;
        for floor in 1..=12 {
            let map = generate_floor(seed, floor);

            assert!(!map.rooms.is_empty(), "floor {floor}: no rooms");

            // Has a start room
            assert!(
                map.rooms.iter().any(|r| r.room_type == RoomType::Start),
                "floor {floor}: no start room"
            );

            // Has down stairs
            let has_stairs = map.tiles.iter().any(|t| *t == TileType::DownStairs);
            assert!(has_stairs, "floor {floor}: no down stairs");
        }
    }

    #[test]
    fn seed_determinism() {
        let seed = 42u64;
        let map1 = generate_floor(seed, 1);
        let map2 = generate_floor(seed, 1);
        assert_eq!(map1.tiles, map2.tiles);
        assert_eq!(map1.rooms.len(), map2.rooms.len());
    }

    #[test]
    fn up_stairs_placed_on_floor_2_plus() {
        let seed = 42u64;
        for floor in 2..=12 {
            let map = generate_floor(seed, floor);
            let has_up_stairs = map.tiles.iter().any(|t| *t == TileType::UpStairs);
            assert!(has_up_stairs, "floor {floor}: no up stairs");
        }
    }

    #[test]
    fn endless_mode_generates_arenas() {
        let seed = 42u64;
        // Floor 13 should be arena (cycle 2: (13-11)%3 = 2)
        let map = generate_floor(seed, 13);
        assert!(!map.rooms.is_empty(), "floor 13: no rooms");
        // Arena has a large room (40x30)
        let has_large_room = map.rooms.iter().any(|r| r.width >= 30 && r.height >= 20);
        assert!(has_large_room, "floor 13: expected arena with large room");
    }
}
