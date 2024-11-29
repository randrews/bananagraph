struct VertexOutput {
    @location(0) tex_coord: vec2<f32>,
    @location(1) override_alpha: f32,
    @location(2) is_override_alpha: u32,
    @location(3) id: u32,
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
    @location(5) size: vec2<f32>,
    @location(6) z: f32,
    @location(7) id: u32,
    @location(8) override_alpha: f32,
    @location(9) is_override_alpha: u32
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

// The vertex shader
@vertex fn vs_main(
    @location(0) position: vec2<f32>, // A point in unit coords: 0..1, +y is down
    sprite: Sprite
) -> VertexOutput {
    var out: VertexOutput;

    var transform = mat3x3<f32>(sprite.transform_i, sprite.transform_j, sprite.transform_k);

    var pt = transform * vec3f(position, 1.0);
    pt = unit_to_world * pt;
    var transformed = locals.transform * vec4f(pt, 1.0);
    out.position = vec4f(transformed.x, transformed.y, sprite.z, 1.0);

    // Convert the world-coord-square into the rectangle of the actual sprite
    out.tex_coord = fma(position, sprite.size, sprite.origin);

    // Write the alpha-override fields and the id:
    out.override_alpha = sprite.override_alpha;
    out.is_override_alpha = sprite.is_override_alpha;
    out.id = sprite.id;

    return out;
}

// The entry point for the fragment shader. Takes vertex outputs and turns them into colors
@fragment fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    //return textureSample(spritesheet, spritesheet_sampler, in.tex_coord);
    var color: vec4<f32> = textureSample(spritesheet, spritesheet_sampler, in.tex_coord);

    // If we should override the alpha, do so. We override it if the original sprite had
    // Some() as the override, and if the texture has an actual opaque pixel for this spot
    if in.is_override_alpha != 0 && color.a > 0.0 {
        color.a = in.override_alpha;
    }

    // Alpha 0 is a special case where we just do nothing. This is _most_ of what you need
    // to make alpha blending work; the other part is that if you want more than one bit of
    // alpha, you need to depth sort the sprites; we can only blend with pixels we've already
    // drawn. But if you don't care about that then just having a depth buffer + this will
    // make 1-bit alpha work fine.
    if color.a == 0.0 {
        discard;
    } else {
        return color;
    }
}

// Entry point for the id pipeline, which renders a texture with the id of the topmost sprite
// at each pixel location (for hit detection)
@fragment fn fs_id(in: VertexOutput) -> @location(0) u32 {
    var color: vec4<f32> = textureSample(spritesheet, spritesheet_sampler, in.tex_coord);

    if in.is_override_alpha != 0 && color.a > 0.0 {
        color.a = in.override_alpha;
    }

    if color.a == 0.0 {
        discard;
    } else {
        return in.id;
    }
}