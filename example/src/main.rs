use bananagraph::{DrawingContext, GpuWrapper, Sprite, WindowEventHandler};
use cgmath::Deg;

struct GameState {
}

impl WindowEventHandler for GameState {
    fn init(&mut self, wrapper: &mut GpuWrapper) {
        wrapper.add_texture(include_bytes!("cube.png"), None);
        wrapper.add_texture(include_bytes!("background.png"), None);
    }

    fn redraw(&self) -> Vec<Sprite> {
        // let (w, h) = (400.0, 225.0);
        // let sprite = Sprite::new((0, 0), (32, 32))
        //     //.translate((-0.5, -0.5))
        //     //.scale((32.0, 32.0))
        //     //.inv_scale((800.0, 450.0))
        //     .translate((-0.5, -0.5))
        //     .rotate(Deg(90.0))
        //     //.scale((2.0, 2.0))
        //     .translate((0.5, 0.5))
        //     .scale((32.0 / w, 32.0 / h))
        //     //.inv_scale((w, h))
        //     .translate((1.0 / w * 32.0, 1.0 / h * 16.0))
        //     ;

        let bg = Sprite::new((0, 0), (800, 450)).with_layer(1).with_z(0.2);
        let sprite = Sprite::new((0, 0), (32, 32)).with_z(0.1);

        // A screen the size of the window, but tilted, shrunk, and translated
        let dc = DrawingContext::new((800.0, 450.0))
            .rotate(Deg(-45.0))
            .scale((0.5, 0.5))
            .translate((0.25, 0.5));

        vec![
            dc.place(bg, (0.0, 0.0)), // Just draw the grayness straight
            dc.place_rotated(sprite, (0.0, 0.0), Deg(45.0)), // Rotate the cube about its center
            sprite
                .scale((32.0, 32.0)) // Scale to the sprite size in pixels. Sprite is now huge, and distorted!
                .translate((100.0, 100.0)) // Translate to pixel coords
                .rotate(Deg(10.0)) // Rotate however we like
                .scale((1.0 / 400.0, 1.0 / 225.0)) // Scale everything back down by the size of the world, also removes distortion
                .with_tint((1.0, 1.0, 0.4, 1.0))
                .with_z(0.01),
            sprite
                .scale((32.0, 32.0)) // Scale to the sprite size in pixels. Sprite is now huge, and distorted!
                .translate((-16.0, -16.0)) // Translate to put the center on the origin
                .rotate(Deg(45.0)) // Rotate however we like
                .translate((26.0, 26.0)) // translate back, and then some, in pixel coords
                .scale((1.0 / 400.0, 1.0 / 225.0)) // Scale everything back down by the size of the world, also removes distortion
                .with_z(0.008),
            bg.with_z(0.9999)
        ]
    }
}

pub fn main() {
    let size = (800, 450);
    let _ = pollster::block_on(bananagraph::run_window("Bananagraph example", size.into(), size.into(), GameState {}));
}
