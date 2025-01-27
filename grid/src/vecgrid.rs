use std::ops::{Index, IndexMut};
use crate::{Coord, Grid, xy};
use crate::grid::GridMut;

/// An implementation of Grid backed by a Vec
#[derive(Clone)]
pub struct VecGrid<T> {
    cells: Vec<T>,
    width: usize,
    default: T
}

impl<T: Clone> Grid for VecGrid<T> {
    type CellType = T;

    fn size(&self) -> Coord {
        xy(self.width as i32, (self.cells.len() / self.width) as i32)
    }

    fn default(&self) -> T {
        self.default.clone()
    }

    fn get(&self, index: Coord) -> Option<&T> {
        if self.contains(index) {
            Some(&self.cells[index.1 as usize * self.width + index.0 as usize])
        } else {
            None
        }
    }
}

impl<T: Clone> GridMut for VecGrid<T> {
    fn get_mut(&mut self, index: Coord) -> Option<&mut T> {
        if self.contains(index) {
            Some(&mut self.cells[index.1 as usize * self.width + index.0 as usize])
        } else {
            None
        }
    }
}

impl<T: Clone> Index<Coord> for VecGrid<T> {
    type Output = T;
    fn index(&self, index: Coord) -> &Self::Output { self.get(index).unwrap() }
}

impl<T: Clone> IndexMut<Coord> for VecGrid<T> {
    fn index_mut(&mut self, index: Coord) -> &mut Self::Output { self.get_mut(index).unwrap() }
}

impl<T: Clone + Copy> VecGrid<T> {
    pub fn new(size: Coord, default: T) -> VecGrid<T> {
        let (width, height) = (size.0 as usize, size.1 as usize);
        let cells = vec![default; width * height];
        Self { cells, width, default }
    }
}

impl<T: Clone> VecGrid<T> {
    pub fn from_vec(cells: Vec<T>, width: usize, default: T) -> Self {
        Self { cells, width, default }
    }

    pub fn map_grid<A: Clone, F: Fn(Coord, &T) -> A>(&self, func: F, default: A) -> VecGrid<A> {
        let list = self.map(func);
        VecGrid::from_vec(list, self.width, default)
    }
}

impl From<&str> for VecGrid<char> {
    fn from(value: &str) -> Self {
        let lines: Vec<Vec<_>> = value.lines().map(|line| line.chars().collect()).collect();
        let width = lines[0].len();
        let cells = lines.concat();
        Self {
            cells,
            width,
            default: ' '
        }
    }
}

impl From<VecGrid<char>> for String {
    fn from(value: VecGrid<char>) -> Self {
        let mut s = String::with_capacity(((value.size().0 + 1) * value.size().1) as usize);
        for pt in value.size() {
            if pt.0 == 0 && pt.1 > 0 { s.push('\n') }
            s.push(value[pt])
        }
        s
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_vecgrid() {
        let mut grid = VecGrid::from("AB\nCD");
        grid[xy(0, 0)] = 'z';
        assert_eq!(grid[xy(0, 0)], 'z');
        assert_eq!(grid[xy(0, 1)], 'C');
        assert_eq!(grid.get(xy(2, 2)), None);
    }
}