use cgmath::Vector2;
use hecs::{Component, Entity, World};
use bananagraph::{DrawingContext, Sprite, Typeface};
use crate::components::Player;
use crate::sprites::{Items, SpriteFor, UiFrame};
use crate::status_bar::set_message;

#[derive(Clone)]
pub struct Inventory {}

#[derive(Clone)]
pub struct InventoryItem {
    pub name: String,
    pub sprite: Sprite,
    pub index: usize,
    pub key: Option<char>
}

impl Inventory {
    pub fn system(world: &World, typeface: &Typeface) -> Vec<Sprite> {
        let dc = DrawingContext::new((960.0 / 2.0, 544.0 / 2.0));
        let mut sprites = UiFrame::draw_frame(dc, (0.0, 0.0), (9, 13), 0.9);

        for (_, item) in world.query::<&InventoryItem>().into_iter() {
            sprites.append(&mut Self::draw_item(dc, typeface, item));
        }
        sprites
    }

    fn draw_item(dc: DrawingContext, typeface: &Typeface, item: &InventoryItem) -> Vec<Sprite> {
        let topleft = Vector2::new(8.0, 8.0 + 16.0 * item.index as f32);
        let mut sprites = typeface.print(dc, topleft + Vector2::new(20.0, typeface.height as f32), 0.8, item.name.as_str());
        sprites.push(dc.place(item.sprite, topleft));
        if let Some(key) = item.key {
            let s = format!("[{}]", key);
            let width = typeface.width(s.as_str());
            let txtright = topleft + Vector2::new(8.0 * 16.0 - 4.0 - width, typeface.height as f32);
            sprites.append(&mut typeface.print(dc, txtright, 0.8, s.as_str()))
        }
        sprites
    }
}

pub trait InventoryWorld {
    fn world(&self) -> &World;
    fn world_mut(&mut self) -> &mut World;

    fn next_inventory_idx(&self) -> usize {
        // Find the max index of all the inventory items:
        let indices = self.world().query::<&InventoryItem>().into_iter().map(|(_, i)| i.index).collect::<Vec<_>>();
        indices.iter().max().map_or(0usize, |m| m + 1)
    }

    fn next_inventory_key(&self) -> char {
        let used_keys = self.world().query::<&InventoryItem>().into_iter().filter_map(|(_, i)| i.key).collect::<Vec<char>>();
        let possible = "abcdefghijklmnopqrstuvwxyz";
        for c in possible.chars() {
            if used_keys.contains(&c) { continue }
            return c
        }
        unreachable!()
    }

    fn inventory_item_for_key(&self, key: char) -> Option<Entity> {
        self.world().query::<&InventoryItem>().into_iter().find_map(|(ent, i)| {
            if i.key == Some(key) {
                Some(ent)
            } else {
                None
            }
        })
    }

    fn add_to_inventory(&mut self, name: &str, sprite: Sprite) -> Entity {
        let inv = InventoryItem {
            name: String::from(name),
            sprite,
            index: self.next_inventory_idx(),
            key: Some(self.next_inventory_key())
        };
        self.world_mut().spawn((inv,))
    }

    fn consume_from_inventory(&mut self, entity: Entity) {
        let world = self.world_mut();
        world.despawn(entity).unwrap();

        let mut indices: Vec<_> = world.query_mut::<&InventoryItem>().into_iter().map(|(e, InventoryItem { index, ..})| (e, index)).collect();
        indices.sort_by(|a, b| a.1.cmp(b.1));

        for (n, (ent, _)) in indices.into_iter().enumerate() {
            world.query_one_mut::<&mut InventoryItem>(ent).unwrap().index = n;
        }
    }
}

impl InventoryWorld for World {
    fn world(&self) -> &World {
        self
    }

    fn world_mut(&mut self) -> &mut World {
        self
    }
}

pub fn activate_item(world: &mut World, item: Entity) {
    HealthPotion::try_activate(world, item);
    EnergyPotion::try_activate(world, item);
}

trait TryActivate where Self: Sized + Component {
    fn activate(world: &mut World, entity: Entity);
    fn try_activate(world: &mut World, ent: Entity) {
        if let Ok((Some(_),)) = world.query_one_mut::<(Option<&Self>,)>(ent) {
            Self::activate(world, ent)
        }
    }
}

pub trait Give where Self: Sized + Component + Default {
    fn give(world: &mut World);
    fn give_inner(world: &mut World, name: &str, sprite: Sprite) {
        let i = world.add_to_inventory(name, sprite);
        let _ = world.insert(i, (Self::default(),));
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct HealthPotion;

impl Give for HealthPotion {
    fn give(world: &mut World) {
        Self::give_inner(world, "Potion", Items::HealthPotion.sprite())
    }
}

impl TryActivate for HealthPotion {
    fn activate(world: &mut World, entity: Entity) {
        let (_, player) = world.query_mut::<&mut Player>().into_iter().next().unwrap();
        player.health = player.max_health.min(player.health + 3);
        world.consume_from_inventory(entity);
        set_message(world, "Drank health potion");
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct EnergyPotion;

impl TryActivate for EnergyPotion {
    fn activate(world: &mut World, entity: Entity) {
        let (_, player) = world.query_mut::<&mut Player>().into_iter().next().unwrap();
        player.energy = player.max_energy.min(player.energy + 2);
        world.consume_from_inventory(entity);
        set_message(world, "Drank energy potion");
    }
}

impl Give for EnergyPotion {
    fn give(world: &mut World) {
        Self::give_inner(world,"Energy Potion", Items::EnergyPotion.sprite());
    }
}