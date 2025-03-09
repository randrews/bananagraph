use cgmath::Vector2;
use hecs::{Entity, World};
use log::info;
use bananagraph::{DrawingContext, Sprite, Typeface};
use grid::Coord;
use crate::components::{player_loc, OnMap, Player, Stairs};
use crate::inventory::{Give, Scroll};
use crate::sprites::UiFrame;

#[derive(Clone)]
pub struct StatusBar {
    pub message: String
}

impl StatusBar {
    pub fn system(world: &World, typeface: &Typeface) -> Vec<Sprite> {
        let mut sprites = Self::frame_sprites();
        let dc = DrawingContext::new((960.0 / 2.0, 544.0 / 2.0));

        // Print the current status line
        if let Some((_, status_bar)) = world.query::<&StatusBar>().into_iter().next() {
            let coord = Self::tile_coord((0, 0)) + Vector2::new(0.0, 11.0);
            sprites.append(&mut typeface.print(dc, coord, 0.3, status_bar.message.as_str()));
        }

        if let Some((_, player)) = world.query::<&Player>().into_iter().next() {
            let energy_icons = (
                Sprite::new((96, 144), (16, 16)).with_z(0.5).with_layer(3),
                Sprite::new((64, 144), (16, 16)).with_z(0.5).with_layer(3)
                );

            let health_icons = (
                Sprite::new((160, 144), (16, 16)).with_z(0.5).with_layer(3),
                Sprite::new((144, 144), (16, 16)).with_z(0.5).with_layer(3),
                Sprite::new((128, 144), (16, 16)).with_z(0.5).with_layer(3)
            );

            let hleft = typeface.width("Health:");
            sprites.append(&mut typeface.print(dc, Self::tile_coord((0, 1)) + Vector2::new(0.0, 11.0), 0.3, "Health:"));
            let eleft = typeface.width("Energy:");
            sprites.append(&mut typeface.print(dc, Self::tile_coord((0, 2)) + Vector2::new(0.0, 11.0), 0.3, "Energy:"));
            let left = hleft.max(eleft);

            for n in 0u32..player.max_energy {
                let c = Self::tile_coord((n as i32, 2)) + Vector2::new(left, 0.0);
                let spr = if n < player.energy {
                    energy_icons.1
                } else {
                    energy_icons.0
                };
                sprites.push(dc.place(spr, c))
            }

            for n in (0u32..player.max_health).step_by(2) {
                let c = Self::tile_coord((n as i32 / 2, 1)) + Vector2::new(left, 0.0);
                let spr = if player.health as i32 - 2 >= n as i32 {
                    health_icons.2
                } else if player.health as i32 - 1 == n as i32 {
                    health_icons.1
                } else {
                    health_icons.0
                };
                sprites.push(dc.place(spr, c))
            }
        }

        sprites.append(&mut EquippedAbilities::sprites(world, dc, typeface));

        let stairs_loc = world.query::<(&OnMap, &Stairs)>().iter().next().unwrap().1.0.location;
        let dist = player_loc(world).dist_to(stairs_loc);
        let message = format!("Stairs are {} away", dist.floor());
        sprites.append(&mut typeface.print(dc, Self::tile_coord((19, 2)) + Vector2::new(0.0, 11.0), 0.5, message.as_str()));

        sprites
    }

    /// With room for the frame and other things, the status area is a rectangle 29 x 3 tiles
    /// in area. This takes a point in that space and returns a point suitable for passing to a
    /// drawingcontext
    fn tile_coord(loc: impl Into<Vector2<i32>>) -> Vector2<f32> {
        let Vector2 { x, y } = loc.into();
        Vector2::new(
            x as f32 * 16.0 + 8.0,
            y as f32 * 16.0 + 13.0 * 16.0 + 8.0
        )
    }

    /// The sprites forming the frame and background
    fn frame_sprites() -> Vec<Sprite> {
        let dims = Vector2::new(960.0 / 2.0, 544.0 / 2.0);
        let dc = DrawingContext::new(dims);
        // First throw the outline sprites in there:
        // The board is 960x544, which we divide by two to get 480x272.
        // We use the whole width and the map takes up the top 13x16 = 208 px
        // so our rectangle is (0, 208) to (479, 271), for 30x4 tiles.
        UiFrame::draw_frame(dc, (0.0, 208.0), (30, 4), 0.9)
    }
}

pub fn set_message(world: &mut World, message: &str) {
    if let Some((_, status)) = world.query_mut::<&mut StatusBar>().into_iter().next() {
        status.message = String::from(message)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub struct EquippedAbilities {
    pub slot1: Option<Entity>,
    pub slot2: Option<Entity>,
    pub slot3: Option<Entity>,
}

impl EquippedAbilities {
    fn sprites(world: &World, dc: DrawingContext, typeface: &Typeface) -> Vec<Sprite> {
        let mut sprites = vec![];
        if let Some((_, EquippedAbilities { slot1, slot2, slot3 })) = world.query::<&EquippedAbilities>().iter().next() {
            if let Some(ent) = *slot1 {
                sprites.append(&mut Self::draw_slot(world, dc, typeface, ent, 0))
            }
            if let Some(ent) = *slot2 {
                sprites.append(&mut Self::draw_slot(world, dc, typeface, ent, 1))
            }
        }

        sprites
    }

    fn draw_slot(world: &World, dc: DrawingContext, typeface: &Typeface, ent: Entity, index: i32) -> Vec<Sprite> {
        let mut sprites = vec![];
        if let Some(scroll) = world.query_one::<&Scroll>(ent).unwrap().get() {
            let (name, sprite) = scroll.inventory_attrs();
            sprites.push(dc.place(sprite, StatusBar::tile_coord((18, index))));
            let caption = format!("[{}] {}", index + 1, name);
            sprites.append(&mut typeface.print(dc, StatusBar::tile_coord((19, index)) + Vector2::new(4.0, 11.0), 0.8, caption.as_str()))
        }
        sprites
    }
}