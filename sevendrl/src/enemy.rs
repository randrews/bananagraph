use hecs::{Entity, World};
use grid::VecGrid;
use crate::components::OnMap;
use crate::terrain::{Solid};

#[derive(Copy, Clone, Debug)]
pub struct Enemy {}

impl Enemy {
    pub fn system(world: &mut World) {

    }
}

// Okay we're gonna do some pathfinding. First we want to know about
// the three kinds of things we care about:
#[derive(Copy, Clone, Debug, PartialEq, Default)]
enum PFCellType {
    #[default]
    Clear,
    Wall,
    Enemy(Entity)
}

// Make a VecGrid of the enemy locations. TODO don't hard-code the map dimensions
fn enemies_map(world: &World) -> VecGrid<PFCellType> {
    let mut map = VecGrid::new((64, 64), PFCellType::Clear);

    for (ent, (solid, enemy, onmap)) in world.query::<(Option<&Solid>, Option<&Enemy>, &OnMap)>().iter() {
        map[onmap.location] = if solid.is_some() {
            PFCellType::Wall
        } else if enemy.is_some() {
            PFCellType::Enemy(ent)
        } else {
            PFCellType::Clear
        }
    }
    map
}