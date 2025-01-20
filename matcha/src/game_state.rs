use std::time::Duration;
use cgmath::Point2;
use hecs::{Entity, World};
use rand::Rng;
use bananagraph::{DrawingContext, Sprite, SpriteId};
use grid::{xy, Coord, Grid, VecGrid};
use crate::animation::{Pulse};
use crate::drawable::Drawable;
use crate::matcha_board::MatchaBoard;
use crate::piece::{Piece, PieceColor};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ClickTarget {
    SPRITE { id: SpriteId },
    LOCATION { location: Point2<f64> }
}

pub struct GameState<'a, R: Rng> {
    world: World,
    board: VecGrid<Entity>,
    rng: &'a mut R,
    screen: (u32, u32),
    selected: Option<Entity>
}

impl<'a, R: Rng> GameState<'a, R> {
    pub fn new(rng: &'a mut R, screen: (u32, u32)) -> Self {
        let mut world = World::new();
        let board = VecGrid::new(xy(8, 8), Entity::DANGLING);

        let mut state = Self {
            world,
            rng,
            screen,
            board,
            selected: None
        };

        state.initialize_board();
        state
    }

    pub fn initialize_board(&mut self) {
        let mut board = VecGrid::new((8, 8).into(), PieceColor::RED);
        loop {
            // board is a temporary vecgrid of just piece colors, until we can create a valid
            // field, then we'll reify it into entities
            for coord in Grid::size(&board) {
                board[coord] = PieceColor::from_rand(self.rng)
            }

            // Clear out all the matches:
            loop {
                if let Some(coords) = board.find_match() {
                    board.scramble_match(coords, self.rng);
                } else {
                    break
                }
            }

            if board.has_move() { break }
        }

        for (n, color) in board.iter().enumerate() {
            let c = self.board.coord(n);
            let _ = self.world.despawn(self.board[c]);
            self.board[c] = self.world.spawn((Piece::new(*color),))
        }
    }

    pub fn tick(&mut self, dt: Duration) {
        // Go through all the animation types
        for (ent, (anim,)) in self.world.query_mut::<(&mut Pulse,)>() {
            anim.tick(dt);
        }
    }

    pub fn redraw(&self) -> Vec<Sprite> {
        let mut sprites = vec![];
        let dc = DrawingContext::new((self.screen.0 as f32, self.screen.1 as f32));
        for (n, coord) in Grid::size(&self.board).into_iter().enumerate() {
            let mut query = self.world.query_one::<(&Piece,Option<&Pulse>)>(self.board[coord]).unwrap();
            let (piece, pulse) = query.get().unwrap();

            // hecs will give us 0 as a sprite id, but bananagraph can't abide that, so, add something to it to
            // ensure we can hear clicks on the sprite
            let mut drawable = piece.as_drawable(self.board[coord].id() + 1000, coord, self.screen);

            if let Some(pulse) = pulse {
                drawable = pulse.apply_to(drawable)
            }

            sprites.push(drawable.as_sprite(dc))
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

            if let Some(selected) = self.selected {
                let selected_coord = self.board.find(|e| *e == selected).unwrap();
                let new_coord = self.board.find(|e| *e == ent).unwrap();
                println!("Swapping {}, {}", selected_coord, new_coord);
                if self.valid_move(selected_coord, new_coord) || self.valid_move(new_coord, selected_coord) {
                    println!("This would match!");
                }
                self.selected = None;
                self.world.remove_one::<Pulse>(selected).unwrap();
            } else {
                self.selected = Some(ent);
                self.world.insert_one(ent, Pulse::new()).unwrap()
            }
            // let has_anim = {
            //     let mut query = self.world.query_one::<(Option<&Animation>,)>(ent).unwrap();
            //     matches!(query.get(), Some((Some(_),)))
            // };
            //
            // // There's no current animation so tack one on:
            // if !has_anim && self.board.is_match(&self.world, self.board.find(|e| *e == ent).unwrap()).is_some() {
            //     self.world.insert_one(ent, Animation::SPIN { angle: Deg(0.0) }).unwrap();
            // }
        }
    }
}

impl<'a, R: Rng> MatchaBoard for GameState<'a, R> {
    fn get(&self, coord: Coord) -> Option<PieceColor> {
        if let Some(&entity) = self.board.get(coord) {
            let mut query = self.world.query_one::<&Piece>(entity).unwrap();
            let piece = query.get().unwrap();
            Some(piece.color)
        } else {
            None
        }
    }

    fn set(&mut self, coord: Coord, color: PieceColor) {
        if let Some(&entity) = self.board.get(coord) {
            let mut piece = self.world.query_one_mut::<&mut Piece>(entity).unwrap();
            piece.color = color
        }
    }

    fn size(&self) -> Coord {
        self.board.size()
    }
}
