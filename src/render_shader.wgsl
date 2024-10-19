struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @builtin(position) position: vec4<f32>,
}

struct Locals {
    transform: mat4x4<f32>,
}

struct Sprite {
    @location(1) transform_i: vec3<f32>,
    @location(2) transform_j: vec3<f32>,
    @location(3) transform_k: vec3<f32>,
    @location(4) origin: vec2<f32>,
    @location(5) size: vec2<f32>
}

@group(0) @binding(0) var spritesheet_sampler: sampler;
@group(0) @binding(1) var spritesheet: texture_2d<f32>;
@group(0) @binding(2) var<uniform> locals: Locals;

@vertex
fn vs_main(
    @location(0) position: vec2<f32>, // A point in world coords: -1..1, +y is up
    sprite: Sprite
) -> VertexOutput {
    var out: VertexOutput;

    var transform = mat3x3<f32>(sprite.transform_i, sprite.transform_j, sprite.transform_k);

    out.position = locals.transform * vec4f(position, 0.0, 1.0);

    // Convert from world coords to texture coords
    let c = fma(position, vec2f(0.5, -0.5), vec2f(0.5, 0.5));

    // Transform the now-texture-coords unit square into the sprite rect
    out.tex_coord = fma(c, sprite.size, sprite.origin);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(spritesheet, spritesheet_sampler, in.tex_coord);
}