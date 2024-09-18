@group(0) @binding(0)
var out_texture: texture_storage_2d<bgra8unorm, write>;

struct WindowGeometry {
    fill: vec4<f32>,
    size: vec2<u32>,
    topleft: vec2<u32>,
    bottomright: vec2<u32>,
    dummy: vec2<u32>
}

@group(0) @binding(1)
var<uniform> geometry: WindowGeometry;

@compute
@workgroup_size(1)
fn pixel_shader(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x < geometry.topleft.x || global_id.x >= geometry.bottomright.x || global_id.y < geometry.topleft.y || global_id.y >= geometry.bottomright.y) {
       textureStore(out_texture, vec2<u32>(global_id.x, global_id.y), geometry.fill);
       //textureStore(out_texture, vec2<u32>(global_id.x, global_id.y), vec4<f32>(0.1, 0.1, 0.3, 1.0));
    } else {
       textureStore(out_texture, vec2<u32>(global_id.x, global_id.y), vec4<f32>(0, 0.5, 0.5, 1.0));
    }
}