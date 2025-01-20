use lazy_static::lazy_static;
use grid::{xy, Coord, Grid, VecGrid};
use rand::Rng;
use crate::game_state::GameState;
use crate::piece::{Piece, PieceColor};

lazy_static! {
    static ref VALID_MOVES: Vec<(Coord, Coord, Coord)> = all_valid_moves();
}

pub trait MatchaBoard {
    /// Get the color of a single cell on the board
    fn get(&self, coord: Coord) -> Option<PieceColor>;
    fn set(&mut self, coord: Coord, color: PieceColor);
    fn size(&self) -> Coord;

    /// If the given cell is topmost or leftmost of a match, return the cells that match it
    fn is_match(&self, coord: Coord) -> Option<Vec<Coord>> {
        if let Some(color) = self.get(coord) {
            let (right1, right2, right3, right4,
                down1, down2, down3, down4) = (
                self.get(coord.east()),
                self.get(coord.east().east()),
                self.get(coord.east().east().east()),
                self.get(coord.east().east().east().east()),
                self.get(coord.south()),
                self.get(coord.south().south()),
                self.get(coord.south().south().south()),
                self.get(coord.south().south().south().south()),
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
    fn is_move(&self, coord: Coord) -> bool {
        if let Some(color) = self.get(coord) {
            for (a, b, _) in VALID_MOVES.iter() {
                if let (Some(a_col), Some(b_col)) = (self.get(coord + *a), self.get(coord + *b)) {
                    if color == a_col && color == b_col {
                        return true
                    }
                }
            }
        }
        false
    }

    /// Return whether there are any valid moves on the board
    fn has_move(&self) -> bool {
        for coord in self.size().into_iter() {
            if self.is_move(coord) { return true }
        }
        false
    }

    /// Whether the two pieces, if swapped, would cause the piece in coord1 to be a valid match.
    /// To really ensure this is a valid move, call it twice with coord1 and coord2 swapped!
    fn valid_move(&self, coord1: Coord, coord2: Coord) -> bool {
        if let Some(a_color) = self.get(coord1) {
            for (x1, x2, b) in VALID_MOVES.iter() {
                if let (Some(x1_color), Some(x2_color)) = (self.get(*x1 + coord1), self.get(*x2 + coord1)) {
                    if a_color == x1_color && a_color == x2_color && *b + coord1 == coord2 {
                        return true
                    }
                }
            }
        }

        false
    }

    /// Return the list of cells that are in an arbitrary match in the board
    fn find_match(&self) -> Option<Vec<Coord>> {
        for coord in self.size().into_iter() {
            if let Some(v) = self.is_match(coord) {
                return Some(v)
            }
        }
        None
    }

    /// If the given cell is topmost or leftmost of a match, randomize the colors of the matching cells
    fn scramble_match<R: Rng + ?Sized>(&mut self, coords: Vec<Coord>, rng: &mut R) {
        for cell in coords.into_iter() {
            self.set(cell, PieceColor::from_rand(rng))
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

impl MatchaBoard for VecGrid<PieceColor> {
    fn get(&self, coord: Coord) -> Option<PieceColor> {
        Grid::get(self, coord).map(|p| *p)
    }

    fn set(&mut self, coord: Coord, color: PieceColor) {
        self[coord] = color
    }

    fn size(&self) -> Coord {
        Grid::size(self)
    }
}