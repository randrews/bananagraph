use cgmath::Vector2;

/// The four cardinal directions
#[derive(Copy, Clone, PartialEq)]
pub enum Dir { North, South, East, West }

/// A trait to define operations involving cells on a square grid. `Vector2<i32>` and `Point2<i32>`
/// implement it out of the box.
pub trait Coord: Copy + PartialEq<Self> + Into<(i32, i32)> + From<(i32, i32)> {

    /// These four fns, north / south / east / west, return a Coord that's to the north / south / etc
    /// of the given coord. It assumes a square grid with (0, 0) in the top left
    fn north(&self) -> Self { let (x, y) = (*self).into(); Self::from((x, y - 1)) }
    fn south(&self) -> Self { let (x, y) = (*self).into(); Self::from((x, y + 1)) }
    fn east(&self) -> Self { let (x, y) = (*self).into(); Self::from((x + 1, y)) }
    fn west(&self) -> Self { let (x, y) = (*self).into(); Self::from((x - 1, y)) }

    /// Just like north / south / east / west, but the diagonals
    fn northwest(&self) -> Self { let (x, y) = (*self).into(); Self::from((x - 1, y - 1)) }
    fn southwest(&self) -> Self { let (x, y) = (*self).into(); Self::from((x - 1, y + 1)) }
    fn northeast(&self) -> Self { let (x, y) = (*self).into(); Self::from((x + 1, y - 1)) }
    fn southeast(&self) -> Self { let (x, y) = (*self).into(); Self::from((x + 1, y + 1)) }

    /// Translates a Coord in a given Dir
    fn translate(&self, dir: Dir) -> Self {
        match dir {
            Dir::North => self.north(),
            Dir::South => self.south(),
            Dir::East => self.east(),
            Dir::West => self.west()
        }
    }

    /// Returns whether `self` falls within a rectangle that has (0, 0) on the top left and `other`
    /// as the bottom right: in other words, whether 0 <= self.x < other.x, same for y.
    fn within(&self, other: impl Coord) -> bool {
        let (x, y) = (*self).into();
        let (ox, oy) = other.into();
        x >= 0 && y >= 0 && x < ox && y < oy
    }

    /// The straight-line distance between two points
    fn dist_to(&self, other: impl Coord) -> f32 {
        let (x, y) = (*self).into();
        let (ox, oy) = other.into();
        let (dx, dy) = (x - ox, y - oy);
        ((dx * dx) as f32 + (dy * dy) as f32).sqrt()
    }

    /// The "manhattan" distance; adding the differences in x and y together (no diagonal movement)
    fn manhattan_dist_to(&self, other: impl Coord) -> i32 {
        let (x, y) = (*self).into();
        let (ox, oy) = other.into();
        let (dx, dy) = (x - ox, y - oy);
        dx.abs() + dy.abs()
    }

    /// Whether two Coords are orthogonally adjacent: one is north / south / east / west of the other
    fn orthogonal(&self, other: Self) -> bool {
        other == self.north() || other == self.south() ||
            other == self.east() || other == self.west()
    }

    /// Like `orthogonal` but for the diagonal directions
    fn diagonal(&self, other: Self) -> bool {
        other == self.northeast() || other == self.northwest() ||
            other == self.southeast() || other == self.southwest()
    }

    /// Whether two coords are adjacent orthogonally or diagonally
    fn adjacent(&self, other: Self) -> bool {
        self.orthogonal(other) || self.diagonal(other)
    }

    /// Iterates over all (x, y) points in the rectangle `0 <= x < self.x`, `0 <= y < self.y`
    fn iter(self) -> CoordIterator {
        CoordIterator { end: self.into(), curr: 0 }
    }

    /// Turns a `Coord` into an index into a row-major 1d array in reading order (left to right,
    /// top to bottom).
    fn index(self, width: impl Into<usize>) -> usize {
        let width = width.into();
        let (x, y) = self.into();
        y as usize * width + x as usize
    }
}

pub struct CoordIterator {
    end: (i32, i32),
    curr: i32
}

impl Iterator for CoordIterator where {
    type Item = Vector2<i32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end.0 <= 0 || self.end.1 <= 0 {
            None
        } else if self.curr < self.end.0 * self.end.1 {
            let c = (self.curr % self.end.0, self.curr / self.end.0).into();
            self.curr += 1;
            Some(c)
        } else { None }
    }
}

impl Coord for Vector2<i32> {}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_iterator() {
        let pt: Vector2<_> = (2, 1).into();
        let mut c = pt.iter();
        assert_eq!(c.next(), Some((0, 0).into()));
        assert_eq!(c.next(), Some((1, 0).into()));
        assert_eq!(c.next(), None);
    }
}
