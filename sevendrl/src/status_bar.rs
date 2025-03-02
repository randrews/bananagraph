use cgmath::Vector2;
use hecs::World;
use log::info;
use bananagraph::{DrawingContext, Sprite, Typeface};

#[derive(Clone)]
pub struct StatusBar {
    pub message: String
}

impl StatusBar {
    pub fn system(world: &World, typeface: &Typeface) -> Vec<Sprite> {
        let mut sprites = Self::frame_sprites();
        let dc = DrawingContext::new((960.0 / 2.0, 544.0 / 2.0));

        if let Some((_, status_bar)) = world.query::<&StatusBar>().into_iter().next() {
            let coord = Self::tile_coord((0, 0)) + Vector2::new(0.0, 13.0);
            sprites.append(&mut typeface.print(dc, coord, &status_bar.message.as_str()));
        }
        // sprites.push(dc.place(Sprite::new((16, 128), (16, 16)), Self::tile_coord((0, 0))));
        // sprites.push(dc.place(Sprite::new((16, 128), (16, 16)), Self::tile_coord((0, 1))));
        // sprites.push(dc.place(Sprite::new((16, 128), (16, 16)), Self::tile_coord((0, 2))));
        sprites
    }

    /// With room for the frame and other things, the status area is a rectangle 29 x 3 tiles
    /// in area. This takes a point in that space and returns a point suitable for passing to a
    /// drawingcontext
    fn tile_coord(loc: impl Into<Vector2<i32>>) -> Vector2<f32> {
        let Vector2 { x, y } = loc.into();
        Vector2::new(
            x as f32 * 16.0 + 8.0,
            y as f32 * 16.0 + 13.0 * 16.0 + 8.0
        )
    }

    /// The sprites forming the frame and background
    fn frame_sprites() -> Vec<Sprite> {
        let mut sprites = vec![];
        let dims = Vector2::new(960.0 / 2.0, 544.0 / 2.0);
        let dc = DrawingContext::new(dims);
        // First throw the outline sprites in there:
        // The board is 960x544, which we divide by two to get 480x272.
        // We use the whole width and the map takes up the top 13x16 = 208 px
        // so our rectangle is (0, 208) to (479, 271), for 30x4 tiles.
        let corners = (
            Sprite::new((54, 134), (16, 16)).with_z(0.9).with_layer(2),
            Sprite::new((90, 134), (16, 16)).with_z(0.9).with_layer(2),
            Sprite::new((54, 171), (16, 16)).with_z(0.9).with_layer(2),
            Sprite::new((90, 171), (16, 16)).with_z(0.9).with_layer(2)
        );
        sprites.push(dc.place(corners.0, (0.0, 13.0 * 16.0)));
        sprites.push(dc.place(corners.1, (dims.x - 16.0, 13.0 * 16.0)));
        sprites.push(dc.place(corners.2, (0.0, dims.y - 16.0)));
        sprites.push(dc.place(corners.3, (dims.x - 16.0, dims.y - 16.0)));
        let edges = (
            Sprite::new((70, 134), (16, 16)).with_z(0.9).with_layer(2),
            Sprite::new((90, 150), (16, 16)).with_z(0.9).with_layer(2),
            Sprite::new((74, 171), (16, 16)).with_z(0.9).with_layer(2),
            Sprite::new((54, 150), (16, 16)).with_z(0.9).with_layer(2)
        );
        let tile_dims = Vector2::new(dims.x as u32 / 16, dims.y as u32 / 16 - 13);
        for x in 1..tile_dims.x - 1 {
            sprites.push(dc.place(edges.0, (x as f32 * 16.0, 13.0 * 16.0))); // top edge
            sprites.push(dc.place(edges.2, (x as f32 * 16.0, dims.y - 16.0))); // btm edge
        }
        for y in 1..tile_dims.y - 1{
            sprites.push(dc.place(edges.3, (0.0, 13.0 * 16.0 + y as f32 * 16.0))); // left edge
            sprites.push(dc.place(edges.1, (dims.x - 16.0, 13.0 * 16.0 + y as f32 * 16.0))); // btm edge
        }
        let middle = Sprite::new((16, 144), (16, 16)).with_z(0.9).with_layer(2);
        for y in 1 .. tile_dims.y {
            for x in 1..tile_dims.x {
                sprites.push(dc.place(middle, (x as f32 * 16.0 - 8.0, 13.0 * 16.0 + y as f32 * 16.0 - 8.0))); // top edge
            }
        }
        sprites
    }
}