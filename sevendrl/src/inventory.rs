use cgmath::Vector2;
use hecs::{Component, Entity, World};
use tinyrand::Rand;
use bananagraph::{DrawingContext, Sprite, Typeface};
use crate::components::{OnMap, Player, Powerup};
use crate::scrolls::{leap_scroll, phasewalk_scroll, shove_scroll};
use crate::sprites::{Items, SpriteFor, UiFrame};
use crate::status_bar::{set_message, EquippedAbilities};

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
        self.compress_indices();
    }

    fn compress_indices(&mut self) {
        let world = self.world_mut();
        let mut indices: Vec<_> = world.query_mut::<&mut InventoryItem>().into_iter().collect();
        indices.sort_by(|a, b| a.1.index.cmp(&b.1.index));

        for (n, (ent, ii)) in indices.into_iter().enumerate() {
            ii.index = n;
        }
    }

    fn inventory_full(&mut self) -> bool {
        let world = self.world();
        // Inventory limit is a dozen
        world.query::<&InventoryItem>().iter().count() >= 12
    }

    fn has_scroll_of_type(&self, scroll_type: ScrollType) -> bool {
        for (_, (ii, Scroll(st))) in self.world().query::<(Option<&InventoryItem>, &Scroll)>().iter() {
            if ii.is_some() && *st == scroll_type { return true }
        }

        if let Some((_, &ea)) = self.world().query::<&EquippedAbilities>().iter().next() {
            if let Some(s) = ea.slot1 {
                if self.world().query_one::<&Scroll>(s).unwrap().get().unwrap().0 == scroll_type { return true }
            }
            if let Some(s) = ea.slot2 {
                if self.world().query_one::<&Scroll>(s).unwrap().get().unwrap().0 == scroll_type { return true }
            }
            if let Some(s) = ea.slot3 {
                if self.world().query_one::<&Scroll>(s).unwrap().get().unwrap().0 == scroll_type { return true }
            }
        }
        false
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
    Scroll::try_activate(world, item);
}

trait TryActivate where Self: Sized + Component {
    fn activate(world: &mut World, entity: Entity);
    fn try_activate(world: &mut World, ent: Entity) {
        if let Ok((Some(_),)) = world.query_one_mut::<(Option<&Self>,)>(ent) {
            Self::activate(world, ent)
        }
    }
}

pub trait Give where Self: Sized + Component {
    fn inventory_attrs(&self) -> (&str, Sprite);
    fn give(self, world: &mut World) {
        let (name, sprite) = self.inventory_attrs();
        let i = world.add_to_inventory(name, sprite);
        let _ = world.insert(i, (self,));
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct HealthPotion;

impl Give for HealthPotion {
    fn inventory_attrs(&self) -> (&str, Sprite) {
        ("Potion", Items::HealthPotion.sprite())
    }
}

impl TryActivate for HealthPotion {
    fn activate(world: &mut World, entity: Entity) {
        let (_, player) = world.query_mut::<&mut Player>().into_iter().next().unwrap();
        player.health = player.max_health.min(player.health + 4);
        world.consume_from_inventory(entity);
        set_message(world, "Drank health potion");
    }
}

#[derive(Default, Debug, Clone, Copy)]
pub struct EnergyPotion;

impl TryActivate for EnergyPotion {
    fn activate(world: &mut World, entity: Entity) {
        let (_, player) = world.query_mut::<&mut Player>().into_iter().next().unwrap();
        player.energy = player.max_energy.min(player.energy + 3);
        world.consume_from_inventory(entity);
        set_message(world, "Drank energy potion");
    }
}

impl Give for EnergyPotion {
    fn inventory_attrs(&self) -> (&str, Sprite) {
        ("Energy Potion", Items::EnergyPotion.sprite())
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ScrollType {
    PhaseWalk,
    Leap,
    Shove,
}

#[derive(Debug, Clone, Copy)]
pub struct Scroll(pub ScrollType);

impl Give for Scroll {
    fn inventory_attrs(&self) -> (&str, Sprite) {
        match self.0 {
            ScrollType::PhaseWalk => ("Phase Walk", Items::Scroll1.sprite()),
            ScrollType::Leap => ("Leap", Items::Scroll2.sprite()),
            ScrollType::Shove => ("Shove", Items::Scroll3.sprite()),
        }
    }
}

impl Scroll {
    pub fn new_rand(rand: &mut dyn Rand) -> Self {
        match rand.next_u32() % 3 {
            0 => Scroll(ScrollType::Shove),
            1 => Scroll(ScrollType::Leap),
            2 => Scroll(ScrollType::PhaseWalk),
            _ => unreachable!()
        }
    }

    pub fn equip_slot(&self) -> i32 {
        match self.0 {
            ScrollType::PhaseWalk => 2,
            ScrollType::Leap => 0,
            ScrollType::Shove => 0,
        }
    }

    pub fn perform(&self, world: &mut World, rand: &mut impl Rand) {
        match self.0 {
            ScrollType::PhaseWalk => phasewalk_scroll(world),
            ScrollType::Leap => leap_scroll(world, rand),
            ScrollType::Shove => shove_scroll(world)
        }
    }

    pub fn cost(&self) -> u32 {
        match self.0 {
            ScrollType::PhaseWalk => 5,
            ScrollType::Leap => 1,
            ScrollType::Shove => 2
        }
    }
}

impl TryActivate for Scroll {
    fn activate(world: &mut World, entity: Entity) {
        let scroll = *world.query_one::<&Scroll>(entity).unwrap().get().unwrap();
        if let Some((_, equipped)) = world.query_mut::<&mut EquippedAbilities>().into_iter().next() {
            // what was already in the slot?
            let existing =
                match scroll.equip_slot() {
                    0 => equipped.slot1,
                    1 => equipped.slot2,
                    2 => equipped.slot3,
                    _ => equipped.slot1,
                };

            // put it in the slot
            match scroll.equip_slot() {
                0 => equipped.slot1 = Some(entity),
                1 => equipped.slot2 = Some(entity),
                2 => equipped.slot3 = Some(entity),
                _ => equipped.slot1 = Some(entity),
            }

            // remove the new one from the inventory
            world.remove::<(InventoryItem,)>(entity).unwrap();
            world.compress_indices();

            // If there was an old one, put it in the inventory:
            if let Some(old) = existing {
                let scroll = world.query_one_mut::<&Scroll>(old).unwrap();
                scroll.give(world)
            }
        }
    }
}

pub fn activate_ability(world: &mut World, slot: char, rand: &mut impl Rand) {
    // First, figure out what we're actually wanting to do:
    let equipped = *world.query::<&EquippedAbilities>().iter().next().unwrap().1;
    let scroll_ent = match slot {
        '1' => equipped.slot1,
        '2' => equipped.slot2,
        '3' => equipped.slot3,
        _ => equipped.slot1
    };

    if scroll_ent.is_none() {
        set_message(world, format!("No ability in slot {}", slot).as_str());
        return;
    }

    let scroll = *world.query_one::<&Scroll>(scroll_ent.unwrap()).unwrap().get().unwrap();
    scroll.perform(world, rand);
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Grabbable;

impl Grabbable {
    pub fn try_grab(world: &mut World, location: Vector2<i32>) {
        let maybe_grab = world.query::<(&Grabbable, &OnMap)>()
            .iter().find_map(|(ent, (_, om))| {
            if om.location == location { Some(ent) } else { None } });
        if let Some(ent) = maybe_grab {
            if let Some(&pu) = world.query_one_mut::<Option<&Powerup>>(ent).unwrap() {
                let player = world.query_mut::<&mut Player>().into_iter().next().unwrap().1;
                match pu {
                    Powerup::Mushroom => {
                        // remember one heart is two health
                        player.max_health = 24.min(player.max_health + 2);
                        player.health = player.max_health.min(player.health + 2);
                        set_message(world, "Eating the mushroom makes you feel stronger!")
                    }
                    Powerup::Crystal => {
                        player.max_energy = 12.min(player.max_energy + 1);
                        player.energy = player.max_energy.min(player.energy + 1);
                        set_message(world, "Gazing into the crystal, you feel more magically attuned!")
                    }
                }
                world.despawn(ent).unwrap()
            } else {
                if world.inventory_full() {
                    set_message(world, "Your inventory is full");
                    return
                }
                world.remove::<(OnMap, Grabbable)>(ent).unwrap();

                if let Some(&hp) = world.query_one_mut::<Option<&HealthPotion>>(ent).unwrap() {
                    hp.give(world);
                } else if let Some(&ep) = world.query_one_mut::<Option<&EnergyPotion>>(ent).unwrap() {
                    ep.give(world);
                } else if let Some(&sc) = world.query_one_mut::<Option<&Scroll>>(ent).unwrap() {
                    if world.has_scroll_of_type(sc.0) {
                        set_message(world, "You already have this scroll, so your focus increases");
                        let player = world.query_mut::<&mut Player>().into_iter().next().unwrap().1;
                        player.energy += 1;
                        player.max_energy += 1;
                    } else {
                        sc.give(world)
                    }
                }
            }
        }
    }
}