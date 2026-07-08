use rand::{Rng, RngExt};

use crate::engine::entity::Position;
use crate::engine::map::{Map, TileType};

pub fn carve_l_corridor(map: &mut Map, start: Position, end: Position, rng: &mut impl Rng) {
    // L-shaped corridor: go horizontal first, then vertical (or vice versa randomly)
    if rng.random_bool(0.5) {
        carve_horizontal(map, start.x, end.x, start.y);
        carve_vertical(map, start.y, end.y, end.x);
    } else {
        carve_vertical(map, start.y, end.y, start.x);
        carve_horizontal(map, start.x, end.x, end.y);
    }
}

fn carve_horizontal(map: &mut Map, x1: i32, x2: i32, y: i32) {
    let (min_x, max_x) = if x1 < x2 { (x1, x2) } else { (x2, x1) };
    for x in min_x..=max_x {
        if map.in_bounds(x, y) {
            let tile = map.get_tile(x, y);
            if tile == TileType::Wall {
                map.set_tile(x, y, TileType::Floor);
            }
        }
    }
}

fn carve_vertical(map: &mut Map, y1: i32, y2: i32, x: i32) {
    let (min_y, max_y) = if y1 < y2 { (y1, y2) } else { (y2, y1) };
    for y in min_y..=max_y {
        if map.in_bounds(x, y) {
            let tile = map.get_tile(x, y);
            if tile == TileType::Wall {
                map.set_tile(x, y, TileType::Floor);
            }
        }
    }
}
