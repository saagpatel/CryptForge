use std::collections::HashSet;

use super::entity::Position;
use super::map::Map;

// Multipliers for each octant to transform (row, col) into (dx, dy)
static OCTANT_MUL: [[i32; 4]; 8] = [
    [1, 0, 0, 1],
    [0, 1, 1, 0],
    [0, -1, 1, 0],
    [-1, 0, 0, 1],
    [-1, 0, 0, -1],
    [0, -1, -1, 0],
    [0, 1, -1, 0],
    [1, 0, 0, -1],
];

pub fn compute_fov(origin: Position, radius: i32, map: &Map) -> HashSet<Position> {
    let mut visible = HashSet::new();
    visible.insert(origin);

    for octant in 0..8 {
        cast_light(
            map,
            origin,
            radius,
            1,
            1.0,
            0.0,
            &OCTANT_MUL[octant],
            &mut visible,
        );
    }
    visible
}

fn cast_light(
    map: &Map,
    origin: Position,
    radius: i32,
    row: i32,
    mut start: f64,
    end: f64,
    mul: &[i32; 4],
    visible: &mut HashSet<Position>,
) {
    if start < end {
        return;
    }

    let radius_sq = radius * radius;
    let mut new_start = 0.0f64;

    for j in row..=radius {
        let mut dx = -j - 1;
        let dy = -j;
        let mut blocked = false;

        while dx <= 0 {
            dx += 1;

            let map_x = origin.x + dx * mul[0] + dy * mul[1];
            let map_y = origin.y + dx * mul[2] + dy * mul[3];

            if !map.in_bounds(map_x, map_y) {
                continue;
            }

            let l_slope = (dx as f64 - 0.5) / (dy as f64 + 0.5);
            let r_slope = (dx as f64 + 0.5) / (dy as f64 - 0.5);

            if start < r_slope {
                continue;
            }
            if end > l_slope {
                break;
            }

            // Check if within radius
            if dx * dx + dy * dy <= radius_sq {
                visible.insert(Position::new(map_x, map_y));
            }

            if blocked {
                if map.is_opaque(map_x, map_y) {
                    new_start = r_slope;
                } else {
                    blocked = false;
                    start = new_start;
                }
            } else if map.is_opaque(map_x, map_y) && j < radius {
                blocked = true;
                cast_light(map, origin, radius, j + 1, start, l_slope, mul, visible);
                new_start = r_slope;
            }
        }

        if blocked {
            break;
        }
    }
}

pub fn update_fov(map: &mut Map, origin: Position, radius: i32) -> HashSet<Position> {
    let visible = compute_fov(origin, radius, map);
    for pos in &visible {
        map.reveal(pos.x, pos.y);
    }
    visible
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::map::TileType;

    fn make_open_map() -> Map {
        let mut map = Map::new(20, 20);
        for y in 0..20i32 {
            for x in 0..20i32 {
                if x == 0 || y == 0 || x == 19 || y == 19 {
                    map.set_tile(x, y, TileType::Wall);
                } else {
                    map.set_tile(x, y, TileType::Floor);
                }
            }
        }
        map
    }

    #[test]
    fn origin_is_visible() {
        let map = make_open_map();
        let origin = Position::new(10, 10);
        let visible = compute_fov(origin, 8, &map);
        assert!(visible.contains(&origin));
    }

    #[test]
    fn adjacent_tiles_visible() {
        let map = make_open_map();
        let origin = Position::new(10, 10);
        let visible = compute_fov(origin, 8, &map);
        for dx in -1..=1 {
            for dy in -1..=1 {
                let pos = Position::new(10 + dx, 10 + dy);
                assert!(visible.contains(&pos), "Adjacent {:?} not visible", pos);
            }
        }
    }

    #[test]
    fn wall_blocks_vision() {
        let mut map = make_open_map();
        map.set_tile(12, 10, TileType::Wall);

        let origin = Position::new(10, 10);
        let visible = compute_fov(origin, 8, &map);

        assert!(visible.contains(&Position::new(12, 10)));
        assert!(!visible.contains(&Position::new(14, 10)));
    }

    #[test]
    fn respects_radius() {
        let map = make_open_map();
        let origin = Position::new(10, 10);
        let visible = compute_fov(origin, 3, &map);

        assert!(visible.contains(&Position::new(12, 10)));
        assert!(!visible.contains(&Position::new(16, 10)));
    }

    #[test]
    fn symmetry_in_open_space() {
        let map = make_open_map();
        let a = Position::new(8, 8);
        let b = Position::new(11, 11);

        let visible_a = compute_fov(a, 8, &map);
        let visible_b = compute_fov(b, 8, &map);

        if visible_a.contains(&b) {
            assert!(visible_b.contains(&a), "Symmetry violated");
        }
    }
}
