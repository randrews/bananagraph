use std::time::Duration;
use cgmath::{Deg, Point2};
use hecs::{Entity, World};
use lazy_static::lazy_static;
use rand::Rng;
use bananagraph::{DrawingContext, Sprite, SpriteId};
use grid::{xy, Coord, Grid, VecGrid};
use crate::animation::Animation;
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

lazy_static! {
    static ref VALID_MOVES: Vec<(Coord, Coord, Coord)> = all_valid_moves();
}

impl<'a, R: Rng> GameState<'a, R> {
    pub fn new(rng: &'a mut R, screen: (u32, u32)) -> Self {
        let mut world = World::new();
        let mut board = VecGrid::new((8, 8).into(), Entity::DANGLING);

        loop {
            for coord in Grid::size(&board) {
                board[coord] = world.spawn((Piece::new_from_rand(rng),));
            }

            // Clear out all the matches:
            loop {
                if let Some(coords) = board.find_match(&world) {
                    board.scramble_match(&mut world, coords, rng);
                } else {
                    break
                }
            }

            if board.has_move(&world) { break }
        }

        Self {
            world,
            board,
            rng,
            screen,
            selected: None
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
        for (n, coord) in Grid::size(&self.board).into_iter().enumerate() {
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

                Some(Animation::PULSE { scale, .. }) => {
                    dc.place_scaled(sprite, (
                        coord.0 as f32 * 85.0 + margin.0,
                        coord.1 as f32 * 85.0 + margin.1
                    ), (*scale, 2.0 - *scale))
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

            if let Some(selected) = self.selected {
                let selected_coord = self.board.find(|e| *e == selected).unwrap();
                let new_coord = self.board.find(|e| *e == ent).unwrap();
                println!("Swapping {}, {}", selected_coord, new_coord);
                if self.board.valid_move(&self.world, selected_coord, new_coord) || self.board.valid_move(&self.world, new_coord, selected_coord) {
                    println!("This would match!");
                }
                self.selected = None;
                self.world.remove_one::<Animation>(selected).unwrap();
            } else {
                self.selected = Some(ent);
                self.world.insert_one(ent, Animation::PULSE { scale: 1.0, delta: 1.0, run: true }).unwrap()
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

trait MatchaBoard {
    /// Get the color of a single cell on the board
    fn get(&self, world: &World, coord: Coord) -> Option<PieceColor>;
    fn set(&mut self, world: &mut World, coord: Coord, color: PieceColor);
    fn size(&self) -> Coord;

    /// If the given cell is topmost or leftmost of a match, return the cells that match it
    fn is_match(&self, world: &World, coord: Coord) -> Option<Vec<Coord>> {
        if let Some(color) = self.get(world, coord) {
            let (right1, right2, right3, right4,
                down1, down2, down3, down4) = (
                self.get(world, coord.east()),
                self.get(world, coord.east().east()),
                self.get(world, coord.east().east().east()),
                self.get(world, coord.east().east().east().east()),
                self.get(world, coord.south()),
                self.get(world, coord.south().south()),
                self.get(world, coord.south().south().south()),
                self.get(world, coord.south().south().south().south()),
                );

            let is_horiz_match = if let (Some(c2), Some(c3)) = (right1, right2) {
                color == c2 && color == c3
            } else {
                false
            };

            let is_vert_match = if let (Some(c2), Some(c3)) = (down1, down2) {
                color == c2 && color == c3
            } else {
                false
            };

            if is_horiz_match {
                let mut m = vec![coord, coord.east(), coord.east().east()];
                if right3.is_some_and(|c| c == color) { m.push(coord.east().east().east()) }
                if right4.is_some_and(|c| c == color) { m.push(coord.east().east().east().east()) }
                Some(m)
            } else if is_vert_match {
                let mut m = vec![coord, coord.south(), coord.south().south()];
                if down3.is_some_and(|c| c == color) { m.push(coord.south().south().south()) }
                if down4.is_some_and(|c| c == color) { m.push(coord.south().south().south().south()) }
                Some(m)
            } else {
                None
            }
        } else {
            None
        }
    }

    /// Return whether the given coord is in a legal move: could another cell be moved to match with
    /// this one
    fn is_move(&self, world: &World, coord: Coord) -> bool {
        if let Some(color) = self.get(world, coord) {
            for (a, b, _) in VALID_MOVES.iter() {
                if let (Some(a_col), Some(b_col)) = (self.get(world, coord + *a), self.get(world, coord + *b)) {
                    if color == a_col && color == b_col {
                        return true
                    }
                }
            }
        }
        false
    }

    /// Return whether there are any valid moves on the board
    fn has_move(&self, world: &World) -> bool {
        for coord in self.size().into_iter() {
            if self.is_move(world, coord) { return true }
        }
        false
    }

    /// Whether the two pieces, if swapped, would cause the piece in coord1 to be a valid match.
    /// To really ensure this is a valid move, call it twice with coord1 and coord2 swapped!
    fn valid_move(&self, world: &World, coord1: Coord, coord2: Coord) -> bool {
        if let Some(a_color) = self.get(world, coord1) {
            for (x1, x2, b) in VALID_MOVES.iter() {
                if let (Some(x1_color), Some(x2_color)) = (self.get(world, *x1 + coord1), self.get(world, *x2 + coord1)) {
                    if a_color == x1_color && a_color == x2_color && *b + coord1 == coord2 {
                        return true
                    }
                }
            }
        }

        false
    }

    /// Return the list of cells that are in an arbitrary match in the board
    fn find_match(&self, world: &World) -> Option<Vec<Coord>> {
        for coord in self.size().into_iter() {
            if let Some(v) = self.is_match(world, coord) {
                return Some(v)
            }
        }
        None
    }

    /// If the given cell is topmost or leftmost of a match, randomize the colors of the matching cells
    fn scramble_match<'a, R: Rng + ?Sized>(&mut self, world: &mut World, coords: Vec<Coord>, rng: &'a mut R) {
        for cell in coords.into_iter() {
            self.set(world, cell, PieceColor::from_rand(rng))
        }
    }
}

/// There are only a few patterns we care about:
///
/// ```text
/// _ _ A
/// X X B
///
/// _ A _
/// X B X
///
/// A B X X
/// ```
///
/// Also the mirror images of those in x and y as well as swapping x and y.
/// We'll represent these as sets of deltas off the piece labeled 'A', and the spot
/// labeled 'B' is where 'A' needs to move to create that match.
fn all_valid_moves() -> Vec<(Coord, Coord, Coord)> {
    // These are tuples of (X1, X2, B) offsets from 'A' above
    let deltas = vec![
        ((-1, 1), (-2, 1), (0, 1)),
        ((-1, 1), (1, 1), (0, 1)),
        ((2, 0), (3, 0), (1, 0)),
    ];

    let mut all_deltas = vec![];
    for (x1, x2, b) in deltas {
        all_deltas.push((x1, x2, b)); // push the primary version:
        all_deltas.push(((-x1.0, x1.1), (-x2.0, x2.1), (-b.0, b.1))); // Reflect the x coords
        all_deltas.push(((x1.0, -x1.1), (x2.0, -x2.1), (b.0, -b.1))); // Reflect the y coords
        all_deltas.push(((-x1.0, -x1.1), (-x2.0, -x2.1), (-b.0, -b.1))); // Reflect both

        // rotation versions:
        all_deltas.push(((x1.1, x1.0), (x2.1, x2.0), (b.1, b.0))); // just rotate
        all_deltas.push(((-x1.1, x1.0), (-x2.1, x2.0), (-b.1, b.0))); // Reflect the x coords
        all_deltas.push(((x1.1, -x1.0), (x2.1, -x2.0), (b.1, -b.0))); // Reflect the y coords
        all_deltas.push(((-x1.1, -x1.0), (-x2.1, -x2.0), (-b.1, -b.0))); // Reflect both
    }

    all_deltas.into_iter().map(|(x1, x2, b)| (xy(x1.0, x1.1), xy(x2.0, x2.1), xy(b.0, b.1))).collect()
}

impl MatchaBoard for VecGrid<Entity> {
    fn get(&self, world: &World, coord: Coord) -> Option<PieceColor> {
        if let Some(&entity) = Grid::get(self, coord) {
            let mut query = world.query_one::<&Piece>(entity).unwrap();
            let piece = query.get().unwrap();
            Some(piece.color)
        } else {
            None
        }
    }

    fn set(&mut self, world: &mut World, coord: Coord, color: PieceColor) {
        if let Some(&entity) = Grid::get(self, coord) {
            let mut piece = world.query_one_mut::<&mut Piece>(entity).unwrap();
             piece.color = color
        }
    }

    fn size(&self) -> Coord {
        Grid::size(self)
    }
}