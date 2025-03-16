use cgmath::Point2;
use bananagraph::{GpuWrapper, IdBuffer, WindowEventHandler};

pub struct GameState {

}

impl GameState {
    pub fn new(_seed: u64) -> Self {
        Self { }
    }
}

impl WindowEventHandler for GameState {
    fn redraw(&self, _mouse_pos: Point2<f64>, _wrapper: &GpuWrapper) -> Option<IdBuffer> {
        None
    }
}