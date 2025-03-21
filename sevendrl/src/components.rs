use cgmath::Vector2;
use doryen_fov::{FovAlgorithm, FovRecursiveShadowCasting, MapData};
use hecs::World;
use tinyrand::Rand;
use bananagraph::{DrawingContext, Sprite};
use crate::animation::BreatheAnimation;
use crate::enemy::{Dazed, Enemy, EnemyType};
use crate::inventory::{EnergyPotion, Give, Grabbable, HealthPotion, Scroll, ScrollType};
use crate::sprites::{AnimationSprites, Items, MapCells, SpriteFor};
use crate::status_bar::set_message;
use crate::terrain::{Opaque, Solid};

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
            let sprite = if sprite.z == 0.0 { sprite.with_z(0.8) } else { *sprite };
            sprites.push(dc.place(sprite, local_coords));

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

        for (_, (OnMap { location, .. }, Enemy { awake, .. })) in world.query_mut::<(&mut OnMap, &mut Enemy)>().into_iter() {
            if fov_map.fov[location.x as usize + location.y as usize * 64usize] { // TODO don't hard code map size
                *awake = true
            }
        }
    }
}

pub fn map_data_for(world: &World, size: impl Into<Vector2<usize>>, player_loc: impl Into<Vector2<i32>>) -> MapData {
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
}

#[derive(Copy, Clone, Debug)]
pub enum Chest {
    HealthPotion,
    EnergyPotion,
    Crystal,
    Mushroom,
    Scroll(ScrollType),
    Mimic
}

#[derive(Copy, Clone, Debug)]
pub enum Powerup {
    Crystal,
    Mushroom
}

impl Chest {
    pub fn new_rand(rand: &mut dyn Rand) -> Self {
        match rand.next_u32() % 13 {
            0..=2 => Chest::HealthPotion,
            3..=5 => Chest::EnergyPotion,
            6 | 7 => Chest::Scroll(Scroll::new_rand(rand).0),
            8 | 9 => Chest::Mushroom,
            10 | 11 => Chest::Crystal,
            12 => Chest::Mimic,
            _ => unreachable!()
        }
    }

    pub fn try_bump(world: &mut World, new_loc: Vector2<i32>) {
        let maybe_chest = world.query::<(&Chest, &OnMap)>().iter().find_map(|(e, (&c, om))| {
                if om.location == new_loc { Some((e, c)) } else { None }
        });

        match maybe_chest {
            None => { } // There's not actually a chest here

            // If we've found a chest here, then bumping it has opened it. What's inside?
            // Potions, we just turn the chest into a potion:
            Some((ent, Chest::HealthPotion)) => {
                _ = world.remove::<(Solid, Chest)>(ent);
                world.insert(ent, (HealthPotion, Grabbable)).unwrap();
                world.query_one_mut::<&mut OnMap>(ent).unwrap().sprite = Items::HealthPotion.sprite();
                set_message(world, "The chest contained a health potion!");
            }

            Some((ent, Chest::EnergyPotion)) => {
                _ = world.remove::<(Solid, Chest)>(ent);
                world.insert(ent, (EnergyPotion, Grabbable)).unwrap();
                world.query_one_mut::<&mut OnMap>(ent).unwrap().sprite = Items::EnergyPotion.sprite();
                set_message(world, "The chest contained an energy potion!");
            }

            Some((ent, Chest::Scroll(scroll_type))) => {
                _ = world.remove::<(Solid, Chest)>(ent);
                let scroll = Scroll(scroll_type);
                world.insert(ent, (scroll, Grabbable)).unwrap();
                world.query_one_mut::<&mut OnMap>(ent).unwrap().sprite = scroll.inventory_attrs().1;
                set_message(world, "The chest contained a scroll!");
            }

            // Mimic!
            Some((ent, Chest::Mimic)) => {
                _ = world.remove::<(Chest,)>(ent);
                let breathe = BreatheAnimation::new(AnimationSprites::mimic_breathe());
                // All mimics start dazed, so we get one turn to react
                world.insert(ent, (breathe, Enemy { awake: true, enemy_type: EnemyType::Mimic }, Dazed)).unwrap();
                set_message(world, "That wasn't a chest, it was a mimic!");
            }

            // Powerups
            Some((ent, Chest::Crystal)) => {
                _ = world.remove::<(Chest,Solid)>(ent);
                world.query_one_mut::<&mut OnMap>(ent).unwrap().sprite = Items::Crystal.sprite();
                world.insert(ent, (Grabbable, Powerup::Crystal)).unwrap();
            }
            Some((ent, Chest::Mushroom)) => {
                _ = world.remove::<(Chest,Solid)>(ent);
                world.query_one_mut::<&mut OnMap>(ent).unwrap().sprite = Items::Mushroom.sprite();
                world.insert(ent, (Grabbable, Powerup::Mushroom)).unwrap();
            }
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Stairs;