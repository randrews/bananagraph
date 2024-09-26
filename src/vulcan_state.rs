use rand::RngCore;

/// Return a block of random bytes, but with some of them initialized to reasonable values for a Vulcan display:
/// - The font rom will be initialized to a font and the font pointer will be pointing to it
/// - Thu mode byte will be set to the given mode
/// - Depending on the mode, rows / columns will be set reasonably
/// - Row and column offsets will be zero
pub fn random_vulcan(mode: u8) -> [u8; 131072] {
    let mut rng = rand::thread_rng();
    let mut buf = [0u8; 131072];

    // Randomize!
    rng.fill_bytes(&mut buf);

    // Copy font
    let font_bytes = include_bytes!("font.rom");
    let font_start = (0x20000 - 0x100 - 0x2000) as usize;
    buf[font_start..(font_start + 2048)].copy_from_slice(font_bytes);

    // Copy palette
    let palette_bytes: [u8; 16] = [
        0x00, 0x05, 0x65, 0x11, 0xa8, 0x49, 0xeb, 0xff, 0xe1, 0xf4, 0xfc, 0x1c, 0x37, 0x8e,
        0xee, 0xfa,
    ];
    let palette_start = (0x20000 - 0x100) as usize;
    buf[palette_start..(palette_start + 16)].copy_from_slice(&palette_bytes);

    // Start address for registers
    let start = 16usize;

    // Set mode
    buf[start] = mode;

    // Set font
    poke(&mut buf, start + 7, font_start as u32);

    // set palette
    poke(&mut buf, start + 4, palette_start as u32);

    // set offsets
    poke(&mut buf, start + 16, 0);
    poke(&mut buf, start + 19, 0);

    // set sizes
    let (cols, rows) = match mode {
        0 | 4 => (40, 30), // low-res text
        2 | 6 => (80, 60), // high res text
        1 | 5 => (128, 128), // low-res (pico) graphics
        3 | 7 => (160, 120), // high-res (1/4 vga) graphics
        _ => panic!("Invalid Vulcan graphics mode")
    };
    poke(&mut buf, start + 10, rows);
    poke(&mut buf, start + 13, cols);

    // Set screen ptr to default
    poke(&mut buf, start + 1, 0x10000);

    //buf[0x10000 + 10 + 160 .. 0x10000 + 20 + 160].copy_from_slice(&[0; 10]);

    buf
}

fn poke(buf: &mut [u8], addr: usize, value: u32) {
    let low = (value & 0xff) as u8;
    let mid = ((value & 0xff00) >> 8) as u8;
    let high = ((value & 0xff0000) >> 16) as u8;

    buf[addr] = low;
    buf[addr + 1] = mid;
    buf[addr + 2] = high;
}

#[cfg(test)]
mod test {
    use crate::vulcan_state::random_vulcan;

    #[test]
    fn test_random_vulcan() {
        let mem = random_vulcan(4);
        assert_eq!(mem[16], 4); // mode
        assert_eq!(mem[16 + 7], 0); // three bytes of font address
        assert_eq!(mem[16 + 8], 0xdf);
        assert_eq!(mem[16 + 9], 0);
        assert_eq!(mem[0x20000 - 0x100 + 3], 0x11); // Spot check the palette
    }
}