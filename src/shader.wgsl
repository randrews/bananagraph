@group(0) @binding(0)
var out_texture: texture_storage_2d<bgra8unorm, write>;

@compute
@workgroup_size(1)
fn pixel_shader(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x < 320 && global_id.y < 240) {
       textureStore(out_texture, vec2<u32>(global_id.x, global_id.y), vec4<f32>(0.7, 0.2, 0.2, 1.0));
    } else {
       textureStore(out_texture, vec2<u32>(global_id.x, global_id.y), vec4<f32>(0, 0.5, 0.5, 1.0));
    }
}