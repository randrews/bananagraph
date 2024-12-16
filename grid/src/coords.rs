use std::fmt::Formatter;
use std::hash::{Hash, Hasher};
use std::ops::{Add, Div, Mul};
use line_drawing::Point;

#[derive(Copy, Clone, PartialEq)]
pub enum Dir { North, South, East, West }

/// A tile coordinate: tiles are 8x8 pixels
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Coord(pub i32, pub i32);
pub fn xy(x: i32, y: i32) -> Coord {
    Coord(x, y)
}

impl Hash for Coord {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.hash(state);
        self.1.hash(state);
    }
}

/// A dimension in pixel terms. This is "pixel" in the sense of whatever
/// unspecified thing you're drawing to, meaning, this might get scaled
/// for a `pixels` scaling factor and a hidpi scaling factor
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct PixelCoord(pub i32, pub i32);
pub fn pxy(x: i32, y: i32) -> PixelCoord { PixelCoord(x, y) }

impl Coord {
    pub fn north(&self) -> Coord { Coord(self.0, self.1 - 1) }
    pub fn south(&self) -> Coord { Coord(self.0, self.1 + 1) }
    pub fn east(&self) -> Coord { Coord(self.0 + 1, self.1) }
    pub fn west(&self) -> Coord { Coord(self.0 - 1, self.1) }

    pub fn northwest(&self) -> Coord { Coord(self.0 - 1, self.1 - 1) }
    pub fn southwest(&self) -> Coord { Coord(self.0 - 1, self.1 + 1) }
    pub fn northeast(&self) -> Coord { Coord(self.0 + 1, self.1 - 1) }
    pub fn southeast(&self) -> Coord { Coord(self.0 + 1, self.1 + 1) }

    pub fn translate(&self, dir: Dir) -> Coord {
        match dir {
            Dir::North => self.north(),
            Dir::South => self.south(),
            Dir::East => self.east(),
            Dir::West => self.west()
        }
    }

    pub fn within(&self, other: Coord) -> bool {
        self.0 >= 0 && self.1 >= 0 &&
            self.0 < other.0 && self.1 < other.1
    }

    pub fn dist_to(&self, other: Coord) -> f32 {
        let (dx, dy) = (self.0 - other.0, self.1 - other.1);
        ((dx * dx) as f32 + (dy * dy) as f32).sqrt()
    }

    pub fn manhattan_dist_to(&self, other: Coord) -> i32 {
        let (dx, dy) = (self.0 - other.0, self.1 - other.1);
        dx.abs() + dy.abs()
    }

    pub fn orthogonal(&self, other: Coord) -> bool {
        other == self.north() || other == self.south() ||
            other == self.east() || other == self.west()
    }

    pub fn diagonal(&self, other: Coord) -> bool {
        other == self.northeast() || other == self.northwest() ||
            other == self.southeast() || other == self.southwest()
    }

    pub fn adjacent(&self, other: Coord) -> bool {
        self.orthogonal(other) || self.diagonal(other)
    }
}

impl std::fmt::Display for Coord {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "(x={}, y={})", self.0, self.1)
    }
}

impl Add for Coord {
    type Output = Coord;

    fn add(self, rhs: Self) -> Self::Output {
        xy(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Mul for Coord {
    type Output = Coord;

    fn mul(self, rhs: Self) -> Self::Output {
        xy(self.0 * rhs.0, self.1 * rhs.1)
    }
}

impl Div for Coord {
    type Output = Coord;

    fn div(self, rhs: Self) -> Self::Output {
        xy(self.0 / rhs.0, self.1 / rhs.1)
    }
}

impl Mul<i32> for Coord {
    type Output = Coord;

    fn mul(self, rhs: i32) -> Self::Output {
        xy(self.0 * rhs, self.1 * rhs)
    }
}

impl Div<i32> for Coord {
    type Output = Coord;

    fn div(self, rhs: i32) -> Self::Output {
        xy(self.0 / rhs, self.1 / rhs)
    }
}

impl Add for PixelCoord {
    type Output = PixelCoord;

    fn add(self, rhs: Self) -> Self::Output {
        pxy(self.0 + rhs.0, self.1 + rhs.1)
    }
}

impl Into<Point<i32>> for Coord {
    fn into(self) -> Point<i32> {
        Point::from((self.0, self.1))
    }
}
impl From<Point<i32>> for Coord {
    fn from(value: Point<i32>) -> Self {
        xy(value.0, value.1)
    }
}

pub struct CoordIterator {
    end: Coord,
    curr: i32
}

impl Iterator for CoordIterator {
    type Item = Coord;

    fn next(&mut self) -> Option<Self::Item> {
        if self.end.0 <= 0 || self.end.1 <= 0 {
            None
        } else if self.curr < self.end.0 * self.end.1 {
            let c = xy(self.curr % self.end.0, self.curr / self.end.0);
            self.curr += 1;
            Some(c)
        } else { None }
    }
}

impl IntoIterator for Coord {
    type Item = Coord;
    type IntoIter = CoordIterator;

    fn into_iter(self) -> Self::IntoIter {
        CoordIterator { end: self, curr: 0 }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_iterator() {
        let mut c = xy(2, 1).into_iter();
        assert_eq!(c.next(), Some(xy(0, 0)));
        assert_eq!(c.next(), Some(xy(1, 0)));
        assert_eq!(c.next(), None);
    }
}
