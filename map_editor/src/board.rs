use cgmath::Vector2;
use bananagraph::Sprite;
use grid::{Grid, GridMut};
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
            *cell = if x == 0 || y == 0 || x == (width - 1) as usize || y == (height - 1) as usize {
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

    fn size(&self) -> Vector2<i32> {
        (self.width, self.height).into()
    }

    fn default(&self) -> Self::CellType {
        Cell::Blank
    }

    fn get(&self, index: impl Into<Vector2<i32>>) -> Option<&Self::CellType> {
        let index = index.into();
        if self.contains(index) {
            Some(&self.cells[(index.x + index.y * self.width) as usize])
        } else {
            None
        }
    }
}

impl GridMut for Board {
    fn get_mut(&mut self, index: impl Into<Vector2<i32>>) -> Option<&mut Self::CellType> {
        let index = index.into();
        if self.contains(index) {
            Some(&mut self.cells[(index.x + index.y * self.width) as usize])
        } else {
            None
        }
    }
}