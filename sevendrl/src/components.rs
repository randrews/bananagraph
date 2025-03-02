use cgmath::Vector2;
use hecs::World;
use bananagraph::{DrawingContext, Sprite};

#[derive(Copy, Clone, Debug)]
pub struct OnMap {
    pub location: Vector2<i32>,
    pub sprite: Sprite
}

impl OnMap {
    pub fn system(world: &World) -> Vec<Sprite> {
        let dc = DrawingContext::new((960.0 / 2.0, 540.0 / 2.0));
        let mut sprites = vec![];

        for (_, (on_map,)) in world.query::<(&OnMap,)>().iter() {
            let OnMap { location, sprite } = on_map;
            sprites.push(dc.place(*sprite, (location.x as f32 * 16.0, location.y as f32 * 16.0)));
        }
        sprites
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Player;
