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

// A transform matrix to convert from corners-of-the-world (-1..1, +y is up) to
// corners-of-the-texture (0..1, +y is down, like god intended).
// Note also that this is in a weird order, because the param order for mat3x3
// is column-major
const world_to_texture: mat3x3<f32> = mat3x3<f32>(
    0.5, 0.0, 0.0,
    0.0, -0.5, 0.0,
    0.5, 0.5, 1.0
);

const unit_to_world: mat3x3<f32> = mat3x3<f32>(
    2.0, 0.0, 0.0,
    0.0, -2.0, 0.0,
    -1.0, 1.0, 1.0
);

@vertex
fn vs_main(
    @location(0) position: vec2<f32>, // A point in unit coords: 0..1, +y is down
    sprite: Sprite
) -> VertexOutput {
    var out: VertexOutput;

    var transform = mat3x3<f32>(sprite.transform_i, sprite.transform_j, sprite.transform_k);

    var pt = transform * vec3f(position, 1.0);
    pt = unit_to_world * pt;
    out.position = locals.transform * vec4f(pt, 1.0);
    //out.position = vec4f(pt, 1.0);

    // Convert the world-coord-square into the rectangle of the actual sprite
    out.tex_coord = fma(position, sprite.size, sprite.origin);

    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return textureSample(spritesheet, spritesheet_sampler, in.tex_coord);
}