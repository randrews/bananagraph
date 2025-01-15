use std::time::Duration;
use cgmath::{Deg, Point2};
use hecs::{Entity, World};
use rand::Rng;
use bananagraph::{DrawingContext, Sprite, SpriteId};
use grid::{Grid, VecGrid};
use crate::animation::Animation;
use crate::piece::Piece;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ClickTarget {
    SPRITE { id: SpriteId },
    LOCATION { location: Point2<f64> }
}

pub struct GameState<'a, R: Rng> {
    world: World,
    board: VecGrid<Entity>,
    rng: &'a mut R,
    screen: (u32, u32)
}

impl<'a, R: Rng> GameState<'a, R> {
    pub fn new(rng: &'a mut R, screen: (u32, u32)) -> Self {
        let mut world = World::new();
        let mut board = VecGrid::new((8, 8).into(), Entity::DANGLING);

        for coord in board.size() {
            board[coord] = world.spawn((Piece::new_from_rand(rng),));
        }

        Self {
            world,
            board,
            rng,
            screen
        }
    }

    pub fn tick(&mut self, dt: Duration) {
        let mut finished = vec![];
        for (ent, (anim,)) in self.world.query_mut::<(&mut Animation,)>() {
            anim.tick(dt);
            if anim.finished() { finished.push(ent) }
        }

        for ent in finished {
            self.world.remove_one::<Animation>(ent).unwrap();
        }
    }

    pub fn redraw(&self) -> Vec<Sprite> {
        let mut sprites = vec![];
        let dc = DrawingContext::new((self.screen.0 as f32, self.screen.1 as f32));
        let margin = (
            (self.screen.0 as f32 - 8.0 * 85.0) / 2.0,
            (self.screen.1 as f32 - 8.0 * 85.0) / 2.0
            );
        for (n, coord) in self.board.size().into_iter().enumerate() {
            let mut query = self.world.query_one::<(&Piece,Option<&Animation>)>(self.board[coord]).unwrap();
            let (piece,anim) = query.get().unwrap();

            let sprite = piece.as_sprite().with_z(0.5);
            // hecs will give us 0 as a sprite id, but bananagraph can't abide that, so, add something to it to
            // ensure we can hear clicks on the sprite
            let sprite = sprite.with_id(self.board[coord].id() + 1000);

            let sprite = match anim {
                Some(Animation::SPIN { angle }) => {
                    dc.place_rotated(sprite, (
                        coord.0 as f32 * 85.0 + margin.0,
                        coord.1 as f32 * 85.0 + margin.1
                    ), *angle)
                }
                _ => {
                    dc.place(sprite, (
                        coord.0 as f32 * 85.0 + margin.0,
                        coord.1 as f32 * 85.0 + margin.1
                    ))
                }
            };
            sprites.push(sprite)
        }
        sprites
    }

    pub fn click(&mut self, target: ClickTarget) {
        if let ClickTarget::SPRITE { id} = target {
            // This is only ever clicked sprite ids, which are always an entity id + 1000: hecs will give 0 as
            // entity ids, which bananagraph interprets as an empty sprite id
            let ent = unsafe {
                self.world.find_entity_from_id(id - 1000)
            };

            {
                let mut query = self.world.query_one::<(&Piece,Option<&Animation>)>(ent).unwrap();
                if let Some((_,Some(anim))) = query.get() {
                    return
                }
            }

            // There's no current animation so tack one on:
            self.world.insert_one(ent, Animation::SPIN { angle: Deg(0.0) }).unwrap();
        }
    }
}