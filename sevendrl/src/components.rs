use cgmath::Vector2;
use doryen_fov::{FovAlgorithm, FovRecursiveShadowCasting, MapData};
use hecs::World;
use bananagraph::{DrawingContext, Sprite};
use crate::enemy::Enemy;
use crate::sprites::{MapCells, SpriteFor};
use crate::terrain::Opaque;

#[derive(Copy, Clone, Debug)]
pub struct OnMap {
    pub location: Vector2<i32>,
    pub sprite: Sprite
}

pub fn player_loc(world: &World) -> Vector2<i32> {
    world.query::<(&Player, &OnMap)>().into_iter().next().map(|(_, (_, onmap))| onmap.location).unwrap()
}

impl OnMap {
    pub fn system(world: &World) -> Vec<Sprite> {
        let dc = DrawingContext::new((960.0 / 2.0, 544.0 / 2.0));
        let mut sprites = vec![];

        // We have a 480 x 270 "pixel" area, which with 16x16 tiles means 30x16.875 tiles
        // We want an odd-numbered square centered on the player, with room on the bottom and left
        // for status bar and inventory.
        // - Let's say that height-wise we'll be 13 tiles tall, so 208 px, leaving
        // (540 / 2) - (13 * 16) = 62 px for status bar
        // Width we'll say 21 wide, leaving (960 / 2) - (21 * 16) = 144 for inventory
        let player_loc = player_loc(world);
        let topleft = Vector2::new(player_loc.x - 10, player_loc.y - 6);
        let size = Vector2::new(21, 13);
        let inv_width = (960.0 / 2.0) - (21.0 * 16.0);

        // First let's do some fov work:
        let fov_map = map_data_for(world, (64, 64), player_loc);
        let fog = MapCells::Fog.sprite().with_z(0.7);

        for (_, (on_map,)) in world.query::<(&OnMap,)>().iter() {
            let OnMap { location, sprite } = on_map;
            // Skip things not in the region
            if location.x < topleft.x || location.y < topleft.y || location.x >= topleft.x + size.x || location.y >= topleft.y + size.y {
                continue
            }

            let local_coords = Vector2::new(
                (location.x - topleft.x) as f32 * 16.0 + inv_width,
                (location.y - topleft.y) as f32 * 16.0
            );
            sprites.push(dc.place(*sprite, local_coords).with_z(0.8));

            // If this isn't in fov, plant an opaque fog sprite on top of it:
            if !fov_map.fov[location.x as usize + location.y as usize * 64usize] { // TODO don't hard code map size
                sprites.push(dc.place(fog, local_coords))
            }
        }
        sprites
    }

    pub fn awaken_enemies(world: &mut World) {
        let player_loc = player_loc(world);
        let fov_map = map_data_for(world, (64, 64), player_loc);

        for (_, (OnMap { location, .. }, Enemy { awake })) in world.query_mut::<(&mut OnMap, &mut Enemy)>().into_iter() {
            if fov_map.fov[location.x as usize + location.y as usize * 64usize] { // TODO don't hard code map size
                *awake = true
            }
        }
    }
}

fn map_data_for(world: &World, size: impl Into<Vector2<usize>>, player_loc: impl Into<Vector2<i32>>) -> MapData {
    let (player_loc, size) = (player_loc.into(), size.into());
    let mut map = MapData::new(size.x, size.y);

    for (_, (onmap, opaque)) in world.query::<(&OnMap, Option<&Opaque>)>().into_iter() {
        let (x, y) = onmap.location.into();
        map.transparent[x as usize + (y * size.x as i32) as usize] = opaque.is_none()
    }
    FovRecursiveShadowCasting::new().compute_fov(&mut map, player_loc.x as usize, player_loc.y as usize, 20, true);
    map
}

#[derive(Copy, Clone, Debug)]
pub struct Player {
    pub energy: u32,
    pub health: u32,
    pub max_health: u32,
    pub max_energy: u32
}

impl Default for Player {
    fn default() -> Self {
        Self {
            energy: 0,
            health: 10,
            max_energy: 5,
            max_health: 10
        }
    }
}

impl Player {
    pub fn give_energy(&mut self, delta: u32) {
        self.energy = self.max_energy.min(self.energy + delta)
    }
    pub fn give_health(&mut self, delta: u32) {
        self.health = self.max_health.min(self.health + delta)
    }
}