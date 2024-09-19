/// Create a transform matrix to scale / center a texture into a larger rectangle,
/// scaling it up as much as possible but only evenly. This is copied from the `pixels`
/// crate: https://github.com/parasyte/pixels/blob/main/src/renderers.rs
///
/// That being said... This is actually not a great scaling system for Vulcan. For one
/// thing, it's probably actually okay that we scale to non-integer scaling factors,
/// since we're almost forced to by some monitors: any monitor with a non-integer factor
/// will have winit create a window that's in between scales. Also, we may not even want
/// the nearest-neighbor sampler that this works with; at a high enough resolution it
/// may actually just look fine blurry. So this may get cut down or altered later.
pub fn transform(texture_size: (u32, u32), window_size: (u32, u32)) -> [f32; 16] {
    let (texture_width, texture_height) = texture_size;
    let (screen_width, screen_height) = window_size;
    let (screen_width, screen_height) = (screen_width as f32, screen_height as f32);

    let width_ratio = (screen_width as f32 / texture_width as f32).max(1.0);
    let height_ratio = (screen_height as f32 / texture_height as f32).max(1.0);

    // Get smallest scale size. To force integer scales, add a .floor() to this
    let scale = width_ratio.clamp(1.0, height_ratio);

    let scaled_width = texture_width as f32 * scale;
    let scaled_height = texture_height as f32 * scale;

    // Create a transformation matrix
    let sw = scaled_width / screen_width;
    let sh = scaled_height / screen_height;
    let tx = (screen_width/ 2.0).fract() / screen_width;
    let ty = (screen_height / 2.0).fract() / screen_height;

    [
        sw,  0.0, 0.0, 0.0,
        0.0, sh,  0.0, 0.0,
        0.0, 0.0, 1.0, 0.0,
        tx,  ty,  0.0, 1.0,
    ]
}