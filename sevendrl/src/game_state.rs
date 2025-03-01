use cgmath::Point2;
use log::info;
use bananagraph::{GpuWrapper, IdBuffer, Sprite, WindowEventHandler};

pub struct GameState {}

impl WindowEventHandler for GameState {
    fn init(&mut self, wrapper: &mut GpuWrapper) {
        wrapper.add_texture(include_bytes!("Dungeon.png"), Some("Dungeon.png"));
    }

    fn redraw(&self, _mouse_pos: Point2<f64>, wrapper: &GpuWrapper) -> Option<IdBuffer> {
        let blah = Sprite::new((0, 0), (160, 160)).with_id(1);
        Some(wrapper.redraw_with_ids([blah]).unwrap())
    }

    fn letter_key(&mut self, s: &str) {
        info!("You said {}", s)
    }
}

impl GameState {
    // Gotta shut clippy up about this because it's only called in a fn that's only visible
    // to wasm32.
    #[allow(dead_code)]
    pub fn new() -> Self {
        Self {}
    }
}