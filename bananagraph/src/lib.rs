mod gpu_wrapper;
mod id_buffer;
mod scale_transform;
mod sprite;
mod texture;
mod drawing_context;
mod typeface;
mod event_handler;

pub use gpu_wrapper::GpuWrapper;
pub use id_buffer::IdBuffer;
pub use sprite::{Sprite, SpriteId};
pub use drawing_context::DrawingContext;
pub use event_handler::{Click, WindowEventHandler, MouseButton, Dir, ElementState};
pub use typeface::{Typeface, Glyph, TypefaceBuilder, AddTexture};

#[cfg(feature = "desktop")]
mod windowing;

#[cfg(feature = "desktop")]
pub use windowing::run_window;

#[cfg(feature = "web")]
mod js_gpu_wrapper;

#[cfg(feature = "web")]
pub use js_gpu_wrapper::JsGpuWrapper;