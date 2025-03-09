use cgmath::Vector2;
use hecs::{Entity, World};
use log::info;
use grid::{Grid, VecGrid, bfs, UnreachableError, Coord, Dir};
use crate::animation::OneShotAnimation;
use crate::components::{OnMap, Player};
use crate::scrolls::TimeFreezeEffect;
use crate::sprites::AnimationSprites;
use crate::terrain::{Solid};

#[derive(Copy, Clone, Debug, Default)]
pub enum EnemyType {
    #[default]
    Normal,
    Mimic
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Enemy {
    pub awake: bool,
    pub enemy_type: EnemyType,
}

#[derive(Copy, Clone, Debug, Default)]
pub struct Dazed;

impl Dazed {
    pub fn system(world: &mut World) {
        // Anything that was dazed for this round isn't any more:
        let dazed: Vec<_> = world.query::<&Dazed>().iter().map(|(e, _)| e).collect();
        for e in dazed {
            world.remove_one::<Dazed>(e).unwrap();
        }
    }
}

impl Enemy {
    pub fn death_animation(&self) -> OneShotAnimation {
        match self.enemy_type {
            EnemyType::Normal => OneShotAnimation::new(AnimationSprites::enemy_fade()),
            EnemyType::Mimic => OneShotAnimation::new(AnimationSprites::mimic_fade()),
        }
    }

    pub fn system(world: &mut World) {
        if TimeFreezeEffect::time_freeze_remaining(world).is_some() { return } // Nothing happens while time is frozen

        Enemy::attack_system(world);

        OnMap::awaken_enemies(world); // First let's update who can see us
        let mut enemy_map = enemies_map(world);
        let player_loc = player_loc(world);

        for n in 0..(enemy_map.size().x * enemy_map.size().y) {
            let c = enemy_map.coord(n as usize);

            if let PFCellType::Enemy(ent, true) = enemy_map[c] {
                if let Ok(mut path) = best_path(&mut enemy_map, player_loc, c) {
                    // We have a path to the player!
                    // First cell in our path is where we're at, last cell is the player let's drop those.
                    path.remove(0);
                    // If there's any path left (we're not next to the player, in other words):
                    if let Some(nextmove) = path.first() {
                        let nextmove = *nextmove;
                        // We know where we are and where we're going. Take us there:
                        world.query_one_mut::<&mut OnMap>(ent).unwrap().location = nextmove;
                        // But now we also need to update the temporary enemy_map, because we don't want
                        // other mobs to move where we just did, or for where we were to block other mobs:
                        enemy_map[nextmove] = PFCellType::MovedEnemy;
                        enemy_map[c] = PFCellType::Clear;
                    }
                }
            }
        }
    }

    pub fn attack_system(world: &mut World) {
        let player_loc = player_loc(world);
        let count = world.query::<(&OnMap, &Enemy, Option<&Dazed>)>().iter().filter(|&(_, (om, e, dz))| om.location.orthogonal(player_loc) && dz.is_none() && e.awake).count();
        if count > 0 { damage_player(world, count as u32) }
    }

    pub fn try_shove(world: &mut World, location: Vector2<i32>, dir: Dir) -> bool {
        // First find the enemy at that location, if any:
        let enemy_ent = world.query::<(&Enemy, &OnMap)>().iter().find_map(|(e, (_, om))| if om.location == location { Some(e) } else { None });
        if let Some(enemy_ent) = enemy_ent {
            // Is there a solid cell behind it?
            let beyond = location.translate(dir);
            let wall = world.query::<(&OnMap, &Solid)>().iter().any(|(_, (om, _))| om.location == beyond );
            if !wall {
                // There's a place to shove! Move this enemy there:
                world.query_one_mut::<&mut OnMap>(enemy_ent).unwrap().location = beyond;
                // Daze them so they don't move right back:
                world.insert(enemy_ent, (Dazed,)).unwrap();
                // Shove animation:
                AnimationSprites::shove_at(world, location);
                return true
            }
        }
        false
    }
}

// Okay we're gonna do some pathfinding. First we want to know about
// the three kinds of things we care about:
#[derive(Copy, Clone, Debug, PartialEq, Default)]
pub enum PFCellType {
    #[default]
    Clear,
    Wall,
    Enemy(Entity, bool),
    MovedEnemy // An enemy that has already moved this tick
}

// Make a VecGrid of the enemy locations. TODO don't hard-code the map dimensions
pub fn enemies_map(world: &World) -> VecGrid<PFCellType> {
    let mut map = VecGrid::new((64, 64), PFCellType::Clear);

    for (ent, (solid, enemy, dazed, onmap)) in world.query::<(Option<&Solid>, Option<&Enemy>, Option<&Dazed>, &OnMap)>().iter() {
        if enemy.is_some() {
            map[onmap.location] = PFCellType::Enemy(ent, enemy.unwrap().awake && dazed.is_none()) // enemies are all solid so check this first
        } else if solid.is_some() {
            map[onmap.location] = PFCellType::Wall
        }
    }
    map
}

// Find where the enemies are going
fn player_loc(world: &mut World) -> Vector2<i32> {
    let (_, (_, OnMap { location, .. })) = world.query_mut::<(&Player, &OnMap)>().into_iter().next().unwrap();
    *location
}

fn best_path(enemy_map: &VecGrid<PFCellType>, player_loc: Vector2<i32>, enemy_loc: Vector2<i32>) -> Result<Vec<Vector2<i32>>, UnreachableError> {
    // Okay, first of all, if we're already ortho to the player, don't move:
    // (it's not actually unreachable but this will cause us to not walk)
    if player_loc.orthogonal(enemy_loc) { return Err(UnreachableError{}) }

    // Let's see if there's a free one-move for us that's also ortho to the player:
    let empty_next_to_player = |c: &Vector2<i32>| c.orthogonal(player_loc) && enemy_map[*c] == PFCellType::Clear;
    if let Some(tgt) = enemy_map.adjacent_coords(enemy_loc).filter(empty_next_to_player).next() {
        // There is! Move there:
        return Ok(vec![enemy_loc, tgt])
    }

    // Oof, no one-step answers. Better find a longer path:
    let traversable = |cell: &PFCellType| *cell == PFCellType::Clear;
    let mut simple_path = bfs(enemy_map, enemy_loc, player_loc, true, traversable)?;
    simple_path.pop(); // Remove the player loc from the end
    Ok(simple_path)
}

fn damage_player(world: &mut World, damage: u32) {
    let player = world.query_mut::<&mut Player>().into_iter().next().unwrap().1;
    let new_health = (player.health - damage) as i32;
    player.health = 0i32.max(new_health) as u32;
}