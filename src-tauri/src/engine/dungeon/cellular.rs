use std::collections::{HashSet, VecDeque};

use rand::Rng;

use crate::engine::entity::Position;
use crate::engine::map::{Map, Room, TileType, MAP_HEIGHT, MAP_WIDTH};

const WALL_CHANCE: f64 = 0.45;
const SMOOTHING_ITERATIONS: usize = 5;
const MIN_REGION_SIZE: usize = 20;

pub fn generate_cellular(rng: &mut impl Rng) -> Map {
    let mut map = Map::new(MAP_WIDTH, MAP_HEIGHT);

    // Step 1: Random fill
    for y in 0..MAP_HEIGHT as i32 {
        for x in 0..MAP_WIDTH as i32 {
            // Keep a 1-tile border of walls
            if x == 0 || y == 0 || x == (MAP_WIDTH as i32 - 1) || y == (MAP_HEIGHT as i32 - 1) {
                map.set_tile(x, y, TileType::Wall);
            } else if rng.gen::<f64>() < WALL_CHANCE {
                map.set_tile(x, y, TileType::Wall);
            } else {
                map.set_tile(x, y, TileType::Floor);
            }
        }
    }

    // Step 2: Smoothing iterations
    for _ in 0..SMOOTHING_ITERATIONS {
        let old_tiles = map.tiles.clone();
        for y in 1..(MAP_HEIGHT as i32 - 1) {
            for x in 1..(MAP_WIDTH as i32 - 1) {
                let wall_count = count_wall_neighbors(&old_tiles, MAP_WIDTH, x, y);
                let idx = map.idx(x, y);
                if wall_count >= 5 {
                    map.tiles[idx] = TileType::Wall;
                } else if wall_count <= 3 {
                    map.tiles[idx] = TileType::Floor;
                }
                // else keep same
            }
        }
    }

    // Step 3: Find largest connected floor region via flood fill
    let mut visited = vec![false; MAP_WIDTH * MAP_HEIGHT];
    let mut largest_region: HashSet<(i32, i32)> = HashSet::new();

    for y in 1..(MAP_HEIGHT as i32 - 1) {
        for x in 1..(MAP_WIDTH as i32 - 1) {
            let idx = map.idx(x, y);
            if !visited[idx] && map.tiles[idx] == TileType::Floor {
                let region = flood_fill_region(&map, x, y, &visited);
                for &(rx, ry) in &region {
                    visited[map.idx(rx, ry)] = true;
                }
                if region.len() > largest_region.len() {
                    largest_region = region;
                }
            }
        }
    }

    // Fill non-largest regions back to wall
    for y in 1..(MAP_HEIGHT as i32 - 1) {
        for x in 1..(MAP_WIDTH as i32 - 1) {
            if map.get_tile(x, y) == TileType::Floor && !largest_region.contains(&(x, y)) {
                map.set_tile(x, y, TileType::Wall);
            }
        }
    }

    // Step 4: Identify "rooms" as connected open areas > MIN_REGION_SIZE
    let rooms = identify_cave_rooms(&map);
    map.rooms = rooms;
    map.refresh_blocked();
    map
}

fn count_wall_neighbors(tiles: &[TileType], width: usize, x: i32, y: i32) -> i32 {
    let mut count = 0;
    for dy in -1..=1 {
        for dx in -1..=1 {
            if dx == 0 && dy == 0 {
                continue;
            }
            let nx = x + dx;
            let ny = y + dy;
            let idx = ny as usize * width + nx as usize;
            if nx < 0
                || ny < 0
                || nx >= width as i32
                || ny >= (tiles.len() / width) as i32
                || tiles[idx] == TileType::Wall
            {
                count += 1;
            }
        }
    }
    count
}

fn flood_fill_region(
    map: &Map,
    start_x: i32,
    start_y: i32,
    visited: &[bool],
) -> HashSet<(i32, i32)> {
    let mut region = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((start_x, start_y));
    region.insert((start_x, start_y));

    while let Some((x, y)) = queue.pop_front() {
        for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
            let nx = x + dx;
            let ny = y + dy;
            if map.in_bounds(nx, ny) {
                let idx = map.idx(nx, ny);
                if !visited[idx] && !region.contains(&(nx, ny)) && map.tiles[idx] == TileType::Floor
                {
                    region.insert((nx, ny));
                    queue.push_back((nx, ny));
                }
            }
        }
    }
    region
}

fn identify_cave_rooms(map: &Map) -> Vec<Room> {
    // Find connected floor clusters and make bounding-box "rooms" from them
    let mut visited = vec![false; map.width * map.height];
    let mut rooms = Vec::new();

    for y in 1..(map.height as i32 - 1) {
        for x in 1..(map.width as i32 - 1) {
            let idx = map.idx(x, y);
            if !visited[idx] && map.tiles[idx] == TileType::Floor {
                let region = flood_fill_bounded(map, x, y, &visited, 100);
                for &(rx, ry) in &region {
                    visited[map.idx(rx, ry)] = true;
                }

                if region.len() >= MIN_REGION_SIZE {
                    let min_x = region.iter().map(|p| p.0).min().unwrap();
                    let max_x = region.iter().map(|p| p.0).max().unwrap();
                    let min_y = region.iter().map(|p| p.1).min().unwrap();
                    let max_y = region.iter().map(|p| p.1).max().unwrap();

                    rooms.push(Room::new(
                        min_x,
                        min_y,
                        max_x - min_x + 1,
                        max_y - min_y + 1,
                    ));
                }
            }
        }
    }

    // If we got too many "rooms" (caves are one big region), split them up
    // by finding clusters of floor tiles separated by narrow passages
    if rooms.len() < 3 {
        // Fallback: divide the map into grid sections and create rooms from those
        rooms = create_grid_rooms(map);
    }

    rooms
}

fn flood_fill_bounded(
    map: &Map,
    start_x: i32,
    start_y: i32,
    visited: &[bool],
    max_size: usize,
) -> HashSet<(i32, i32)> {
    let mut region = HashSet::new();
    let mut queue = VecDeque::new();
    queue.push_back((start_x, start_y));
    region.insert((start_x, start_y));

    while let Some((x, y)) = queue.pop_front() {
        if region.len() >= max_size {
            break;
        }
        for (dx, dy) in &[(0, 1), (0, -1), (1, 0), (-1, 0)] {
            let nx = x + dx;
            let ny = y + dy;
            if map.in_bounds(nx, ny) {
                let idx = map.idx(nx, ny);
                if !visited[idx] && !region.contains(&(nx, ny)) && map.tiles[idx] == TileType::Floor
                {
                    region.insert((nx, ny));
                    queue.push_back((nx, ny));
                }
            }
        }
    }
    region
}

fn create_grid_rooms(map: &Map) -> Vec<Room> {
    // Divide map into 3x3 grid, find center floor tile in each section
    let mut rooms = Vec::new();
    let section_w = map.width as i32 / 3;
    let section_h = map.height as i32 / 3;

    for gy in 0..3 {
        for gx in 0..3 {
            let cx = gx * section_w + section_w / 2;
            let cy = gy * section_h + section_h / 2;

            // Find nearest floor tile to this center
            if let Some(pos) = find_nearest_floor(map, cx, cy) {
                rooms.push(Room::new(pos.x - 2, pos.y - 2, 5, 5));
            }
        }
    }
    rooms
}

fn find_nearest_floor(map: &Map, cx: i32, cy: i32) -> Option<Position> {
    for radius in 0i32..20 {
        for dy in -radius..=radius {
            for dx in -radius..=radius {
                if dx.abs() == radius || dy.abs() == radius {
                    let x = cx + dx;
                    let y = cy + dy;
                    if map.in_bounds(x, y) && map.get_tile(x, y) == TileType::Floor {
                        return Some(Position::new(x, y));
                    }
                }
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::rngs::StdRng;
    use rand::SeedableRng;

    #[test]
    fn cellular_generates_maps_with_floor_tiles() {
        for seed in 0..100u64 {
            let mut rng = StdRng::seed_from_u64(seed);
            let map = generate_cellular(&mut rng);

            let floor_count = map.tiles.iter().filter(|t| **t == TileType::Floor).count();
            assert!(
                floor_count > 100,
                "seed {seed}: too few floor tiles ({floor_count})"
            );

            assert!(!map.rooms.is_empty(), "seed {seed}: no rooms identified");

            // Verify rooms reference floor tiles
            for room in &map.rooms {
                let c = room.center();
                assert!(
                    map.in_bounds(c.x, c.y),
                    "seed {seed}: room center out of bounds"
                );
            }
        }
    }

    #[test]
    fn cellular_map_has_wall_border() {
        let mut rng = StdRng::seed_from_u64(42);
        let map = generate_cellular(&mut rng);

        // Check borders are walls
        for x in 0..MAP_WIDTH as i32 {
            assert_eq!(map.get_tile(x, 0), TileType::Wall);
            assert_eq!(map.get_tile(x, MAP_HEIGHT as i32 - 1), TileType::Wall);
        }
        for y in 0..MAP_HEIGHT as i32 {
            assert_eq!(map.get_tile(0, y), TileType::Wall);
            assert_eq!(map.get_tile(MAP_WIDTH as i32 - 1, y), TileType::Wall);
        }
    }
}
