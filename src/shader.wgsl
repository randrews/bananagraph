struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
}

struct WindowGeometry {
    fill: vec4<f32>,
    size: vec2<u32>,
    topleft: vec2<u32>,
    bottomright: vec2<u32>,
    dummy: vec2<u32>
}

@group(0) @binding(0) var compute_texture: texture_storage_2d<bgra8unorm, write>;
@group(0) @binding(1) var render_texture: texture_2d<f32>;
@group(0) @binding(2) var<uniform> geometry: WindowGeometry;
@group(0) @binding(3) var r_tex_sampler: sampler;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>,
) -> VertexOutput {
    var out: VertexOutput;
    out.tex_coord = fma(position, vec2<f32>(0.5, -0.5), vec2<f32>(0.5, 0.5));
    //out.position = r_locals.transform * vec4<f32>(position, 0.0, 1.0);
    out.position = vec4<f32>(position, 0.0, 1.0);
    return out;
}

@compute
@workgroup_size(1)
fn pixel_shader(@builtin(global_invocation_id) global_id: vec3<u32>) {
    if (global_id.x < geometry.topleft.x || global_id.x >= geometry.bottomright.x || global_id.y < geometry.topleft.y || global_id.y >= geometry.bottomright.y) {
       textureStore(compute_texture, vec2<u32>(global_id.x, global_id.y), geometry.fill);
       //textureStore(out_texture, vec2<u32>(global_id.x, global_id.y), vec4<f32>(0.1, 0.1, 0.3, 1.0));
    } else {
       textureStore(compute_texture, vec2<u32>(global_id.x, global_id.y), vec4<f32>(0, 0.5, 0.5, 1.0));
    }
}

@fragment
fn fs_main(@location(0) tex_coord: vec2<f32>) -> @location(0) vec4<f32> {
    return textureSample(render_texture, r_tex_sampler, tex_coord);
}