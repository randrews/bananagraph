use crate::coords::{Coord, xy};

/// A trait for operations on a 2d grid of objects
pub trait Grid {
    /// The type of thing this is a grid of
    type CellType;

    /// Return how large the grid is
    fn size(&self) -> Coord;

    /// The default value to use for unset cells in the grid (or if we look at
    /// a cell outside the grid)
    fn default(&self) -> Self::CellType;

    /// Get a cell in the grid. This function must return `Some` for any point within
    /// the size of the grid, and `None` for any point outside the grid.
    fn get(&self, index: Coord) -> Option<&Self::CellType>;

    /// Is a given point inside the grid?
    fn contains(&self, point: Coord) -> bool {
        let dims = self.size();
        !(point.0 < 0 || point.1 < 0 ||
            point.1 >= dims.1 ||
            point.0 >= dims.0)
    }

    /// Returns the `Coord` representing the nth cell in the grid, in reading order:
    /// left-to-right, top-to-bottom. This is useful because this is also the order that
    /// an `iter()` traverses the grid:
    /// ```
    /// # use grid::*;
    /// let grid = VecGrid::from("ABC\nDEF");
    /// for (n, cell) in grid.iter().enumerate() {
    ///     let pt = grid.coord(n);
    /// }
    /// ```
    fn coord(&self, n: usize) -> Coord {
        let Coord(width, _) = self.size();
        xy(n as i32 % width, n as i32 / width)
    }

    fn iter(&self) -> impl Iterator<Item=&Self::CellType> {
        let num_cells = (self.size().1 * self.size().0) as usize;
        (0..num_cells).map(|n| self.get(self.coord(n)).unwrap())
    }

    fn map<A, F: Fn(Coord, &Self::CellType) -> A>(&self, func: F) -> Vec<A> {
        let mut grid = Vec::with_capacity((self.size().0 * self.size().1) as usize);
        for pt in self.size() {
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
    /// let cs = grid.for_neighbors(xy(0, 0), |_c, ch| ch.to_lowercase().next().unwrap());
    /// ```
    fn for_neighbors<T, F: Fn(Coord, &Self::CellType) -> T>(&self, point: Coord, func: F) -> (T, T, T, T) {
        let def = self.default();
        let Coord(x, y) = point;
        let n = func(xy(x, y-1), self.get(xy(x, y-1)).unwrap_or(&def));
        let s = func(xy(x, y+1), self.get(xy(x, y+1)).unwrap_or(&def));
        let e = func(xy(x+1, y), self.get(xy(x+1, y)).unwrap_or(&def));
        let w = func(xy(x-1, y), self.get(xy(x-1, y)).unwrap_or(&def));
        (n, s, e, w)
    }

    /// Just like `for_neighbors` except it returns `(ne, se, sw, nw)`
    fn for_diagonals<T, F: Fn(Coord, &Self::CellType) -> T>(&self, point: Coord, func: F) -> (T, T, T, T) {
        let def = self.default();
        let Coord(x, y) = point;
        let ne = func(xy(x+1, y-1), self.get(xy(x+1, y-1)).unwrap_or(&def));
        let se = func(xy(x+1, y+1), self.get(xy(x+1, y+1)).unwrap_or(&def));
        let sw = func(xy(x-1, y+1), self.get(xy(x-1, y+1)).unwrap_or(&def));
        let nw = func(xy(x-1, y-1), self.get(xy(x-1, y-1)).unwrap_or(&def));
        (ne, se, sw, nw)
    }

    /// The coordinates of our orthogonal neighbors, but only the ones actually in the grid
    fn neighbor_coords(&self, point: Coord) -> impl Iterator<Item=Coord> {
        let c = vec![point.north(), point.east(), point.south(), point.west()];
        c.into_iter().filter(|pt| self.contains(*pt))
    }

    /// The coordinates of our diagonal neighbors, but only the ones actually in the grid
    fn diagonal_coords(&self, point: Coord) -> impl Iterator<Item=Coord> {
        let c = vec![point.northeast(), point.southeast(), point.southwest(), point.northwest()];
        c.into_iter().filter(|pt| self.contains(*pt))
    }

    /// The coordinates of our orthogonal and diagonal neighbors, but only the ones actually in the grid
    fn adjacent_coords(&self, point: Coord) -> impl Iterator<Item=Coord> {
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
    fn neighbors_equal(&self, point: Coord, val: Self::CellType) -> (bool, bool, bool, bool)
        where Self::CellType: PartialEq {
        self.for_neighbors(point, |_, cell| *cell == val)
    }

    /// Convenience method for `for_diagonals` just comparing with ==
    fn diagonals_equal(&self, point: Coord, val: Self::CellType) -> (bool, bool, bool, bool)
        where Self::CellType: PartialEq {
        self.for_diagonals(point, |_, cell| *cell == val)
    }

    /// Returns a coord (arbitrary, but in practice the top-left) of a cell that fits the
    /// given filter
    fn find<F: Fn(&Self::CellType) -> bool>(&self, test: F) -> Option<Coord> {
        for c in self.size() {
            if test(self.get(c).unwrap()) { return Some(c) }
        }
        None
    }

    /// Return an iterator of all the coords that match a certain predicate
    fn find_all<'a, F: Fn(&Self::CellType) -> bool + 'a>(&'a self, test: F) -> impl Iterator<Item=Coord> {
        self.size().into_iter().filter(move |c| test(self.get(*c).unwrap()))
    }
}

/// A trait that can be applied to any `Grid` to represent mutating cells in the grid.
pub trait GridMut: Grid {
    /// This behaves just like `get`: it must return `Some` for any coord in the bounds of the
    /// grid and `None` outside.
    fn get_mut(&mut self, index: Coord) -> Option<&mut Self::CellType>;
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
        fn size(&self) -> Coord { xy(self.1, self.0.len() as i32 / self.1) }
        fn default(&self) -> Self::CellType { ' ' }

        fn get(&self, index: Coord) -> Option<&char> {
            if self.contains(index) {
                Some(&self.0[index.0 as usize + (index.1 * self.1) as usize])
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
        assert_eq!(grid.neighbors_equal(xy(1, 1), 'B'), (true, false, false, true));

        // One near the edge:
        assert_eq!(grid.neighbors_equal(xy(1, 0), 'B'), (false, true, false, false))
    }
}