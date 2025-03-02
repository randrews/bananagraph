use cgmath::Vector2;
use hecs::World;
use bananagraph::{DrawingContext, Sprite};

#[derive(Copy, Clone, Debug)]
pub struct OnMap {
    pub location: Vector2<i32>,
    pub sprite: Sprite
}

fn player_loc(world: &World) -> Vector2<i32> {
    world.query::<(&Player, &OnMap)>().into_iter().next().map(|(_, (_, onmap))| onmap.location.clone()).unwrap()
}

impl OnMap {
    pub fn system(world: &World) -> Vec<Sprite> {
        let dc = DrawingContext::new((960.0 / 2.0, 540.0 / 2.0));
        let mut sprites = vec![];

        // We have a 480 x 270 "pixel" area, which with 16x16 tiles means 30x16.875 tiles
        // We want an odd-numbered square centered on the player, with room on the bottom and left
        // for status bar and inventory.
        // - Let's say that height-wise we'll be 13 tiles tall, so 208 px, leaving
        // (540 / 2) - (13 * 16) = 62 px for status bar
        // Width we'll say 21 wide, leaving (960 / 2) - (21 * 16) = 144 for inventory
        let player_loc = player_loc(world);
        let topleft = Vector2::new(player_loc.x - 10, player_loc.y - 6);
        let size = Vector2::new(21, 13);
        let inv_width = (960.0 / 2.0) - (21.0 * 16.0);

        for (_, (on_map,)) in world.query::<(&OnMap,)>().iter() {
            let OnMap { location, sprite } = on_map;
            // Skip things not in the region
            if location.x < topleft.x || location.y < topleft.y || location.x >= topleft.x + size.x || location.y >= topleft.y + size.y {
                continue
            }

            let local_coords = Vector2::new(
                (location.x - topleft.x) as f32 * 16.0 + inv_width,
                (location.y - topleft.y) as f32 * 16.0
            );
            sprites.push(dc.place(*sprite, local_coords));
        }
        sprites
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Player;
