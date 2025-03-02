use cgmath::Vector2;
use bananagraph::Sprite;
use crate::components::OnMap;
use crate::game_state::GameState;

/// Doors can be open or closed
#[derive(Copy, Clone, Debug)]
pub struct Door {
    pub open: bool
}

impl Door {
    pub fn try_move(game_state: &mut GameState, new_loc: Vector2<i32>) -> bool {
        let mut can_move = true;
        for (_, (door, on_map)) in game_state.world.query_mut::<(&mut Door, &mut OnMap)>() {
            if on_map.location != new_loc || door.open { continue }
            on_map.sprite = Sprite::new((96, 16), (16, 16));
            door.open = true;
            can_move = false;
        }
        can_move
    }
}
