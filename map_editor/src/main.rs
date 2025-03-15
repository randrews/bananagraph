use bananagraph::{Click, DrawingContext, ElementState, GpuWrapper, IdBuffer, Sprite, WindowEventHandler};
use cgmath::num_traits::Pow;
use cgmath::{Point2, Vector2};
use rand::Rng;
use grid::{Coord, Grid, GridMut};
use crate::board::{Board, Cell};
use crate::iso_map::{AsSprite, IsoMap};

mod board;
mod iso_map;

fn toggle_wall(id: u32, board: &mut Board) {
    if id >= 100000 {
        let coord = sprite_id_to_coord(id, board.size().x);
        let cell = board.get(coord).unwrap();

        *board.get_mut(coord).unwrap() = match cell {
            Cell::Black | Cell::White => {
                Cell::TallWall
            },
            Cell::ShortWall | Cell::TallWall => {
                Board::square_color(coord)
            },
            _ => unreachable!()
        }
    }
}

fn create_background(size: usize) -> Vec<u8> {
    let mut texture = vec![0u8; size * size * 4];
    let center = size as f32 / 2.0;
    let max_distance = (center * center * 2.0).sqrt();
    let mut rng = rand::rng();

    for y in 0..size {
        for x in 0..size {
            let distance: f32 = ((x as f32 - center).pow(2.0) + (y as f32 - center).pow(2.0)).sqrt() / max_distance;
            let distance = distance + (rng.random::<f32>() - 0.5) / 10.0;
            let val = (distance * 255.0) as u8;
            texture[(x + size * y) * 4 .. (x + size * y) * 4 + 4].copy_from_slice(&[val, val, val, 0xff])
        }
    }
    texture
}

/// This handles creating the drawing context to display the map, which implies creating a lot of the
/// screen layout of the game. The screen width is divided up like this:
/// |MM|----AA----|MM|-------------BB-------------|MM|
/// MMs are 5% margin columns; AA is a toolbar / status bar, BB is the map.
/// We want to devote most of the width to the map, so, let's say 0.65 map and 0.2 sidebar.
/// We would like to scale the map so it fits in that rectangle:
fn create_drawing_contexts(dims: Vector2<f32>, base_dc: DrawingContext) -> DrawingContext {
    let screen_proportion = 0.65; // The fraction of the screen width we devote to the map
    // We want to scale by the same factor, width and height, so whichever of those will fill the screen
    // with the lowest factor, that's what we use for both.
    let factor = (base_dc.screen.x / dims.x * screen_proportion).min(base_dc.screen.y / dims.y);
    let dc = base_dc.scale((factor, factor));

    // Shift us over by 0.3x of the screen width
    let dc = dc.translate((0.3, 0.0));

    // We're smaller in one dimension or the other, probably, so, center
    // us in that axis:
    if dims.x * factor < base_dc.screen.x * screen_proportion {
        let margin = base_dc.screen.x - dims.x;
        dc.translate(((margin / 2.0) / dc.screen.x / factor, 0.0))
    } else if dims.y * factor < base_dc.screen.y {
        let margin = base_dc.screen.y - dims.y;
        dc.translate((0.0, (margin / 2.0) / dc.screen.y / factor))
    } else {
        dc
    }
}

fn shorten_walls(board: &Board, mouse_coord: (i32, i32), sprites: Vec<Sprite>) -> Vec<Sprite> {
    let width = board.size().x;
    sprites.iter().map(|sprite| {
        if sprite.id >= 100000 {
            let coord: Vector2<i32> = sprite_id_to_coord(sprite.id, width).into();
            if matches!(board.get(coord), Some(Cell::TallWall)) &&
                (coord == mouse_coord.into() || coord.adjacent(mouse_coord.into())) &&
                coord.x > 0 &&
                coord.y > 0 {
                // The trick here is that the transform is the same. So we just make a new sprite
                // with the same transform, id, and z:
                return Cell::ShortWall.as_sprite().with_transform(sprite.transform).with_id(sprite.id).with_z(sprite.z)
            }
        }
        *sprite
    }).collect()
}

fn sprite_id_to_coord(id: u32, width: i32) -> (i32, i32) {
    ((id - 100000) as i32 % width, (id - 100000) as i32 / width)
}

fn highlight_sprites() -> (Sprite, Sprite) {
    let toph = Sprite::new((416, 0), (32, 48));
    let btmh = Sprite::new((384, 0), (32, 48));
    (
        toph,
        btmh
    )
}

struct GameState {
    board: Board
}

impl WindowEventHandler for GameState {
    fn init(&mut self, wrapper: &mut GpuWrapper) {
        wrapper.add_texture(include_bytes!("iso_dungeon_world.png"), Some("dungeon"));
        // wrapper.add_texture(include_bytes!("background.png"), Some("background"));
        wrapper.add_texture_from_array(create_background(720), 720, Some("background"));
    }

    fn redraw(&self, mouse_pos: Point2<f64>, wrapper: &GpuWrapper) -> Option<IdBuffer> {
        let size = wrapper.logical_size;
        let iso_map = IsoMap::new(&self.board, (32, 48), (32, 16));
        let base_dc = DrawingContext::new((size.x as f32, size.y as f32));

        let dims = iso_map.dimensions();
        let dc = create_drawing_contexts((dims.x as f32, dims.y as f32).into(), base_dc);

        let mut sprites = iso_map.sprites(dc);
        let mut buffer = wrapper.redraw_ids(&sprites).unwrap();

        if buffer.contains((mouse_pos.x as u32, mouse_pos.y as u32).into()) {
            let id = buffer[mouse_pos];
            if id >= 100000 {
                let board_coord = sprite_id_to_coord(id, self.board.size().x);
                sprites = shorten_walls(&self.board, board_coord, sprites);
                buffer = wrapper.redraw_ids(&sprites).unwrap();
            }
        }

        if buffer.contains((mouse_pos.x as u32, mouse_pos.y as u32).into()) {
            let id = buffer[mouse_pos];
            if id >= 100000 {
                let board_coord = sprite_id_to_coord(id, self.board.size().x);
                if let Some(Cell::White | Cell::Black) = self.board.get(board_coord) {
                    let highlight = highlight_sprites();
                    let z = iso_map.z_coord(board_coord);
                    sprites.push(iso_map.sprite(highlight.0.with_z(z - 0.0001), board_coord, &dc));
                    sprites.push(iso_map.sprite(highlight.1.with_z(z - 0.0003), board_coord, &dc));
                }
            }
        }

        // Push the background:
        sprites.push(Sprite::new((0, 0), (720, 720)).with_layer(1).with_z(0.99999).with_tint((0.2, 0.3, 0.4, 1.0)));

        wrapper.redraw_with_ids(sprites).ok()
    }

    fn click(&mut self, event: Click) {
        if let (ElementState::Pressed, Some(id)) = (event.state, event.entity) {
            toggle_wall(id, &mut self.board)
        }
    }
}

pub fn main() {
    env_logger::init();
    let size = (1280, 720);
    let board = Board::new(10, 7);
    let _ = pollster::block_on(bananagraph::run_window("The Thing", size.into(), size.into(), GameState { board }));
}
