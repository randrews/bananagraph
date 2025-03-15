use std::ops::{Deref, DerefMut};
use wasm_bindgen::prelude::*;
use bananagraph::{GpuWrapper, Sprite};

#[wasm_bindgen(start)]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Error).expect("Couldn't initialize logger");
}

/// We can't send a GpuWrapper to JS directly without it trying to generate stuff it can't generate
/// so we need to wrap it in a bindgen'd type so we can tell bindgen to skip it. We also can't expose
/// a lifetime'd type to JS so all we can really do is make it static
#[wasm_bindgen]
pub struct JsGpuWrapper {
    #[wasm_bindgen(skip)]
    wrapper: GpuWrapper<'static>
}

impl From<GpuWrapper<'static>> for JsGpuWrapper {
    fn from(wrapper: GpuWrapper<'static>) -> Self {
        Self { wrapper }
    }
}

impl Deref for JsGpuWrapper {
    type Target = GpuWrapper<'static>;

    fn deref(&self) -> &Self::Target {
        &self.wrapper
    }
}

impl DerefMut for JsGpuWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.wrapper
    }
}

#[cfg(target_arch = "wasm32")]
#[wasm_bindgen]
pub async fn init_gpu_wrapper(canvas_id: &str) -> JsGpuWrapper {
    use web_sys::HtmlCanvasElement;
    use wgpu::SurfaceTarget;
    use bananagraph::GpuWrapper;

    web_sys::window()
        .and_then(|win| win.document())
        .and_then(|doc| {
            let canvas = doc.get_element_by_id(canvas_id).expect("Expected to find a canvas with the given id");
            let canvas: HtmlCanvasElement = canvas.dyn_into().expect("element must be a canvas");
            let size = (canvas.width(), canvas.height()).into();
            let surface_target = SurfaceTarget::Canvas(canvas);
            Some(GpuWrapper::targeting(surface_target, size, size))
        })
        .expect("Failed to init wgpu somehow").await.into()
}

#[wasm_bindgen]
pub fn draw_toast(mut wrapper: JsGpuWrapper) {
    wrapper.add_texture(include_bytes!("angrytoast.png"), Some("angrytoast.png"));
    let toast = Sprite::new((0, 0), (320, 240)).with_id(1);
    wrapper.redraw_with_ids([toast]).unwrap();
}