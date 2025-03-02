mod gpu_wrapper;
mod id_buffer;
mod scale_transform;
mod sprite;
mod texture;
mod drawing_context;
mod windowing;
mod typeface;

pub use gpu_wrapper::GpuWrapper;
pub use id_buffer::IdBuffer;
pub use sprite::{Sprite, SpriteId};
pub use drawing_context::DrawingContext;
pub use windowing::{Click, WindowEventHandler, Dir, MouseButton, ElementState};
pub use typeface::{Typeface, Glyph, TypefaceBuilder, AddTexture};

#[cfg(not(target_arch = "wasm32"))]
pub use windowing::{run_window};