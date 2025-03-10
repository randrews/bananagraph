use cgmath::Point2;
use hecs::{Entity, World};
use bananagraph::Sprite;
use grid::{Grid, VecGrid};
use crate::components::OnMap;

/// Walls are immovable terrain that can't be walked through
#[derive(Copy, Clone, Debug)]
pub struct Wall;

/// Terrain is anything that's determined solely from the map generation: walls + floors + doors +
/// water + etc.
#[derive(Copy, Clone, Debug)]
pub struct Terrain;

/// Given a VecGrid<char> of the map, recreates all terrain in the world (after despawning
/// the preexisting Terrain entities). Looks for a VecGrid containing the following chars:
/// - `#` for a wall
/// - `.` for empty floor
/// - `+` for a door
pub fn recreate_terrain(map: VecGrid<char>, world: &mut World) {
    // Despawn everything that's a Terrain
    let terrain: Vec<Entity> = world.query::<(&Terrain,)>().iter().map(|x| x.0).collect();
    for e in terrain {
        world.despawn(e).unwrap()
    }

    // Go over the map creating things
    for (n, ch) in map.iter().enumerate() {
        let location = map.coord(n);
        match ch {
            '#' => {
                let sprite = wall_tile(map.neighbors_equal(location, '#'));
                world.spawn((Wall, Terrain, OnMap { location, sprite }));
            }
            '.' => {
                let sprite = Sprite::new((144, 128), (16, 16));
                world.spawn((Terrain, OnMap { location, sprite }));
            },
            _ => {}
        }
    }
}

pub fn wall_tile(neighbors: (bool, bool, bool, bool)) -> Sprite {
    // north, south, east, west
    let mut origin = match neighbors {
        (false, false, false, false) => (5, 1),
        (true, true, true, true) => (5, 0),

        (true, true, false, false) => (4, 1),
        (false, false, true, true) => (3, 0),

        (true, false, false, false) => (0, 2),
        (false, false, true, false) => (2, 2),
        (false, true, false, false) => (1, 2),
        (false, false, false, true) => (3, 1),

        (false, true, true, true) => (0, 0),
        (true, true, false, true) => (1, 0),
        (true, false, true, true) => (1, 1),
        (true, true, true, false) => (0, 1),

        (false, true, true, false) => (2, 0),
        (false, true, false, true) => (4, 0),
        (true, false, true, false) => (2, 1),
        (true, false, false, true) => (4, 2),
    };

    origin.1 += 3;
    Sprite::new(Point2::from(origin) * 16, (16, 16))
}
