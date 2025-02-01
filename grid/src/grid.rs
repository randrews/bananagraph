use crate::coords::Coord;
use cgmath::Vector2;

/// A trait for operations on a 2d grid of objects
pub trait Grid {
    /// The type of thing this is a grid of
    type CellType;

    /// Return how large the grid is
    fn size(&self) -> Vector2<i32>;

    /// The default value to use for unset cells in the grid (or if we look at
    /// a cell outside the grid)
    fn default(&self) -> Self::CellType;

    /// Get a cell in the grid. This function must return `Some` for any point within
    /// the size of the grid, and `None` for any point outside the grid.
    fn get(&self, index: impl Into<Vector2<i32>>) -> Option<&Self::CellType>;

    /// Is a given point inside the grid?
    fn contains(&self, point: impl Into<Vector2<i32>>) -> bool {
        let point = point.into();
        let dims = self.size();
        !(point.x < 0 || point.y < 0 ||
            point.y >= dims.y ||
            point.x >= dims.x)
    }

    /// Returns the coord representing the nth cell in the grid, in reading order:
    /// left-to-right, top-to-bottom. This is useful because this is also the order that
    /// an `iter()` traverses the grid:
    /// ```
    /// # use grid::*;
    /// let grid = VecGrid::from("ABC\nDEF");
    /// for (n, cell) in grid.iter().enumerate() {
    ///     let pt = grid.coord(n);
    /// }
    /// ```
    fn coord(&self, n: usize) -> Vector2<i32> {
        let (width, _) = self.size().into();
        (n as i32 % width, n as i32 / width).into()
    }

    /// The opposite of `coord`: returns `n` for a given coord in the grid, in reading order:
    /// left-to-right, top-to-bottom. If the govin coord is not in the grid, returns None
    /// ```
    /// # use grid::*;
    /// let grid = VecGrid::new((5, 5).into(), 0);
    /// let n = grid.nth((3, 2));
    /// ```
    fn nth(&self, point: impl Into<Vector2<i32>>) -> Option<usize> {
        let point = point.into();
        if self.contains(point) {
            Some((point.x + point.y * self.size().x) as usize)
        } else {
            None
        }
    }

    fn iter(&self) -> impl Iterator<Item=&Self::CellType> {
        let num_cells = (self.size().x * self.size().y) as usize;
        (0..num_cells).map(|n| self.get(self.coord(n)).unwrap())
    }

    fn map<A, F: Fn(Vector2<i32>, &Self::CellType) -> A>(&self, func: F) -> Vec<A> {
        let mut grid = Vec::with_capacity((self.size().x * self.size().y) as usize);
        for pt in self.size().iter() {
            grid.push(func(pt, self.get(pt).unwrap()))
        }
        grid
    }

    /// Runs a given lambda on all orthogonally-adjacent cells, running it on the default
    /// for any cells not in the grid
    /// ```
    /// # use grid::*;
    /// let grid = VecGrid::from("+A\nAB");
    /// // Downcase all the neighbors:
    /// let cs = grid.for_neighbors((0, 0), |_c, ch| ch.to_lowercase().next().unwrap());
    /// ```
    fn for_neighbors<T, F: Fn(Vector2<i32>, &Self::CellType) -> T>(&self, point: impl Into<Vector2<i32>>, func: F) -> (T, T, T, T) {
        let point = point.into();
        let def = self.default();
        let (x, y) = point.into();
        let n = func((x, y-1).into(), self.get((x, y-1)).unwrap_or(&def));
        let s = func((x, y+1).into(), self.get((x, y+1)).unwrap_or(&def));
        let e = func((x+1, y).into(), self.get((x+1, y)).unwrap_or(&def));
        let w = func((x-1, y).into(), self.get((x-1, y)).unwrap_or(&def));
        (n, s, e, w)
    }

    /// Just like `for_neighbors` except it returns `(ne, se, sw, nw)`
    fn for_diagonals<T, F: Fn(Vector2<i32>, &Self::CellType) -> T>(&self, point: impl Into<Vector2<i32>>, func: F) -> (T, T, T, T) {
        let point = point.into();
        let def = self.default();
        let (x, y) = point.into();
        let ne = func((x+1, y-1).into(), self.get((x+1, y-1)).unwrap_or(&def));
        let se = func((x+1, y+1).into(), self.get((x+1, y+1)).unwrap_or(&def));
        let sw = func((x-1, y+1).into(), self.get((x-1, y+1)).unwrap_or(&def));
        let nw = func((x-1, y-1).into(), self.get((x-1, y-1)).unwrap_or(&def));
        (ne, se, sw, nw)
    }

    /// The coordinates of our orthogonal neighbors, but only the ones actually in the grid
    fn neighbor_coords(&self, point: impl Into<Vector2<i32>>) -> impl Iterator<Item=Vector2<i32>> {
        let point = point.into();
        let c = vec![point.north(), point.east(), point.south(), point.west()];
        c.into_iter().filter(|pt| self.contains(*pt))
    }

    /// The coordinates of our diagonal neighbors, but only the ones actually in the grid
    fn diagonal_coords(&self, point: impl Into<Vector2<i32>>) -> impl Iterator<Item=Vector2<i32>> {
        let point = point.into();
        let c = vec![point.northeast(), point.southeast(), point.southwest(), point.northwest()];
        c.into_iter().filter(|pt| self.contains(*pt))
    }

    /// The coordinates of our orthogonal and diagonal neighbors, but only the ones actually in the grid
    fn adjacent_coords(&self, point: impl Into<Vector2<i32>>) -> impl Iterator<Item=Vector2<i32>> {
        let point = point.into();
        let c = vec![
            point.north(),
            point.northeast(),
            point.east(),
            point.southeast(),
            point.south(),
            point.southwest(),
            point.west(),
            point.northwest()
        ];
        c.into_iter().filter(|pt| self.contains(*pt))
    }

    /// Convenience method for `for_neighbors` just comparing with ==
    fn neighbors_equal(&self, point: impl Into<Vector2<i32>>, val: Self::CellType) -> (bool, bool, bool, bool)
        where Self::CellType: PartialEq {
        self.for_neighbors(point, |_, cell| *cell == val)
    }

    /// Convenience method for `for_diagonals` just comparing with ==
    fn diagonals_equal(&self, point: impl Into<Vector2<i32>>, val: Self::CellType) -> (bool, bool, bool, bool)
        where Self::CellType: PartialEq {
        self.for_diagonals(point, |_, cell| *cell == val)
    }

    /// Returns a coord (arbitrary, but in practice the top-left) of a cell that fits the
    /// given filter
    fn find<F: Fn(&Self::CellType) -> bool>(&self, test: F) -> Option<Vector2<i32>> {
        for c in self.size().iter() {
            if test(self.get(c).unwrap()) { return Some(c) }
        }
        None
    }

    /// Return an iterator of all the coords that match a certain predicate
    fn find_all<'a, F: Fn(&Self::CellType) -> bool + 'a>(&'a self, test: F) -> impl Iterator<Item=Vector2<i32>> {
        self.size().iter().filter(move |c| test(self.get(*c).unwrap()))
    }
}

/// A trait that can be applied to any `Grid` to represent mutating cells in the grid.
pub trait GridMut: Grid {
    /// This behaves just like `get`: it must return `Some` for any coord in the bounds of the
    /// grid and `None` outside.
    fn get_mut(&mut self, index: impl Into<Vector2<i32>>) -> Option<&mut Self::CellType>;
}

/// Trait impld on `(bool, bool, bool, bool)` to make it easy to count
/// how many neighbors fit some criteria (since neighbors and diagonals fns
/// in Grid return that tuple)
pub trait CountableNeighbors {
    fn count(&self) -> i32;
}

impl CountableNeighbors for (bool, bool, bool, bool) {
    fn count(&self) -> i32 {
        let mut t = 0;
        let (n, s, e, w) = self;
        if *n { t += 1 }
        if *s { t += 1 }
        if *e { t += 1 }
        if *w { t += 1 }
        t
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestGrid(Vec<char>, i32);
    impl Grid for TestGrid {
        type CellType = char;
        fn size(&self) -> Vector2<i32> { (self.1, self.0.len() as i32 / self.1).into() }
        fn default(&self) -> Self::CellType { ' ' }

        fn get(&self, index: impl Into<Vector2<i32>>) -> Option<&char> {
            let index = index.into();
            if self.contains(index) {
                Some(&self.0[index.x as usize + (index.y * self.1) as usize])
            } else {
                None
            }
        }
    }

    impl From<&str> for TestGrid {
        fn from(value: &str) -> Self {
            let lines: Vec<Vec<_>> = value.lines().map(|line| line.chars().collect()).collect();
            let width = lines[0].len() as i32;
            Self(lines.concat(), width)
        }
    }

    #[test]
    fn test_iter() {
        let grid = TestGrid::from("AB\nCD");
        let mut it = grid.iter();
        assert_eq!(it.next(), Some(&'A'));
        assert_eq!(it.next(), Some(&'B'));
        assert_eq!(it.next(), Some(&'C'));
        assert_eq!(it.next(), Some(&'D'));
        assert_eq!(it.next(), None);
    }

    #[test]
    fn test_neighbors() {
        // ABA
        // BBA
        // AAA
        let grid = TestGrid::from("ABA\nBBA\nAAA");
        // The one in the center:
        assert_eq!(grid.neighbors_equal((1, 1), 'B'), (true, false, false, true));

        // One near the edge:
        assert_eq!(grid.neighbors_equal((1, 0), 'B'), (false, true, false, false))
    }
}