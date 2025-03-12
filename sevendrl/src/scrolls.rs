use cgmath::Vector2;
use hecs::World;
use tinyrand::Rand;
use grid::{Coord, Dir, Grid, VecGrid};
use crate::animation::OneShotAnimation;
use crate::components::{player_loc, OnMap, Player};
use crate::enemy::{enemies_map, Enemy, PFCellType};
use crate::game_state::{GameMode, GameState};
use crate::inventory::Scroll;
use crate::inventory::ScrollType::{Leap, PhaseWalk, Shove, TimeFreeze};
use crate::modal::{ContentType, DismissType, Modal};
use crate::sprites::{AnimationSprites, SpriteFor};
use crate::status_bar::set_message;

pub fn shove_scroll(game_state: &mut GameState) {
    let world = &mut game_state.world;
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

/// A leap scroll teleports you to a random free space within your vision
pub fn leap_scroll(game_state: &mut GameState) {
    let world = &mut game_state.world;
    let rand = &mut game_state.rand;
    let cost = Scroll(Leap).cost();
    if get_player(world).energy < cost {
        set_message(world, format!("Need {} energy to leap", cost).as_str());
        return
    }

    let target_cell = { // Pick a random clear one
        // So first we need a list of spaces in your vision:
        let mut visible = visible_cells(world);
        let enemies = enemies_map(world);
        let mut c;
        loop {
            if visible.is_empty() {
                set_message(world, "Nowhere to leap to!");
                return
            }
            let i = rand.next_usize() % visible.len();
            c = Some(visible.remove(i));
            if enemies[c.unwrap()] == PFCellType::Clear { break }
        }
        c.unwrap()
    };

    // Pull the player loc, we're going to drop an animation there
    let old = player_loc(world);
    // Update the player's loc
    get_player_onmap_mut(world).location = target_cell;
    // Place an animation
    AnimationSprites::shove_at(world, old);
    // Charge them for it
    get_player_mut(world).energy -= cost;
    set_message(world, "You leap to safety!")
}

pub fn phasewalk_scroll(game_state: &mut GameState) {
    // This is the first step, where we ask the player which direction. Just set the gs' mode:
    game_state.mode = GameMode::PhaseWalk;
    set_message(&mut game_state.world, "Which direction? [esc to cancel]")
}

pub fn actually_phasewalk(game_state: &mut GameState, dir: Dir) {
    let world = &mut game_state.world;

    let cost = Scroll(PhaseWalk).cost();
    if get_player(world).energy < cost {
        set_message(world, format!("Need {} energy to phasewalk", cost).as_str());
        return
    }
    get_player_mut(world).energy -= cost;

    let enemies = enemies_map(world);
    let player_loc = player_loc(world);
    let mut dead = vec![];
    let mut dead_locs = vec![];
    let mut curr = player_loc;

    loop {
        curr = curr.translate(dir);
        match enemies.get(curr) {
            None | Some(PFCellType::Wall) => {
                // off map / wall, dead
                game_state.mode = GameMode::GameOver;
                create_phase_modal(world);
                break
            }
            Some(PFCellType::Enemy(ent, ..)) => {
                // Add killed enemy to dead list
                dead.push(*ent);
                dead_locs.push(curr);
            }
            _ => {
                // Done, player is here now
                break
            }
        }
    }

    get_player_onmap_mut(world).location = curr;
    if dead.len() == 1 {
        set_message(world, "Phase walked through 1 enemy!");
    } else {
        set_message(world, format!("Phase walked through {} enemies!", dead.len()).as_str());
    }

    for (n, ent) in dead.into_iter().enumerate() {
        AnimationSprites::enemy_fade_at(world, ent, dead_locs[n], false);
        world.despawn(ent).unwrap();
    }
}

pub fn create_phase_modal(world: &mut World) {
    world.spawn((Modal::new((15, 6), vec![
        ContentType::Center(String::from("You have died")),
        ContentType::Text(String::from("Your phase walked into a solid object! You")),
        ContentType::Text(String::from("rematerialize in the wall, dying instantly,")),
        ContentType::Text(String::from("and are part of the dungeon forever.")),
        ContentType::Center(String::from("-= press any key to restart =-")),
    ], DismissType::Any),));
}

#[derive(Copy, Clone, Debug)]
pub struct TimeFreezeEffect(i32);

impl TimeFreezeEffect {
    pub fn time_freeze_remaining(world: &World) -> Option<i32> {
        world.query::<&TimeFreezeEffect>().iter().next().map(|(_, &tf)| tf.0)
    }

    pub fn system(world: &mut World) {
        let q = world.query_mut::<&mut TimeFreezeEffect>().into_iter().next();
        let mut finished_eff = None;
        if let Some((e, effect)) = q {
            effect.0 -= 1;
            if effect.0 == 0 {
                finished_eff = Some(e)
            }
        }

        if let Some(e) = finished_eff {
            world.despawn(e).unwrap()
        }
    }
}

pub fn time_freeze(game_state: &mut GameState) {
    let world = &mut game_state.world;

    let cost = Scroll(TimeFreeze).cost();
    if get_player(world).energy < cost {
        set_message(world, format!("Need {} energy to freeze time", cost).as_str());
        return
    }

    if TimeFreezeEffect::time_freeze_remaining(world).is_some() {
        set_message(world, "Time is already frozen!");
        return
    }

    get_player_mut(world).energy -= cost;

    // The time freeze system will fire _this_ round also, so to get the full 10 turns...
    world.spawn((TimeFreezeEffect(11),));
}

fn visible_cells(world: &World) -> Vec<Vector2<i32>> {
    let fov_map = crate::components::map_data_for(world, (64, 64), player_loc(world)); // TODO don't hardcode map size
    let mut cells = vec![];
    for y in 0..fov_map.height {
        for x in 0..fov_map.width {
            if fov_map.fov[x + y * 64usize] {
                cells.push(Vector2::new(x as i32, y as i32))
            }
        }
    }
    cells
}

fn get_player(world: &World) -> Player {
    *world.query::<&Player>().iter().next().unwrap().1
}

fn get_player_mut(world: &mut World) -> &mut Player {
    world.query_mut::<&mut Player>().into_iter().next().unwrap().1
}

fn get_player_onmap_mut(world: &mut World) -> &mut OnMap {
    world.query_mut::<(&mut Player, &mut OnMap)>().into_iter().next().unwrap().1.1
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
                AnimationSprites::enemy_fade_at(world, ent, curr, true);
                get_player_mut(world).give_energy(1);
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
            } else {
                // If we are, drop him here and grab this current one:
                let tmp = map[curr];
                map[curr] = moved_enemy.unwrap();
                moved_enemy = Some(tmp)
            }

        }

        // Move along in the direction:
        curr += dir
    }

    moved_enemy.is_some()
}