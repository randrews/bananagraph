use std::time::Duration;
use cgmath::{Point2, Vector2};
use hecs::{Entity, Query, World};
use log::info;
use tinyrand::{Rand, Seeded, Xorshift};
use bananagraph::{GpuWrapper, IdBuffer, Sprite, Typeface, TypefaceBuilder, WindowEventHandler};
use grid::{create_bsp_map, CellType, Coord, Dir, Grid, VecGrid};
use crate::animation::BreatheAnimation;
use crate::components::{Enemy, OnMap, Player};
use crate::door::Door;
use crate::status_bar::StatusBar;
use crate::terrain::{recreate_terrain, Wall};

#[derive(Default)]
pub struct GameState {
    pub(crate) world: World,
    pub(crate) rand: Xorshift,
    pub(crate) typeface: Option<Typeface>
}

impl WindowEventHandler for GameState {
    fn init(&mut self, wrapper: &mut GpuWrapper) {
        wrapper.add_texture(include_bytes!("Dungeon.png"), Some("Dungeon.png"));
        wrapper.add_texture(include_bytes!("Heroes-Animated.png"), Some("Heroes-Animated.png"));
        wrapper.add_texture(include_bytes!("Frames.png"), Some("Frames.png"));
        wrapper.add_texture(include_bytes!("Icons.png"), Some("Icons.png"));
        wrapper.add_texture(include_bytes!("Monsters-Animated.png"), Some("Monsters-Animated.png"));

        let mut builder = TypefaceBuilder::new(include_bytes!("Curly-Girly.png"), [0, 0, 0, 0xff], 4, 13);
        builder.add_glyphs("ABCDEFGH", (7, 15), (1, 1), Some(1));
        builder.add_glyphs("IJKLMNOP", (7, 15), (1, 17), Some(1));
        builder.add_glyphs("QRSTUVWX", (7, 15), (1, 33), Some(1));
        builder.add_glyphs("YZ", (7, 15), (1, 49), Some(1));

        builder.add_glyphs("abcdefgh", (7, 15), (1, 65), Some(1));
        builder.add_glyphs("ijklmnop", (7, 15), (1, 81), Some(1));
        builder.add_glyphs("qrstuvwx", (7, 15), (1, 97), Some(1));
        builder.add_glyphs("yz", (7, 15), (1, 113), Some(1));

        builder.add_glyphs("01234567", (7, 15), (1, 129), Some(1));
        builder.add_glyphs("89", (7, 15), (1, 145), Some(1));

        builder.add_glyphs("!~#$%&'", (7, 15), (9, 161), Some(1));
        builder.add_glyphs("()*+,-./", (7, 15), (1, 177), Some(1));
        builder.add_glyphs(":;<=>?[]", (7, 15), (1, 193), Some(1));
        builder.add_glyphs("\\^_`{}|", (7, 15), (1, 209), Some(1));
        builder.add_glyphs("@", (7, 15), (1, 225), Some(1));

        builder.set_x_offset('p', -3);
        builder.set_x_offset('j', -3);
        builder.add_sized_glyph(' ', (3, 1), (17, 113));
        self.typeface = Some(builder.into_typeface(wrapper));
    }

    fn redraw(&self, _mouse_pos: Point2<f64>, wrapper: &GpuWrapper) -> Option<IdBuffer> {
        let mut sprites = OnMap::system(&self.world);
        sprites.append(&mut StatusBar::system(&self.world, self.typeface.as_ref().unwrap()));
        wrapper.redraw_with_ids(sprites).ok()
    }

    fn tick(&mut self, dt: Duration) {
        BreatheAnimation::system(&mut self.world, dt)
    }

    fn arrow_key(&mut self, dir: bananagraph::Dir) {
        self.walk(convert_dir(dir))
    }
}

impl GameState {
    pub fn seed(&mut self, seed: u64) {
        info!("seed: {}", seed);
        self.rand = Xorshift::seed(seed)
    }

    pub fn set_map(&mut self, map: VecGrid<CellType>) {
        self.spawn_enemies(&map, 500);
        recreate_terrain(map, &mut self.world);
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
            Player::default(),
            OnMap { location, sprite: frames[0] },
            BreatheAnimation::new(frames)
        ));
    }

    pub fn spawn_enemies(&mut self, map: &VecGrid<CellType>, count: u32) {
        let frames = [
            Sprite::new((64, 16), (16, 16)).with_layer(4),
            Sprite::new((80, 16), (16, 16)).with_layer(4),
            Sprite::new((96, 16), (16, 16)).with_layer(4),
            Sprite::new((96, 16), (16, 16)).with_layer(4),
            Sprite::new((96, 16), (16, 16)).with_layer(4),
            Sprite::new((80, 16), (16, 16)).with_layer(4)
        ];

        for _ in 0..count {
            let loc = map.random_satisfying(|| { self.rand.next_usize() }, |c| map[c] == CellType::Clear);
            self.world.spawn((
                Enemy {},
                OnMap { sprite: frames[0], location: loc },
                BreatheAnimation::new_with_start(frames.to_vec(), Duration::from_millis(self.rand.next_u64()))
            ));
        }
    }

    pub fn create_status_bar(&mut self) {
        self.world.spawn((StatusBar { message: String::from("Welcome!") },));
    }

    fn find_on_map<Q: Query>(&mut self, loc: impl Into<Vector2<i32>>) -> Vec<(Entity, <Q as Query>::Item<'_>)> {
        let loc = loc.into();
        self.world.query_mut::<(Q, &OnMap)>().into_iter()
            .filter_map(|(e, (q, on_map))| {
                if on_map.location == loc {
                    Some((e, q))
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

        // First check for walls:
        if self.exists_on_map::<&Wall>(new_loc) { return }

        // Now bump doors:
        let can_move = Door::try_move(self, new_loc);

        // If all the bumps let us through, actually move:
        if can_move {
            self.get_player::<&mut OnMap>().location = new_loc;
        }
    }

    // Gotta shut clippy up about this because it's only called in a fn that's only visible
    // to wasm32.
    #[allow(dead_code)]
    pub fn new(seed: u64) -> Self {
        let mut game_state = Self::default();
        game_state.seed(seed);
        let map = create_bsp_map((64, 64), 6, &mut game_state.rand);
        game_state.set_map(map);
        game_state.set_player((4, 2));
        game_state.create_status_bar();
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
