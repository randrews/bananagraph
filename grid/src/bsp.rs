use cgmath::Vector2;
use tinyrand::Rand;
use crate::VecGrid;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum CellType { Clear, Wall, Door }

pub fn create_bsp_map(size: impl Into<Vector2<i32>>, max_depth: u32, rng: &mut impl Rand) -> VecGrid<CellType> {
    let size = size.into();
    let mut grid = VecGrid::new(size, Clear);
    use CellType::{Clear, Wall};

    // Cover the edges with walls
    for x in 0..size.x {
        grid[(x, 0)] = Wall;
        grid[(x, size.x - 1)] = Wall;
    }

    for y in 0..size.y {
        grid[(0, y)] = Wall;
        grid[(size.x - 1, y)] = Wall;
    }

    divide_h(rng, max_depth, grid, (1, 1), (size.x - 2, size.y - 2), 1)
}

/// Divide a room by cutting a horizontal wall across it
fn divide_h(rng: &mut impl Rand, max_depth: u32, mut grid: VecGrid<CellType>, top_left: impl Into<Vector2<i32>>, room_size: impl Into<Vector2<i32>>, depth: u32) -> VecGrid<CellType> {
    let (top_left, room_size) = (top_left.into(), room_size.into());

    // Randomly _don't_ split the room, for room size variety:
    // if rng.next_u32() % 50 < depth - 1 {
    //     return grid
    // }

    // Bail if we would have to split this into a room narrower than 2
    if room_size.y < 10 { return grid }
    // Try to place a splitting wall
    let wall_coord = 2 + (rng.next_u32() % (room_size.y as u32 - 4)) as i32 + top_left.y;
    // If the wall is on top of a door, nope:
    if grid[(top_left.x - 1, wall_coord)] == CellType::Door || grid[(top_left.x + room_size.x, wall_coord)] == CellType::Door {
        return divide_h(rng, max_depth, grid, top_left, room_size, depth)
    }
    for x in top_left.x..(top_left.x + room_size.x) {
        grid[(x, wall_coord)] = CellType::Wall
    }
    let door_coord = (rng.next_u32() % room_size.x as u32) as i32 + top_left.x;
    grid[(door_coord, wall_coord)] = CellType::Door;

    if depth < max_depth {
        grid = divide_v(rng, max_depth, grid, top_left, (room_size.x, wall_coord - top_left.y), depth + 1);
        divide_v(rng, max_depth, grid, (top_left.x, wall_coord + 1), (room_size.x, top_left.y + room_size.y - wall_coord - 1), depth + 1)
    } else {
        grid
    }
}

/// Divide a room by cutting a vertical wall across it
fn divide_v(rng: &mut impl Rand, max_depth: u32, mut grid: VecGrid<CellType>, top_left: impl Into<Vector2<i32>>, room_size: impl Into<Vector2<i32>>, depth: u32) -> VecGrid<CellType> {
    let (top_left, room_size) = (top_left.into(), room_size.into());

    // Randomly _don't_ split the room, for room size variety:
    // if rng.next_u32() % 50 < depth - 1 {
    //     return grid
    // }

    if room_size.x < 10 { return grid }
    let wall_coord = 2 + (rng.next_u32() % (room_size.x as u32 - 4)) as i32 + top_left.x;
    // If the wall is on top of a door, nope:
    if grid[(wall_coord, top_left.y - 1)] == CellType::Door || grid[(wall_coord, top_left.y + room_size.y)] == CellType::Door {
        return divide_v(rng, max_depth, grid, top_left, room_size, depth)
    }
    for y in top_left.y..(top_left.y + room_size.y) {
        grid[(wall_coord, y)] = CellType::Wall
    }
    let door_coord = (rng.next_u32() % room_size.y as u32) as i32 + top_left.y;
    grid[(wall_coord, door_coord)] = CellType::Door;

    if depth < max_depth {
        grid = divide_h(rng, max_depth, grid, top_left, (wall_coord - top_left.x, room_size.y), depth + 1);
        divide_h(rng, max_depth, grid, (wall_coord + 1, top_left.y), (top_left.x + room_size.x - wall_coord - 1, room_size.y), depth + 1)
    } else {
        grid
    }
}
