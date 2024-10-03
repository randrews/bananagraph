@group(0) @binding(0)
var out_texture: texture_storage_2d<bgra8unorm, write>;

// This is actually 128k bytes but we have to access it a word at a time
@group(0) @binding(1)
var<storage> mem: array<u32>;

struct DisplayRegisters {
    screen: u32,
    palette: u32,
    font: u32,
    height: u32,
    width: u32,
    row_offset: u32,
    col_offset: u32,
}

@compute
@workgroup_size(1)
fn pixel_shader(@builtin(global_invocation_id) global_id: vec3<u32>) {
    let mode = peek8(16u);
    let reg = read_display_registers();
    var color: vec4<f32>;
    switch (mode) {
        case 0u: {
            color = text_lowres_direct(reg, global_id.x, global_id.y);
        }
        case 1u: {
            color = gfx_lowres_direct(reg, global_id.x, global_id.y);
        }
        case 2u: {
            color = text_highres_direct(reg, global_id.x, global_id.y);
        }
        case 3u: {
            color = gfx_highres_direct(reg, global_id.x, global_id.y);
        }
        case 4u: {
            color = text_lowres_paletted(reg, global_id.x, global_id.y);
        }
        case 5u: {
            color = gfx_lowres_paletted(reg, global_id.x, global_id.y);
        }
        case 6u: {
            color = text_highres_paletted(reg, global_id.x, global_id.y);
        }
        case 7u: {
            color = gfx_highres_paletted(reg, global_id.x, global_id.y);
        }
        default: {
            color = vec4<f32>(0.3, 0.3, 0.5, 1.0);
        }
    }
    textureStore(out_texture, vec2<u32>(global_id.x, global_id.y), color);
}

/// Mode 0
fn text_lowres_direct(reg: DisplayRegisters, x: u32, y: u32) -> vec4<f32> {
    let vulcan_row = y >> 4;
    let vulcan_col = x >> 4;

    let addr = to_byte_address(reg, vulcan_col, vulcan_row);
    let char_idx = peek8(addr);
    let char_row = (y / 2) % 8;
    let char_col = (x / 2) % 8;
    let char_byte = peek8(reg.font + (char_idx << 3) + char_row);

    let color_addr = addr + (reg.width * reg.height);
    var color = peek8(color_addr);

    if !text_pixel_set(char_col, char_byte) {
        color = 0u;
    }

    return to_color(color);
}

/// Mode 1
fn gfx_lowres_direct(reg: DisplayRegisters, x: u32, y: u32) -> vec4<f32> {
    if y >= (240 - 64 * 3) && y < (240 + 64 * 3) && x >= (320 - 64 * 3) && x < (320 + 64 * 3) {
        let vy = (y - (240 - 64 * 3)) / 3;
        let vx = (x - (320 - 64 * 3)) / 3;

        let vb = peek8(to_byte_address(reg, vx, vy));
        return to_color(vb);
    } else {
        return to_color(0u);
    }
}

/// Mode 2
fn text_highres_direct(reg: DisplayRegisters, x: u32, y: u32) -> vec4<f32> {
    let vulcan_row = y >> 3;
    let vulcan_col = x >> 3;

    let addr = to_byte_address(reg, vulcan_col, vulcan_row);
    let char_idx = peek8(addr);
    let char_row = y % 8;
    let char_col = x % 8;
    let char_byte = peek8(reg.font + (char_idx << 3) + char_row);

    let color_addr = addr + (reg.width * reg.height);
    var color = peek8(color_addr);
    if !text_pixel_set(char_col, char_byte) {
        color = 0u;
    }
    return to_color(color);
}

/// Mode 3
fn gfx_highres_direct(reg: DisplayRegisters, x: u32, y: u32) -> vec4<f32> {
    let vulcan_row = y >> 2;
    let vulcan_col = x >> 2;
    let color = peek8(to_byte_address(reg, vulcan_col, vulcan_row));
    return to_color(color);
}

/// Mode 4
fn text_lowres_paletted(reg: DisplayRegisters, x: u32, y: u32) -> vec4<f32> {
    let vulcan_row = y >> 4;
    let vulcan_col = x >> 4;

    let addr = to_byte_address(reg, vulcan_col, vulcan_row);
    let char_idx = peek8(addr);
    let char_row = (y / 2) % 8;
    let char_col = (x / 2) % 8;
    let char_byte = peek8(reg.font + (char_idx << 3) + char_row);

    let color_addr = addr + (reg.width * reg.height);
    var index = peek8(color_addr);

    if text_pixel_set(char_col, char_byte) {
        index = index & 0xf; // Set, so, fg color
    } else {
        index = index >> 4u; // Clear, so, bg color
    }
    return palette_lookup(reg, index);
}

/// Mode 5
fn gfx_lowres_paletted(reg: DisplayRegisters, x: u32, y: u32) -> vec4<f32> {
    if y >= (240 - 64 * 3) && y < (240 + 64 * 3) && x >= (320 - 64 * 3) && x < (320 + 64 * 3) {
        let vy = (y - (240 - 64 * 3)) / 3;
        let vx = (x - (320 - 64 * 3)) / 3;

        let index = peek8(to_byte_address(reg, vx, vy));
        return palette_lookup(reg, index);
    } else {
        return to_color(0u);
    }
}

/// Mode 6
fn text_highres_paletted(reg: DisplayRegisters, x: u32, y: u32) -> vec4<f32> {
    let vulcan_row = y >> 3;
    let vulcan_col = x >> 3;

    let addr = to_byte_address(reg, vulcan_col, vulcan_row);
    let char_idx = peek8(addr);
    let char_row = y % 8;
    let char_col = x % 8;
    let char_byte = peek8(reg.font + (char_idx << 3) + char_row);

    let color_addr = addr + (reg.width * reg.height);
    var index = peek8(color_addr);
    if text_pixel_set(char_col, char_byte) {
        index = index & 0xf; // Set, so, fg color
    } else {
        index = index >> 4u; // Clear, so, bg color
    }
    return palette_lookup(reg, index);
}

/// Mode 7
fn gfx_highres_paletted(reg: DisplayRegisters, x: u32, y: u32) -> vec4<f32> {
    let vulcan_row = y >> 2;
    let vulcan_col = x >> 2;
    let index = peek8(to_byte_address(reg, vulcan_col, vulcan_row));
    return palette_lookup(reg, index);
}

fn text_pixel_set(char_col: u32, char_byte: u32) -> bool {
    return (char_byte & u32(1u << (7 - char_col))) != 0;
}

// Return the byte address of the given screen coords, taking scroll registers into account
fn to_byte_address(reg: DisplayRegisters, x: u32, y: u32) -> u32 {
    let row_start = ((y + reg.row_offset) % reg.height) * reg.width + reg.screen;
    return ((x + reg.col_offset) % reg.width) + row_start;
}

// Read the register block and parse it into a struct for comfort
fn read_display_registers() -> DisplayRegisters {
    let reg = DisplayRegisters(
        peek24(17u), // screen
        peek24(20u), // palette
        peek24(23u), // font
        peek24(26u), // height
        peek24(29u), // width
        peek24(32u), // row_offset
        peek24(35u) // col_offset
    );
    return reg;
}

// Turn a byte into a color; unpack / scale RRRGGGBB into a vec4<f32>
fn to_color(byte: u32) -> vec4<f32> {
    let blue = byte & 3;
    let green = (byte >> 2) & 7;
    let red = (byte >> 5) & 7;

    let blue_shifted = (blue << 2) | blue;
    let blue_scaled = (blue_shifted << 4) | blue_shifted;

    let scaled_green = (((green << 3) | green) << 2) | (green & 3);
    let scaled_red = (((red << 3) | red) << 2) | (red & 3);

    return vec4<f32>(f32(scaled_red) / 255.0, f32(scaled_green) / 255.0, f32(blue_scaled) / 255.0, 1.0);
}

fn palette_lookup(reg: DisplayRegisters, index: u32) -> vec4<f32> {
    let color_word = peek24(reg.palette + 3 * (index % 16));
    return to_24_color(color_word);
}

fn to_24_color(word: u32) -> vec4<f32> {
    let bytes = unpack4xU8(word);
    return vec4<f32>(f32(bytes[0]) / 255.0, f32(bytes[1]) / 255.0, f32(bytes[2]) / 255.0, 1.0);
}

// Actually returns a byte, only the low-order bit is populated
fn peek8(addr: u32) -> u32 {
    let word = unpack4xU8(mem[addr / 4]);
    return word[addr % 4];
}

// Return the (zero-padded) 24-bit value whose low byte is at addr
fn peek24(addr: u32) -> u32 {
    let low_word = unpack4xU8(mem[addr / 4]);

    switch (addr % 4) {
        case 0u: {
            // Happy case 1; we have the whole word right here:
            return pack4xU8(vec4<u32>(low_word[0], low_word[1], low_word[2], 0));
        }
        case 1u: {
            // Happy case 2; we have the whole word right here, but we're a bit misaligned:
            return pack4xU8(vec4<u32>(low_word[1], low_word[2], low_word[3], 0));
        }
        case 2u: {
            // We need to read the low byte of the next thing, so do that:
            let high_word = unpack4xU8(mem[(addr / 4) + 1]);
            return pack4xU8(vec4<u32>(low_word[2], low_word[3], high_word[0], 0));
        }
        case 3u: {
            // We need to read the low two bytes of the next thing, so do that:
            let high_word = unpack4xU8(mem[(addr / 4) + 1]);
            return pack4xU8(vec4<u32>(low_word[3], high_word[0], high_word[1], 0));
        }
        default: {
            // Just in case something % 4 isn't in 0..3?
            return pack4xU8(vec4<u32>(0, 0, 0, 0));
        }
    }
}