use std::iter::Filter;
use std::time::Duration;
use cgmath::{Point2, Vector2};
use hecs::{Entity, Query, QueryIter, World};
use log::info;
use bananagraph::{GpuWrapper, IdBuffer, Sprite, WindowEventHandler};
use grid::{Coord, Dir, VecGrid};
use crate::animation::BreatheAnimation;
use crate::components::{OnMap, Player};
use crate::terrain::{recreate_terrain, Wall};

pub struct GameState {
    world: World,
}

impl Default for GameState {
    fn default() -> Self {
        Self { world: World::default() }
    }
}

impl WindowEventHandler for GameState {
    fn init(&mut self, wrapper: &mut GpuWrapper) {
        wrapper.add_texture(include_bytes!("Dungeon.png"), Some("Dungeon.png"));
        wrapper.add_texture(include_bytes!("Heroes-Animated.png"), Some("Heroes-Animated.png"));
    }

    fn redraw(&self, _mouse_pos: Point2<f64>, wrapper: &GpuWrapper) -> Option<IdBuffer> {
        wrapper.redraw_with_ids(OnMap::system(&self.world)).ok()
    }

    fn tick(&mut self, dt: Duration) {
        BreatheAnimation::system(&mut self.world, dt)
    }

    fn arrow_key(&mut self, dir: bananagraph::Dir) {
        self.walk(convert_dir(dir))
    }
}

impl GameState {
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

    fn find_on_map<Q: Query>(&mut self, loc: impl Into<Vector2<i32>>) -> Vec<<Q as Query>::Item<'_>> {
        let loc = loc.into();
        self.world.query_mut::<(Q, &OnMap)>().into_iter()
            .filter_map(|(e, (q, on_map))| {
                if on_map.location == loc {
                    Some(q)
                } else {
                    None
                }
            }).collect()
    }

    fn exists_on_map<Q: Query>(&self, loc: impl Into<Vector2<i32>>) -> bool {
        let loc = loc.into();
        self.world.query::<(Q, &OnMap)>().iter().any(|(_, (_, on_map))| on_map.location == loc)
    }

    // There's only one, after all...
    fn get_player<Q: Query>(&mut self) -> <Q as Query>::Item<'_> {
        let (_, (q, _)) = self.world.query_mut::<(Q, &mut Player)>().into_iter().next().unwrap();
        q
    }

    pub fn walk(&mut self, dir: Dir) {
        let new_loc = self.get_player::<&OnMap>().location.translate(dir);
        if self.find_on_map::<&Wall>(new_loc).is_empty() {
            self.get_player::<&mut OnMap>().location = new_loc;
        }
    }

    // Gotta shut clippy up about this because it's only called in a fn that's only visible
    // to wasm32.
    #[allow(dead_code)]
    pub fn new() -> Self {
        let map: VecGrid<char> = VecGrid::from([
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

        let mut game_state = Self::default();
        game_state.set_map(map);
        game_state.set_player((4, 2));
        game_state
    }
}

/// Glue to convert bgraph's dir to grid's
fn convert_dir(bdir: bananagraph::Dir) -> Dir {
    match bdir {
        bananagraph::Dir::North => Dir::North,
        bananagraph::Dir::South => Dir::South,
        bananagraph::Dir::East => Dir::East,
        bananagraph::Dir::West => Dir::West,
    }
}
