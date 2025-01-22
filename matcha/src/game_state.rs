use std::time::Duration;
use cgmath::{Point2, Vector2};
use hecs::{Entity, World};
use rand::Rng;
use bananagraph::{DrawingContext, Sprite, SpriteId};
use grid::{xy, Grid, VecGrid};
use crate::animation::{Animation, MoveAnimation, Pulse};
use crate::matcha_board::MatchaBoard;
use crate::piece::{Piece, PieceColor};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ClickTarget {
    SPRITE { id: SpriteId },
    LOCATION { location: Point2<f64> }
}

pub struct GameState<'a, R: Rng> {
    world: World,
    rng: &'a mut R,
    screen: (u32, u32),
    selected: Option<Entity>,
    swapped: Option<Entity>
}

impl<'a, R: Rng> GameState<'a, R> {
    pub fn new(rng: &'a mut R, screen: (u32, u32)) -> Self {
        let mut world = World::new();

        let board = initialize_board(rng);

        for (n, color) in board.iter().enumerate() {
            let c = board.coord(n);
            world.spawn((Piece::new(*color, (c.0, c.1)),));
        }

        let mut state = Self {
            world,
            rng,
            screen,
            selected: None,
            swapped: None
        };

        state
    }

    pub fn tick(&mut self, dt: Duration) {
        // Go through all the animation types
        Pulse::system(dt, &mut self.world);
        MoveAnimation::system(dt, &mut self.world);

        // Only way this happens is the first tick after the move animations finish, so,
        // it's time to actually swap the pieces and do captures:
        if !self.animation_blocked() {
            if let (Some(a), Some(b)) = (self.swapped, self.selected) {
                let pos_a = self.world.get::<&Piece>(a).unwrap().position;
                let pos_b = self.world.get::<&Piece>(b).unwrap().position;
                (*self.world.get::<&mut Piece>(a).unwrap()).position = pos_b;
                (*self.world.get::<&mut Piece>(b).unwrap()).position = pos_a;
                self.selected = None;
                self.swapped = None;
            }
        }
    }

    pub fn redraw(&self) -> Vec<Sprite> {
        let mut sprites = vec![];
        let dc = DrawingContext::new((self.screen.0 as f32, self.screen.1 as f32));

        let mut query = self.world.query::<(&Piece, Option<&Pulse>, Option<&MoveAnimation>)>();
        for (ent, (piece, pulse, move_anim)) in query.into_iter() {
            // hecs will give us 0 as a sprite id, but bananagraph can't abide that, so, add something to it to
            // ensure we can hear clicks on the sprite
            let mut drawable = piece.as_drawable(ent.id() + 1000, self.screen);

            pulse.map(|p| drawable = p.apply_to(drawable));
            move_anim.map(|m| drawable = m.apply_to(drawable));

            sprites.push(drawable.as_sprite(dc))
        }

        sprites
    }

    pub fn click(&mut self, target: ClickTarget) {
        if let ClickTarget::SPRITE { id} = target {
            // If we're waiting on animations,
            if self.animation_blocked() { return }

            // This is only ever clicked sprite ids, which are always an entity id + 1000: hecs will give 0 as
            // entity ids, which bananagraph interprets as an empty sprite id
            let ent = unsafe {
                self.world.find_entity_from_id(id - 1000)
            };

            if let Some(selected) = self.selected {
                let new_piece = *self.world.get::<&Piece>(ent).unwrap();
                let selected_piece = *self.world.get::<&Piece>(selected).unwrap();

                // Valid move?
                if self.valid_move(selected_piece.position, new_piece.position) || self.valid_move(new_piece.position, selected_piece.position) {
                    // Create and attach animations
                    let (anim1, anim2) = Piece::swap_animations(selected_piece, new_piece);
                    self.world.insert_one(selected, anim1).unwrap();
                    self.world.insert_one(ent, anim2).unwrap();

                    // Record the piece we're swapping with
                    self.swapped = Some(ent);
                } else {
                    // Invalid, clear the selection
                    self.selected = None;
                }
                // Either way stop pulsing
                self.world.remove_one::<Pulse>(selected).unwrap();
            } else {
                self.selected = Some(ent);
                self.world.insert_one(ent, Pulse::new()).unwrap()
            }
        }
    }

    fn board_from_world(&self) -> VecGrid<PieceColor> {
        let mut board = VecGrid::new(xy(8, 8), PieceColor::RED);
        for (_ent, (piece)) in self.world.query::<&Piece>().into_iter() {
            board[xy(piece.position.x, piece.position.y)] = piece.color;
        }
        board
    }

    fn valid_move(&self, p1: impl Into<Vector2<i32>>, p2: impl Into<Vector2<i32>>) -> bool {
        let (p1, p2) = (p1.into(), p2.into());
        let board = self.board_from_world();
        board.valid_move(xy(p1.x, p1.y), xy(p2.x, p2.y))
    }

    pub fn animation_blocked(&self) -> bool {
        self.world.query::<&MoveAnimation>().into_iter().next().is_some()
    }
}

fn initialize_board<R: Rng + Sized>(rng: &mut R) -> VecGrid<PieceColor> {
    let mut board = VecGrid::new((8, 8).into(), PieceColor::RED);
    loop {
        // board is a temporary vecgrid of just piece colors, until we can create a valid
        // field, then we'll reify it into entities
        for coord in Grid::size(&board) {
            board[coord] = PieceColor::from_rand(rng)
        }

        // Clear out all the matches:
        loop {
            if let Some(coords) = board.find_match() {
                board.scramble_match(coords, rng);
            } else {
                break
            }
        }

        if board.has_move() { break }
    }

    board
}
