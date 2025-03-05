use cgmath::Vector2;
use hecs::World;
use bananagraph::{DrawingContext, Sprite, Typeface};
use crate::sprites::UiFrame;

#[derive(Clone)]
pub struct Inventory {
}

impl Inventory {
    pub fn system(world: &World, typeface: &Typeface) -> Vec<Sprite> {
        let dc = DrawingContext::new((960.0 / 2.0, 544.0 / 2.0));
        let mut sprites = UiFrame::draw_frame(dc, (0.0, 0.0), (9, 13), 0.9);
        sprites
    }
}