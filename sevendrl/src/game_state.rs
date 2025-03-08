use std::collections::HashSet;
use std::time::Duration;
use cgmath::{Point2, Vector2};
use hecs::{Entity, Query, World};
use log::info;
use tinyrand::{Rand, Seeded, Xorshift};
use wgpu::CompositeAlphaMode::Opaque;
use bananagraph::{GpuWrapper, IdBuffer, Sprite, Typeface, TypefaceBuilder, WindowEventHandler};
use grid::{create_bsp_map, CellType, Coord, Dir, Grid, VecGrid};
use crate::animation::{BreatheAnimation, OneShotAnimation};
use crate::components::{Chest, OnMap, Player};
use crate::door::Door;
use crate::enemy::Enemy;
use crate::inventory::{activate_ability, activate_item, EnergyPotion, Give, Grabbable, HealthPotion, Inventory, InventoryWorld, Scroll, ScrollType};
use crate::modal::{ContentType, DismissType, Modal};
use crate::sprites::{AnimationSprites, Items, SpriteFor};
use crate::status_bar::{EquippedAbilities, StatusBar};
use crate::terrain::{recreate_terrain, Solid};

enum KeyPress<'a> {
    Enter,
    Esc,
    Letter(&'a str),
    Arrow(Dir),
}

#[derive(Default)]
pub struct GameState {
    pub(crate) world: World,
    pub(crate) rand: Xorshift,
    pub(crate) typeface: Option<Typeface>,
}

impl WindowEventHandler for GameState {
    fn init(&mut self, wrapper: &mut GpuWrapper) {
        wrapper.add_texture(include_bytes!("Dungeon.png"), Some("Dungeon.png"));
        wrapper.add_texture(include_bytes!("Heroes-Animated.png"), Some("Heroes-Animated.png"));
        wrapper.add_texture(include_bytes!("Frames.png"), Some("Frames.png"));
        wrapper.add_texture(include_bytes!("Icons.png"), Some("Icons.png"));
        wrapper.add_texture(include_bytes!("Monsters-Animated.png"), Some("Monsters-Animated.png"));
        wrapper.add_texture(include_bytes!("Items.png"), Some("Items.png"));

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
        builder.set_right_offset('q', -3);
        builder.add_sized_glyph(' ', (3, 1), (17, 113));
        self.typeface = Some(builder.into_typeface(wrapper));
    }

    fn redraw(&self, _mouse_pos: Point2<f64>, wrapper: &GpuWrapper) -> Option<IdBuffer> {
        let mut sprites = OnMap::system(&self.world);
        let tf = self.typeface.as_ref().unwrap();
        sprites.append(&mut StatusBar::system(&self.world, tf));
        sprites.append(&mut Inventory::system(&self.world, tf));
        sprites.append(&mut Modal::system(&self.world, tf));
        wrapper.redraw_with_ids(sprites).ok()
    }

    fn tick(&mut self, dt: Duration) {
        BreatheAnimation::system(&mut self.world, dt);
        OneShotAnimation::system(&mut self.world, dt);
    }

    fn letter_key(&mut self, letter: &str) {
        self.handle_key(KeyPress::Letter(letter))
    }

    fn enter_key(&mut self) {
        self.handle_key(KeyPress::Enter)
    }

    fn esc_key(&mut self) {
        self.handle_key(KeyPress::Esc)
    }

    fn arrow_key(&mut self, dir: bananagraph::Dir) {
        self.handle_key(KeyPress::Arrow(convert_dir(dir)))
    }
}

impl GameState {
    fn handle_key(&mut self, key: KeyPress) {
        // if a modal is up, that gets first crack:
        if let Some((ent, modal)) = self.world.query_mut::<&Modal>().into_iter().next() {
            // We pressed something, kill it.
            if modal.dismiss == DismissType::Any {
                self.world.despawn(ent).unwrap()
            }
        } else {
            // First, is there a one-shot animation going? Let's ignore input until it finishes:
            if self.world.query::<&OneShotAnimation>().iter().next().is_some() {
                return
            }

            match key {
                KeyPress::Letter("?") => {
                    self.create_help_modal()
                }
                KeyPress::Arrow(dir) => {
                    self.walk(dir);
                    Enemy::system(&mut self.world)
                }
                KeyPress::Letter(s) => {
                    let c = s.chars().next().unwrap();
                    if let Some(ent) = self.world.inventory_item_for_key(c) {
                        activate_item(&mut self.world, ent);
                        Enemy::system(&mut self.world)
                    } else if c == '1' || c == '2' || c == '3' {
                        activate_ability(&mut self.world, c, &mut self.rand)
                    }
                }
                _ => {}
            }
        }
    }

    pub fn seed(&mut self, seed: u64) {
        info!("seed: {}", seed);
        self.rand = Xorshift::seed(seed)
    }

    pub fn set_map(&mut self, map: VecGrid<CellType>) {
        self.spawn_enemies(&map, 100);
        recreate_terrain(map, &mut self.world);
        self.spawn_treasure(35);
    }

    pub fn set_player(&mut self) {
        // Remove the old player
        let player = self.world.query::<&Player>().iter().map(|(e, _)| e).next();
        player.map(|e| self.world.despawn(e));

        let location = self.random_spots(1)[0];

        // Spawn a new player
        self.world.spawn((
            Player::default(),
            Solid {},
            OnMap { location, sprite: AnimationSprites::Player1.sprite() },
            BreatheAnimation::new(AnimationSprites::player_breathe())
        ));
    }

    /// Generate a list of `count` random spots in the grid that don't yet have
    /// any Solid + OnMap entities in them.
    pub fn random_spots(&mut self, count: usize) -> Vec<Vector2<i32>> {
        let mut spots = vec![];
        // Make a list of all the places the player can't be spawned:
        let filled_cells: Vec<_> = self.world.query::<(&OnMap, &Solid)>().iter().map(|(_, (om, _))| om.location).collect();

        // Find a random spot not on the list
        while spots.len() < count {
            // TODO don't hardcode map dimensions
            let candidate: Vector2<_> = ((self.rand.next_u32() % 64) as i32, (self.rand.next_u32() % 64) as i32).into();
            if !filled_cells.contains(&candidate) && !spots.contains(&candidate) {
                spots.push(candidate)
            }
        }

        spots
    }

    pub fn spawn_treasure(&mut self, count: usize) {
        for spot in self.random_spots(35) {
            self.world.spawn((
                OnMap { location: spot, sprite: Items::Chest.sprite() },
                Solid,
                Opaque,
                Chest::new_rand(&mut self.rand),
            ));
        }
    }

    pub fn spawn_enemies(&mut self, map: &VecGrid<CellType>, count: u32) {
        // Delete the old enemies
        let ents = self.world.query_mut::<&Enemy>().into_iter().map(|(e, _)| e).collect::<Vec<_>>();
        for e in ents { self.world.despawn(e).unwrap() }

        // A set of every place we've spawned an enemy
        let mut enemy_locs = HashSet::new();
        for _ in 0..count {
            let loc = map.random_satisfying(|| { self.rand.next_usize() }, |c| map[c] == CellType::Clear && !enemy_locs.contains(&c));
            self.world.spawn((
                Enemy::default(),
                Solid {},
                OnMap { sprite: AnimationSprites::Enemy1.sprite(), location: loc },
                BreatheAnimation::new_with_start(AnimationSprites::enemy_breathe(), Duration::from_millis(self.rand.next_u64()))
            ));
            enemy_locs.insert(loc);
        }
    }

    pub fn create_status_bar(&mut self) {
        self.world.spawn((
            StatusBar { message: String::from("Welcome! Press ? for help.") },
            EquippedAbilities::default(),
        ));
    }

    pub fn create_inventory(&mut self) {
        self.world.spawn((Inventory {},));
        HealthPotion.give(&mut self.world);
        EnergyPotion.give(&mut self.world);

        Scroll(ScrollType::Shove).give(&mut self.world);
        Scroll(ScrollType::Leap).give(&mut self.world);
    }

    // fn find_on_map<Q: Query>(&mut self, loc: impl Into<Vector2<i32>>) -> Vec<(Entity, <Q as Query>::Item<'_>)> {
    //     let loc = loc.into();
    //     self.world.query_mut::<(Q, &OnMap)>().into_iter()
    //         .filter_map(|(e, (q, on_map))| {
    //             if on_map.location == loc {
    //                 Some((e, q))
    //             } else {
    //                 None
    //             }
    //         }).collect()
    // }

    fn find_entities_on_map<Q: Query>(&self, loc: impl Into<Vector2<i32>>) -> Vec<Entity> {
        let loc = loc.into();
        self.world.query::<(Q, &OnMap)>().into_iter()
            .filter_map(|(e, (_, on_map))| {
                if on_map.location == loc {
                    Some(e)
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

        // First check if it's passable:
        let can_move = !self.exists_on_map::<&Solid>(new_loc);

        // Even if we can't move there, if there's a door, bump it:
        Door::try_bump(&mut self.world, new_loc);

        // Also bump chests:
        Chest::try_bump(&mut self.world, new_loc);

        // If all the bumps let us through, actually move:
        if can_move {
            self.get_player::<&mut OnMap>().location = new_loc;

            // If there's an enemy in the space beyond our new_loc, splat it:
            let beyond = new_loc.translate(dir);
            if let Some(&ent) = self.find_entities_on_map::<&Enemy>(beyond).first() {
                // What animation should we show?
                let anim = self.world.query_one::<&Enemy>(ent).unwrap().get().unwrap().death_animation();
                self.world.despawn(ent).unwrap(); // Kill the enemy
                // Give the player some energy as a reward
                if let Some((_, player)) = self.world.query_mut::<&mut Player>().into_iter().next() {
                    player.energy = (player.energy + 1).min(player.max_energy)
                }
                // Spawn a one-shot showing the enemy fading
                let frame = anim.current_frame().unwrap();
                self.world.spawn((
                    anim,
                    OnMap { location: beyond, sprite: frame }
                    ));
            }

            // Try to grab things if things are there:
            Grabbable::try_grab(&mut self.world, new_loc);
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
        game_state.set_player();
        game_state.create_status_bar();
        game_state.create_inventory();
        game_state.create_intro_modal();
        game_state
    }

    fn create_intro_modal(&mut self) {
        self.world.spawn((Modal::new((15, 9), vec![
            ContentType::Center(String::from("Welcome, Adventurer!")),
            ContentType::Text(["You aspire to be one of the fabled Monks of",
                "Sevendral! To prove your honor to the order,",
                "you are to search this dungeon for the fabled",
                "Amulet of Sevendral, which some careless",
                "butterfingers apparently dropped down here.",
                "",
                "Good luck!"].join("\n")),
            ContentType::Center(String::from("-= press any key =-")),
        ], DismissType::Any),));
    }

    fn create_help_modal(&mut self) {
        self.world.spawn((Modal::new((25, 15), vec![
            ContentType::Center(String::from("How to play")),
            ContentType::Text([
                "- Use arrow keys to walk through the dungeon. Like all Monks of Sevendral,",
                "  you have taken a solemn vow never to move diagonally (your enemies, of",
                "  course, can and will, as they lack honor).",
                "",
                "- Move toward enemies from two spaces away to attack. Each one you slay",
                "  increases your energy focus, which can be used to perform abilities.",
            ].join("\n")),
            ContentType::CenterSprite(Sprite::new((154, 0), (48, 32)).with_layer(2)),
            ContentType::Text([
                "- Ability scrolls allow special moves and combos. Activate equipped abilities",
                "  with [1], [2], or [3]"
            ].join("\n")),
            ContentType::CenterSprite(Sprite::new((64, 112), (16, 16)).with_layer(5)),
            ContentType::Text([
                "- You can carry other items in your inventory and activate them with other",
                "  keys.",
                ""
            ].join("\n")),
            ContentType::Center(String::from("-= press any key =-")),
        ], DismissType::Any),));
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
