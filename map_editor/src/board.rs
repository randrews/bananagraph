use bananagraph::Sprite;
use grid::{xy, Coord, Grid, GridMut};
use crate::iso_map::AsSprite;

#[derive(Copy, Clone, Debug)]
pub enum Cell {
    White,
    Black,
    ShortWall,
    TallWall,
    Blank
}

impl AsSprite for Cell {
    fn as_sprite(&self) -> Sprite {
        match self {
            Cell::White => Sprite::new((320, 0), (32, 48)),
            Cell::Black => Sprite::new((352, 0), (32, 48)),
            Cell::ShortWall => Sprite::new((320, 48), (32, 48)),
            Cell::TallWall => Sprite::new((0, 96), (32, 48)),
            Cell::Blank => Sprite::new((0, 0), (32, 48)),
        }
    }
}

pub struct Board {
    width: i32,
    height: i32,
    cells: Vec<Cell>
}

impl Board {
    pub fn new(width: i32, height: i32) -> Self {
        let mut cells = vec![Cell::Blank; (width * height) as usize];

        for (n, cell) in cells.iter_mut().enumerate() {
            let (x, y) = (n % width as usize, n / width as usize);
            *cell = if x == 0 || y == 0 {
                Cell::TallWall
            } else if x == (width - 1) as usize || y == (height - 1) as usize {
                Cell::TallWall
            } else {
                Board::square_color((x as i32, y as i32))
            }
        }

        Self { width, height, cells }
    }

    pub fn square_color(coord: (i32, i32)) -> Cell {
        let (x, y) = coord;
        if (x + y) % 2 == 0 {
            Cell::White
        } else {
            Cell::Black
        }
    }
}

impl Grid for Board {
    type CellType = Cell;

    fn size(&self) -> Coord {
        xy(self.width, self.height)
    }

    fn default(&self) -> Self::CellType {
        Cell::Blank
    }

    fn get(&self, index: Coord) -> Option<&Self::CellType> {
        if self.contains(index) {
            Some(&self.cells[(index.0 + index.1 * self.width) as usize])
        } else {
            None
        }
    }
}

impl GridMut for Board {
    fn get_mut(&mut self, index: Coord) -> Option<&mut Self::CellType> {
        if self.contains(index) {
            Some(&mut self.cells[(index.0 + index.1 * self.width) as usize])
        } else {
            None
        }
    }
}