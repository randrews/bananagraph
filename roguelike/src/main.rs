mod components;
mod terrain;
mod animation;

use std::time::Duration;
use cgmath::Vector2;
use hecs::World;
use winit::event::{ElementState, KeyEvent};
use winit::keyboard::{Key, NamedKey};
use animation::BreatheAnimation;
use bananagraph::{Click, GpuWrapper, Sprite, WindowEventHandler};
use grid::{Coord, Dir, VecGrid};
use crate::components::{OnMap, Player};
use crate::terrain::recreate_terrain;

pub struct GameState {
    world: World
}

impl GameState {
    pub fn new() -> Self {
        Self {
            world: World::new()
        }
    }

    pub fn set_map(&mut self, map: VecGrid<char>) {
        recreate_terrain(map, &mut self.world)
    }

    pub fn set_player(&mut self, location: impl Into<Vector2<i32>>) {
        let location = location.into();
        // Remove the old player
        let player = self.world.query::<&Player>().iter().map(|(e, _)| e).next();
        player.map(|e| self.world.despawn(e));

        // Player animation frames
        let frames = vec![
            Sprite::new((0, 0), (16, 16)).with_layer(1),
            Sprite::new((16, 0), (16, 16)).with_layer(1),
            Sprite::new((32, 0), (16, 16)).with_layer(1),
            Sprite::new((32, 0), (16, 16)).with_layer(1),
            Sprite::new((32, 0), (16, 16)).with_layer(1),
            Sprite::new((16, 0), (16, 16)).with_layer(1)
        ];

        // Spawn a new player
        self.world.spawn((
            Player,
            OnMap { location, sprite: frames[0] },
            BreatheAnimation::new(frames)
        ));
    }

    pub fn walk(&mut self, dir: Dir) {
        for (_, (_p, on_map)) in self.world.query_mut::<(&mut Player, &mut OnMap)>() {
            on_map.location = on_map.location.translate(dir);
        }
    }
}

impl WindowEventHandler for GameState {
    fn init(&mut self, wrapper: &mut GpuWrapper) {
        wrapper.add_texture(include_bytes!("Dungeon.png"), Some("Dungeon.png"));
        wrapper.add_texture(include_bytes!("Heroes-Animated.png"), Some("Heroes-Animated.png"));
    }

    fn redraw(&self) -> Vec<Sprite> {
        OnMap::system(&self.world)
    }

    fn tick(&mut self, dt: Duration) {
        BreatheAnimation::system(&mut self.world, dt)
    }

    fn exit(&mut self) -> bool {
        true
    }

    fn click(&mut self, _event: Click) {
    }

    fn key(&mut self, event: KeyEvent, is_synthetic: bool) {
        // We can ignore release events...
        if event.state == ElementState::Released { return }

        match event.logical_key {
            Key::Named(NamedKey::ArrowDown) => self.walk(Dir::South),
            Key::Named(NamedKey::ArrowUp) => self.walk(Dir::North),
            Key::Named(NamedKey::ArrowLeft) => self.walk(Dir::West),
            Key::Named(NamedKey::ArrowRight) => self.walk(Dir::East),
            _ => {}
        }
    }
}

fn main() {
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

    let mut game_state = GameState::new();
    game_state.set_map(map);
    game_state.set_player((4, 2));

    let _ = pollster::block_on(bananagraph::run_window("Foo!", (1000, 800).into(), (250, 200).into(), &mut game_state));
}
