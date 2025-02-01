use std::ops::{Index, IndexMut};
use crate::{Coord, Grid};
use crate::grid::GridMut;
use cgmath::Vector2;

/// An implementation of Grid backed by a Vec
#[derive(Clone)]
pub struct VecGrid<T> {
    cells: Vec<T>,
    width: usize,
    default: T
}

impl<T: Clone> Grid for VecGrid<T> {
    type CellType = T;

    fn size(&self) -> Vector2<i32> {
        (self.width as i32, (self.cells.len() / self.width) as i32).into()
    }

    fn default(&self) -> T {
        self.default.clone()
    }

    fn get(&self, index: impl Into<Vector2<i32>>) -> Option<&T> {
        let index = index.into();
        if self.contains(index) {
            Some(&self.cells[index.index(self.width)])
        } else {
            None
        }
    }
}

impl<T: Clone> GridMut for VecGrid<T> {
    fn get_mut(&mut self, index: impl Into<Vector2<i32>>) -> Option<&mut T> {
        let index = index.into();
        if self.contains(index) {
            Some(&mut self.cells[index.index(self.width)])
        } else {
            None
        }
    }
}

impl<T: Clone, I: Into<Vector2<i32>>> Index<I> for VecGrid<T> {
    type Output = T;
    fn index(&self, index: I) -> &Self::Output { self.get(index).unwrap() }
}

impl<T: Clone, I: Into<Vector2<i32>>> IndexMut<I> for VecGrid<T> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output { self.get_mut(index).unwrap() }
}

impl<T: Clone + Copy> VecGrid<T> {
    pub fn new(size: impl Into<Vector2<i32>>, default: T) -> VecGrid<T> {
        let size = size.into();
        let (width, height) = (size.x as usize, size.y as usize);
        let cells = vec![default; width * height];
        Self { cells, width, default }
    }
}

impl<T: Clone> VecGrid<T> {
    pub fn from_vec(cells: Vec<T>, width: usize, default: T) -> Self {
        Self { cells, width, default }
    }

    pub fn map_grid<A: Clone, F: Fn(Vector2<i32>, &T) -> A>(&self, func: F, default: A) -> VecGrid<A> {
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
        let mut s = String::with_capacity(((value.size().x + 1) * value.size().y) as usize);
        for pt in value.size().iter() {
            if pt.x == 0 && pt.y > 0 { s.push('\n') }
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
        grid[Vector2::from((0, 0))] = 'z';
        assert_eq!(grid[(0, 0)], 'z');
        assert_eq!(grid[(0, 1)], 'C');
        assert_eq!(grid.get((2, 2)), None);
    }
}