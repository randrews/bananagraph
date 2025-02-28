use wasm_bindgen::prelude::*;

#[wasm_bindgen(start)]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Error).expect("Couldn't initialize logger");
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn init_gpu_wrapper() {
    use web_sys::HtmlCanvasElement;
    use wgpu::SurfaceTarget;
    use bananagraph::{GpuWrapper, Sprite};

    let mut wrapper = web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let canvas = doc.get_element_by_id("main_canvas").expect("Expected to find a canvas with id main_canvas");
            let canvas: HtmlCanvasElement = canvas.dyn_into().expect("element must be a canvas");
            let size = (canvas.width(), canvas.height()).into();
            let surface_target = SurfaceTarget::Canvas(canvas);
            Some(GpuWrapper::targeting(surface_target, size, size))
        })
        .expect("I have no idea what I'm doing").await;

    wrapper.add_texture(include_bytes!("angrytoast.png"), Some("angrytoast.png"));
    let toast = Sprite::new((0, 0), (320, 240)).with_id(1);
    let ids = wrapper.redraw_with_ids([toast]).unwrap();
}