use cgmath::Vector2;
use hecs::World;
use bananagraph::Sprite;
use crate::components::OnMap;
use crate::terrain::{Opaque, Solid};

/// Doors can be open or closed
#[derive(Copy, Clone, Debug)]
pub struct Door {
    pub open: bool
}

impl Door {
    pub fn try_bump(world: &mut World, new_loc: Vector2<i32>) -> bool {
        let mut can_move = true;
        let mut opened = vec![];
        for (ent, (door, on_map)) in world.query_mut::<(&mut Door, &mut OnMap)>() {
            if on_map.location != new_loc || door.open { continue }
            on_map.sprite = Sprite::new((96, 16), (16, 16));
            door.open = true;
            opened.push(ent);
            can_move = false;
        }

        // If we opened anything then it's no longer opaque or solid
        for e in opened {
            let _ = world.remove::<(Opaque,Solid)>(e);
        }

        can_move
    }
}
