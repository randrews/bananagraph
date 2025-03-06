use cgmath::Vector2;
use hecs::{Entity, World};
use log::info;
use bananagraph::{DrawingContext, Sprite, Typeface};
use crate::components::Player;
use crate::sprites::UiFrame;
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

pub fn next_inventory_idx(world: &World) -> usize {
    // Find the max index of all the inventory items:
    let indices = world.query::<&InventoryItem>().into_iter().map(|(_, i)| i.index).collect::<Vec<_>>();
    *indices.iter().max().unwrap_or(&0usize)
}

pub fn next_inventory_key(world: &World) -> char {
    let used_keys = world.query::<&InventoryItem>().into_iter().filter_map(|(_, i)| i.key).collect::<Vec<char>>();
    let possible = "abcdefghijklmnopqrstuvwxyz";
    for c in possible.chars() {
        if used_keys.contains(&c) { continue }
        return c
    }
    unreachable!()
}

pub fn inventory_item_for_key(world: &World, key: char) -> Option<Entity> {
    world.query::<&InventoryItem>().into_iter().find_map(|(ent, i)| {
        if i.key == Some(key) {
            Some(ent)
        } else {
            None
        }
    })
}

pub fn activate_item(world: &mut World, item: Entity) {
    let result = world.query_one_mut::<(Option<&mut HealthPotion>,)>(item).unwrap();

    // If it's a health pot:
    if let Some(_) = result.0 {
        let (_, player) = world.query_mut::<&mut Player>().into_iter().next().unwrap();
        player.health = player.max_health.min(player.health + 3);
        world.despawn(item).unwrap();
        set_message(world, "Drank health potion")
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct HealthPotion {}
