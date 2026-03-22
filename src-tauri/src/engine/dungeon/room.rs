use rand::Rng;

use crate::engine::entity::Position;
use crate::engine::map::{Room, RoomType};

pub fn assign_room_types(
    rooms: &mut Vec<Room>,
    rng: &mut impl Rng,
    is_boss_floor: bool,
    floor: u32,
) {
    if rooms.is_empty() {
        return;
    }

    // Reset all to Normal
    for room in rooms.iter_mut() {
        room.room_type = RoomType::Normal;
    }

    let map_center = Position::new(40, 25);

    // Start room: closest to center
    let start_idx = rooms
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            let da = a.center().distance_to(&map_center);
            let db = b.center().distance_to(&map_center);
            da.partial_cmp(&db).unwrap()
        })
        .map(|(i, _)| i)
        .unwrap_or(0);
    rooms[start_idx].room_type = RoomType::Start;

    let start_center = rooms[start_idx].center();

    // Sort remaining by distance from start
    let mut by_distance: Vec<(usize, f64)> = rooms
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != start_idx)
        .map(|(i, r)| (i, r.center().distance_to(&start_center)))
        .collect();
    by_distance.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());

    // Stairs: furthest from start
    if let Some(&(stairs_idx, _)) = by_distance.first() {
        // On boss floors, boss room is the second-furthest and stairs are still furthest
        if is_boss_floor && by_distance.len() >= 2 {
            rooms[stairs_idx].room_type = RoomType::Normal; // stairs room stays normal, stairs placed on tile
            let boss_idx = by_distance[1].0;
            rooms[boss_idx].room_type = RoomType::Boss;
        }
        // Mark stairs location (actual stair tiles placed separately)
    }

    // Shop room on floors 2, 5, 8 and every 3 floors in endless
    let is_shop_floor = floor == 2 || floor == 5 || floor == 8 || (floor > 10 && floor % 3 == 0);
    if is_shop_floor {
        // Find a middle-distance Normal room for the shop
        let skip = if is_boss_floor { 2 } else { 1 };
        let mid = by_distance.len() / 2;
        for &(idx, _) in by_distance.iter().skip(skip.max(mid)).take(3) {
            if rooms[idx].room_type == RoomType::Normal {
                rooms[idx].room_type = RoomType::Shop;
                break;
            }
        }
    }

    // Assign special types to remaining rooms
    for &(idx, _) in by_distance.iter().skip(if is_boss_floor { 2 } else { 1 }) {
        if rooms[idx].room_type != RoomType::Normal {
            continue;
        }
        let roll: f32 = rng.gen();
        rooms[idx].room_type = if roll < 0.25 {
            RoomType::Treasure
        } else if roll < 0.40 {
            RoomType::Shrine
        } else if roll < 0.50 {
            RoomType::Library
        } else if roll < 0.60 {
            RoomType::Armory
        } else {
            RoomType::Normal
        };
    }
}

pub fn get_stairs_room_idx(rooms: &[Room]) -> Option<usize> {
    let start_idx = rooms.iter().position(|r| r.room_type == RoomType::Start)?;
    let start_center = rooms[start_idx].center();

    rooms
        .iter()
        .enumerate()
        .filter(|(i, _)| *i != start_idx)
        .max_by(|(_, a), (_, b)| {
            let da = a.center().distance_to(&start_center);
            let db = b.center().distance_to(&start_center);
            da.partial_cmp(&db).unwrap()
        })
        .map(|(i, _)| i)
}
