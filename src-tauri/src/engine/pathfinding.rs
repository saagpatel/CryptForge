use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};

use serde::{Deserialize, Serialize};

use super::entity::Position;
use super::map::Map;

pub const UNREACHABLE: i32 = i32::MAX;

#[derive(Serialize, Deserialize)]
pub struct DijkstraMap {
    pub width: usize,
    pub height: usize,
    pub values: Vec<i32>,
}

impl DijkstraMap {
    pub fn compute(map: &Map, sources: &[Position]) -> Self {
        let size = map.width * map.height;
        let mut values = vec![UNREACHABLE; size];
        let mut queue = VecDeque::new();

        for source in sources {
            let idx = map.idx(source.x, source.y);
            values[idx] = 0;
            queue.push_back(*source);
        }

        while let Some(pos) = queue.pop_front() {
            let current_dist = values[map.idx(pos.x, pos.y)];

            for (dx, dy) in &[
                (-1, 0),
                (1, 0),
                (0, -1),
                (0, 1),
                (-1, -1),
                (-1, 1),
                (1, -1),
                (1, 1),
            ] {
                let nx = pos.x + dx;
                let ny = pos.y + dy;
                if !map.in_bounds(nx, ny) {
                    continue;
                }
                let nidx = map.idx(nx, ny);
                if !map.tiles[nidx].is_walkable() {
                    continue;
                }
                let new_dist = current_dist + 1;
                if new_dist < values[nidx] {
                    values[nidx] = new_dist;
                    queue.push_back(Position::new(nx, ny));
                }
            }
        }

        DijkstraMap {
            width: map.width,
            height: map.height,
            values,
        }
    }

    pub fn get(&self, x: i32, y: i32) -> i32 {
        if x < 0 || y < 0 || x as usize >= self.width || y as usize >= self.height {
            return UNREACHABLE;
        }
        self.values[y as usize * self.width + x as usize]
    }

    pub fn best_neighbor(&self, pos: Position, map: &Map) -> Option<Position> {
        let mut best_pos = None;
        let mut best_val = self.get(pos.x, pos.y);

        for (dx, dy) in &[
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1),
            (-1, 1),
            (1, -1),
            (1, 1),
        ] {
            let nx = pos.x + dx;
            let ny = pos.y + dy;
            if map.in_bounds(nx, ny) && map.tiles[map.idx(nx, ny)].is_walkable() {
                let val = self.get(nx, ny);
                if val < best_val {
                    best_val = val;
                    best_pos = Some(Position::new(nx, ny));
                }
            }
        }
        best_pos
    }

    pub fn flee_neighbor(&self, pos: Position, map: &Map) -> Option<Position> {
        let mut best_pos = None;
        let mut best_val = self.get(pos.x, pos.y);

        for (dx, dy) in &[
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1),
            (-1, 1),
            (1, -1),
            (1, 1),
        ] {
            let nx = pos.x + dx;
            let ny = pos.y + dy;
            if map.in_bounds(nx, ny) && map.tiles[map.idx(nx, ny)].is_walkable() {
                let val = self.get(nx, ny);
                if val > best_val && val != UNREACHABLE {
                    best_val = val;
                    best_pos = Some(Position::new(nx, ny));
                }
            }
        }
        best_pos
    }
}

// A* pathfinding

#[derive(Debug, Clone, Eq, PartialEq)]
struct AStarNode {
    pos: Position,
    cost: i32,
    estimated_total: i32,
}

impl Ord for AStarNode {
    fn cmp(&self, other: &Self) -> Ordering {
        other.estimated_total.cmp(&self.estimated_total)
    }
}

impl PartialOrd for AStarNode {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

pub fn astar(map: &Map, start: Position, goal: Position) -> Option<Vec<Position>> {
    let size = map.width * map.height;
    let mut costs = vec![UNREACHABLE; size];
    let mut came_from = vec![None::<Position>; size];
    let mut heap = BinaryHeap::new();

    let start_idx = map.idx(start.x, start.y);
    costs[start_idx] = 0;
    heap.push(AStarNode {
        pos: start,
        cost: 0,
        estimated_total: chebyshev(start, goal),
    });

    while let Some(node) = heap.pop() {
        if node.pos == goal {
            // Reconstruct path
            let mut path = Vec::new();
            let mut current = goal;
            while current != start {
                path.push(current);
                let idx = map.idx(current.x, current.y);
                current = came_from[idx]?;
            }
            path.reverse();
            return Some(path);
        }

        if node.cost > costs[map.idx(node.pos.x, node.pos.y)] {
            continue;
        }

        for (dx, dy) in &[
            (-1, 0),
            (1, 0),
            (0, -1),
            (0, 1),
            (-1, -1),
            (-1, 1),
            (1, -1),
            (1, 1),
        ] {
            let nx = node.pos.x + dx;
            let ny = node.pos.y + dy;

            if !map.in_bounds(nx, ny) {
                continue;
            }
            let nidx = map.idx(nx, ny);
            if !map.tiles[nidx].is_walkable() {
                continue;
            }

            let new_cost = node.cost + 1;
            if new_cost < costs[nidx] {
                costs[nidx] = new_cost;
                came_from[nidx] = Some(node.pos);
                let npos = Position::new(nx, ny);
                heap.push(AStarNode {
                    pos: npos,
                    cost: new_cost,
                    estimated_total: new_cost + chebyshev(npos, goal),
                });
            }
        }
    }
    None
}

fn chebyshev(a: Position, b: Position) -> i32 {
    (a.x - b.x).abs().max((a.y - b.y).abs())
}

pub fn has_line_of_sight(map: &Map, from: Position, to: Position) -> bool {
    // Bresenham's line algorithm
    let mut x = from.x;
    let mut y = from.y;
    let dx = (to.x - from.x).abs();
    let dy = -(to.y - from.y).abs();
    let sx = if from.x < to.x { 1 } else { -1 };
    let sy = if from.y < to.y { 1 } else { -1 };
    let mut err = dx + dy;

    loop {
        if x == to.x && y == to.y {
            return true;
        }
        // Check if this tile blocks sight (skip origin)
        if (x != from.x || y != from.y) && map.is_opaque(x, y) {
            return false;
        }
        let e2 = 2 * err;
        if e2 >= dy {
            err += dy;
            x += sx;
        }
        if e2 <= dx {
            err += dx;
            y += sy;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::engine::map::TileType;

    fn make_test_map() -> Map {
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
    fn dijkstra_source_is_zero() {
        let map = make_test_map();
        let source = Position::new(10, 10);
        let dmap = DijkstraMap::compute(&map, &[source]);
        assert_eq!(dmap.get(10, 10), 0);
    }

    #[test]
    fn dijkstra_adjacent_is_one() {
        let map = make_test_map();
        let source = Position::new(10, 10);
        let dmap = DijkstraMap::compute(&map, &[source]);
        assert_eq!(dmap.get(11, 10), 1);
        assert_eq!(dmap.get(10, 11), 1);
    }

    #[test]
    fn dijkstra_walls_are_unreachable() {
        let map = make_test_map();
        let source = Position::new(10, 10);
        let dmap = DijkstraMap::compute(&map, &[source]);
        assert_eq!(dmap.get(0, 0), UNREACHABLE);
    }

    #[test]
    fn astar_finds_path() {
        let map = make_test_map();
        let path = astar(&map, Position::new(1, 1), Position::new(18, 18));
        assert!(path.is_some());
        let path = path.unwrap();
        assert_eq!(*path.last().unwrap(), Position::new(18, 18));
    }

    #[test]
    fn astar_no_path_through_wall() {
        let mut map = make_test_map();
        // Wall off the bottom half
        for x in 0..20 {
            map.set_tile(x, 10, TileType::Wall);
        }
        let path = astar(&map, Position::new(5, 5), Position::new(5, 15));
        assert!(path.is_none());
    }

    #[test]
    fn line_of_sight_clear() {
        let map = make_test_map();
        assert!(has_line_of_sight(
            &map,
            Position::new(5, 5),
            Position::new(15, 5)
        ));
    }

    #[test]
    fn line_of_sight_blocked() {
        let mut map = make_test_map();
        map.set_tile(10, 5, TileType::Wall);
        assert!(!has_line_of_sight(
            &map,
            Position::new(5, 5),
            Position::new(15, 5)
        ));
    }
}
