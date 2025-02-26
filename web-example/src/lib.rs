use std::sync::Arc;
use cgmath::Point2;
use wasm_bindgen::prelude::*;
use winit::dpi::PhysicalSize;
use winit::window::Window;
use bananagraph::{GpuWrapper, IdBuffer, WindowEventHandler};

struct Example {}

impl WindowEventHandler for Example {
    fn init(&mut self, _wrapper: &mut GpuWrapper, window: Arc<Window>) {
        let _ = window.request_inner_size(PhysicalSize::new(450, 400));

        #[cfg(target_arch = "wasm32")]
        {
            use winit::platform::web::WindowExtWebSys;

            web_sys::window()
                .and_then(|win| win.document())
                .and_then(|doc| {
                    let dst = doc.get_element_by_id("wasm-example")?;
                    let canvas = web_sys::Element::from(window.as_ref().canvas()?);
                    dst.append_child(&canvas).ok()?;
                    Some(())
                })
                .expect("Couldn't append canvas to document body.");
        }
    }

    fn redraw(&self, _mouse_pos: Point2<f64>, _wrapper: &GpuWrapper) -> Option<IdBuffer> {
        None
    }
}

#[cfg_attr(target_arch="wasm32", wasm_bindgen(start))]
pub fn run() {
    std::panic::set_hook(Box::new(console_error_panic_hook::hook));
    console_log::init_with_level(log::Level::Warn).expect("Couldn't initialize logger");

    // Winit prevents sizing with CSS, so we have to set the size manually when on web.
}
