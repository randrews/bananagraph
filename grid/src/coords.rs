use cgmath::Vector2;

#[derive(Copy, Clone, PartialEq)]
pub enum Dir { North, South, East, West }

pub trait Coord: Copy + PartialEq<Self> + Into<(i32, i32)> + From<(i32, i32)> {
    fn north(&self) -> Self { let (x, y) = (*self).into(); Self::from((x, y - 1)) }
    fn south(&self) -> Self { let (x, y) = (*self).into(); Self::from((x, y + 1)) }
    fn east(&self) -> Self { let (x, y) = (*self).into(); Self::from((x + 1, y)) }
    fn west(&self) -> Self { let (x, y) = (*self).into(); Self::from((x - 1, y)) }

    fn northwest(&self) -> Self { let (x, y) = (*self).into(); Self::from((x - 1, y - 1)) }
    fn southwest(&self) -> Self { let (x, y) = (*self).into(); Self::from((x - 1, y + 1)) }
    fn northeast(&self) -> Self { let (x, y) = (*self).into(); Self::from((x + 1, y - 1)) }
    fn southeast(&self) -> Self { let (x, y) = (*self).into(); Self::from((x + 1, y + 1)) }

    fn translate(&self, dir: Dir) -> Self {
        match dir {
            Dir::North => self.north(),
            Dir::South => self.south(),
            Dir::East => self.east(),
            Dir::West => self.west()
        }
    }

    fn within(&self, other: impl Coord) -> bool {
        let (x, y) = (*self).into();
        let (ox, oy) = other.into();
        x >= 0 && y >= 0 && x < ox && y < oy
    }

    fn dist_to(&self, other: impl Coord) -> f32 {
        let (x, y) = (*self).into();
        let (ox, oy) = other.into();
        let (dx, dy) = (x - ox, y - oy);
        ((dx * dx) as f32 + (dy * dy) as f32).sqrt()
    }

    fn manhattan_dist_to(&self, other: impl Coord) -> i32 {
        let (x, y) = (*self).into();
        let (ox, oy) = other.into();
        let (dx, dy) = (x - ox, y - oy);
        dx.abs() + dy.abs()
    }

    fn orthogonal(&self, other: Self) -> bool {
        other == self.north() || other == self.south() ||
            other == self.east() || other == self.west()
    }

    fn diagonal(&self, other: Self) -> bool {
        other == self.northeast() || other == self.northwest() ||
            other == self.southeast() || other == self.southwest()
    }

    fn adjacent(&self, other: Self) -> bool {
        self.orthogonal(other) || self.diagonal(other)
    }

    fn iter(self) -> CoordIterator {
        CoordIterator { end: self.into(), curr: 0 }
    }

    fn index(self, width: impl Into<usize>) -> usize {
        let width = width.into();
        let (x, y) = self.into();
        y as usize * width + x as usize
    }
}

// /// A dimension in pixel terms. This is "pixel" in the sense of whatever
// /// unspecified thing you're drawing to, meaning, this might get scaled
// /// for a `pixels` scaling factor and a hidpi scaling factor
// #[derive(Copy, Clone, Debug, PartialEq)]
// pub struct PixelCoord(pub i32, pub i32);
// pub fn pxy(x: i32, y: i32) -> PixelCoord { PixelCoord(x, y) }

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
