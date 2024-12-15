use bytemuck::{Pod, Zeroable};
use winit::dpi::PhysicalSize;

/// Warning: head must be entirely de-assed before touching this code!
/// Represents the geometry of a window with a logical 640x480 display scaled / centered in it.
/// The window is both physically and logically larger than the display, so, this tells us how
/// to center the display, scale it to fill as much of the space as possible, and what color to
/// fill the margins with.
/// A word about alignment: vec4s in WGSL need to be aligned on 16-byte offsets, so `fill` has to
/// either be the first thing in the struct (0 % 16 == 0) or have an even number of vec2s before it.
/// Likewise, the entire struct needs to be sized so it's a multiple of that max alignment, so,
/// we add a dummy vec2 on the end to eat up eight more bytes. If we add more later, some of it
/// maybe could replace the dummy.
#[repr(C)]
#[derive(Copy, Clone, Debug, PartialEq, Pod, Zeroable)]
pub struct WindowGeometry {
    /// The RGBA color we want to fill the borders with
    pub fill: [f32; 4],

    /// The size of the physical window (and thus its Surface)
    pub size: [u32; 2],

    /// The pixel coordinates of the top left of the display
    pub topleft: [u32; 2],

    /// The pixel coordinates of the bottom right of the display
    pub bottomright: [u32; 2],

    /// Unused, for alignment
    pub dummy: [u32; 2],
}

impl WindowGeometry {
    pub fn new(size: PhysicalSize<u32>, fill: Option<[f32; 4]>) -> Self {
        let topleft = [(size.width - 640) / 2, (size.height - 480) / 2];

        let bottomright = [topleft[0] + 640, topleft[1] + 480];

        Self {
            size: [size.width, size.height],
            topleft,
            bottomright,
            fill: fill.unwrap_or([0f32, 0f32, 0f32, 1f32]),
            dummy: [0, 0],
        }
    }
}
