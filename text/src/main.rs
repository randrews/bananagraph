mod typeface;

use bananagraph::{DrawingContext, GpuWrapper, IdBuffer, Sprite, WindowEventHandler};
use cgmath::{Point2, Vector2};
use crate::typeface::{Typeface, TypefaceBuilder};

#[derive(Default)]
struct GameState {
    typeface: Option<Typeface>
}

impl WindowEventHandler for GameState {
    fn init(&mut self, wrapper: &mut GpuWrapper) {
        let mut builder = TypefaceBuilder::new(include_bytes!("Curly-Girly.png"), [0, 0, 0, 0xff], 4, 13);
        builder.add_glyphs("abcdefgh", (7, 15), (1, 65), Some(1));
        builder.add_glyphs("ijklmnop", (7, 15), (1, 81), Some(1));
        builder.add_glyphs("qrstuvwx", (7, 15), (1, 97), Some(1));
        builder.add_glyphs("yz", (7, 15), (1, 113), Some(1));
        builder.set_x_offset('p', -3);
        builder.set_x_offset('j', -3);
        builder.add_sized_glyph(' ', (3, 1), (17, 113));
        self.typeface = Some(builder.into_typeface(wrapper));
    }

    fn redraw<F>(&self, _size: Vector2<u32>, _mouse_pos: Point2<f64>, _draw_ids: F) -> Vec<Sprite>
    where
        F: Fn(&Vec<Sprite>) -> IdBuffer,
    {
        let dc = DrawingContext::new((160.0, 120.0));
        self.typeface.as_ref().unwrap().print(dc, (0.0, 40.0), "i made a thing to render\nvariable width bitmap fonts")
    }
}

pub fn main() {
    let min_size = (80, 60);
    let _ = pollster::block_on(bananagraph::run_window("Text example", Vector2::from(min_size) * 8, min_size.into(), GameState::default()));
}
