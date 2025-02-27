use wasm_bindgen::prelude::*;
use web_sys::HtmlCanvasElement;
use wgpu::SurfaceTarget;
use wgpu::VertexStepMode::Instance;
use bananagraph::GpuWrapper;

#[wasm_bindgen(start)]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Debug).expect("Couldn't initialize logger");
}

#[wasm_bindgen]
pub async fn init_gpu_wrapper() {
    let wrapper = web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let canvas = doc.get_element_by_id("main_canvas").expect("Expected to find a canvas with id main_canvas");

            let canvas: HtmlCanvasElement = canvas.dyn_into().expect("element must be a canvas");
            let size = (canvas.width(), canvas.height()).into();
            let surface_target = SurfaceTarget::Canvas(canvas);
            // let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            //     backends: wgpu::Backends::PRIMARY,
            //     ..Default::default()
            // });
            //
            // let surface = instance.create_surface(surface_target).expect("Failed to create surface");

            // Some(surface)
            Some(GpuWrapper::targeting(surface_target, size))
        })
        .expect("I have no idea what I'm doing").await;
}