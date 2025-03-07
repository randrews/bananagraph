use cgmath::Vector2;
use hecs::World;
use grid::{Coord, Grid, VecGrid};
use crate::animation::OneShotAnimation;
use crate::components::{player_loc, OnMap, Player};
use crate::enemy::{enemies_map, Enemy, PFCellType};
use crate::inventory::Scroll;
use crate::inventory::ScrollType::Shove;
use crate::sprites::{AnimationSprites, SpriteFor};
use crate::status_bar::set_message;

pub fn shove_scroll(world: &mut World) {
    let player = player_loc(world);
    let mut enemies = enemies_map(world);

    // Check that the player can afford this:
    let cost = Scroll(Shove).cost();
    if get_player(world).energy < cost {
        set_message(world, format!("Need {} energy to shove", cost).as_str());
        return
    }

    let mut any_moved = false;
    any_moved |= shove_in_direction(player, (1, 0), &mut enemies, world);
    any_moved |= shove_in_direction(player, (-1, 0), &mut enemies, world);
    any_moved |= shove_in_direction(player, (0, 1), &mut enemies, world);
    any_moved |= shove_in_direction(player, (0, -1), &mut enemies, world);
    any_moved |= shove_in_direction(player, (1, -1), &mut enemies, world);
    any_moved |= shove_in_direction(player, (1, 1), &mut enemies, world);
    any_moved |= shove_in_direction(player, (-1, -1), &mut enemies, world);
    any_moved |= shove_in_direction(player, (-1, 1), &mut enemies, world);

    // Wow we shoved and there was nothing to shove!
    if !any_moved {
        set_message(world, "No adjacent monsters to shove!")
    } else {
        // Now we need to spawn a cool animation!
        for c in enemies.adjacent_coords(player) {
            if enemies[c] == PFCellType::Clear {
                world.spawn((
                    OnMap { location: c, sprite: AnimationSprites::Shove1.sprite() },
                    OneShotAnimation::new(AnimationSprites::shove())
                ));
            }
        }
        // We shoved things but only on our local map, we need to update the world's OnMaps
        update_world(enemies, world);
        // Charge the player their nickel:
        get_player_mut(world).energy -= cost
    }
}

fn get_player(world: &World) -> Player {
    *world.query::<&Player>().iter().next().unwrap().1
}

fn get_player_mut(world: &mut World) -> &mut Player {
    world.query_mut::<&mut Player>().into_iter().next().unwrap().1
}

fn update_world(map: VecGrid<PFCellType>, world: &mut World) {
    for c in map.size().iter() {
        if let PFCellType::Enemy(ent, _) = map[c] {
            let (_, onmap) = world.query_one_mut::<(&Enemy, &mut OnMap)>(ent).unwrap();
            onmap.location = c;
        }
    }
}

fn shove_in_direction(player: Vector2<i32>, dir: impl Into<Vector2<i32>>, map: &mut VecGrid<PFCellType>, world: &mut World) -> bool {
    let dir = dir.into();
    let mut curr = player + dir;
    let mut moved_enemy: Option<PFCellType> = None;

    loop {
        if !map.contains(curr) || map[curr] == PFCellType::Wall {
            // If we had a moved enemy, he's squished!
            if let Some(PFCellType::Enemy(ent, _)) = moved_enemy {
                // TODO cool animation here?
                world.despawn(ent).unwrap()
            }
            break
        }

        if map[curr] == PFCellType::Clear {
            // If we have an enemy midair, drop him here:
            if let Some(c) = moved_enemy {
                map[curr] = c
            }
            break
        }

        if let PFCellType::Enemy(ent, awake) = map[curr] {
            // An enemy is already in this cell
            // If we aren't carrying an enemy yet, we'll grab him and clear the cell
            if moved_enemy.is_none() {
                moved_enemy = Some(PFCellType::Enemy(ent, awake));
                map[curr] = PFCellType::Clear
            }
            // If we are, just move on.
        }

        // Move along in the direction:
        curr += dir
    }

    moved_enemy.is_some()
}