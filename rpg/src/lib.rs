mod game_state;

use wasm_bindgen::prelude::*;
#[cfg(target_arch = "wasm32")]
use bananagraph::JsGpuWrapper;

#[wasm_bindgen(start)]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Info).expect("Couldn't initialize logger");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn init_game(canvas_id: &str, seed: f64) -> JsGpuWrapper {
    use web_sys::HtmlCanvasElement;
    use wgpu::SurfaceTarget;
    use bananagraph::{GpuWrapper, WindowEventHandler};
    use crate::game_state::GameState;
    use web_sys::js_sys::Math::pow;

    let mut wrapper = web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let canvas = doc.get_element_by_id(canvas_id).expect("Expected to find a canvas with the given id");
            let canvas: HtmlCanvasElement = canvas.dyn_into().expect("element must be a canvas");
            let size = (canvas.width(), canvas.height()).into();
            let surface_target = SurfaceTarget::Canvas(canvas);
            Some(GpuWrapper::targeting(surface_target, size, size))
        })
        .expect("Failed to init wgpu somehow").await;

    let mut handler = GameState::new((seed * pow(2.0, 32.0)) as u64);
    handler.init(&mut wrapper);

    JsGpuWrapper::new(wrapper, handler)
}
