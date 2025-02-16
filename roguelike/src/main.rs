use std::time::Duration;
use cgmath::{Point2, Vector2};
use bananagraph::{Click, DrawingContext, GpuWrapper, Sprite, WindowEventHandler};
use grid::{Grid, VecGrid};

pub struct GameState {}

pub fn wall_tile(neighbors: (bool, bool, bool, bool)) -> Sprite {
    // north, south, east, west
    let mut origin = match neighbors {
        (false, false, false, false) => (5, 1),
        (true, true, true, true) => (5, 0),

        (true, true, false, false) => (4, 1),
        (false, false, true, true) => (3, 0),

        (true, false, false, false) => (0, 2),
        (false, false, true, false) => (2, 2),
        (false, true, false, false) => (1, 2),
        (false, false, false, true) => (3, 1),

        (false, true, true, true) => (0, 0),
        (true, true, false, true) => (1, 0),
        (true, false, true, true) => (1, 1),
        (true, true, true, false) => (0, 1),

        (false, true, true, false) => (2, 0),
        (false, true, false, true) => (4, 0),
        (true, false, true, false) => (2, 1),
        (true, false, false, true) => (4, 2),
    };

    origin.1 += 3;
    Sprite::new(Point2::from(origin) * 16, (16, 16))
}

impl WindowEventHandler for GameState {
    fn init(&mut self, wrapper: &mut GpuWrapper) {
        wrapper.add_texture(include_bytes!("Dungeon.png"), Some("Dungeon.png"));
    }

    fn redraw(&self) -> Vec<Sprite> {
        let map: VecGrid<char> = VecGrid::from(vec![
            "..........",
            "..######..",
            "..#....#..",
            "..#..###..",
            "..####.#..",
            "...#...#..",
            "...#...#..",
            "...#####..",
            "..........",
            "..........",
        ].join("\n").as_str());

        let dc = DrawingContext::new((320.0, 256.0));
        let mut sprites = vec![];
        for (n, ch) in map.iter().enumerate() {
            let c = map.coord(n);
            let spr = match ch {
                '.' => Sprite::new((144, 128), (16, 16)),
                '#' => {
                    wall_tile(map.neighbors_equal(c, '#'))
                },
                _ => Sprite::new((80, 16), (16, 16))
            };
            sprites.push(dc.place(spr, (c.x as f32 * 16.0, c.y as f32 * 16.0)))
        }
        sprites
    }

    fn tick(&mut self, dt: Duration) {
    }

    fn exit(&mut self) -> bool {
        true
    }

    fn click(&mut self, event: Click) {
    }
}

fn main() {
    let mut game_state = GameState {};
    let _ = pollster::block_on(bananagraph::run_window("Foo!", (1000, 800).into(), (250, 200).into(), &mut game_state));
}
