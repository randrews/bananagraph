use rand::{Rng, RngCore};
use bananagraph::Sprite;
use grid::Coord;
use crate::drawable::Drawable;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PieceColor {
    RED,
    YELLOW,
    GREEN,
    BLUE,
    PINK,
    PURPLE
}

impl PieceColor {
    pub fn from_rand<R: Rng + ?Sized>(rng: &mut R) -> Self {
        match rng.next_u32() % 6 {
            0 => PieceColor::RED,
            1 => PieceColor::YELLOW,
            2 => PieceColor::GREEN,
            3 => PieceColor::BLUE,
            4 => PieceColor::PINK,
            5 => PieceColor::PURPLE,
            _ => unreachable!()
        }
    }
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Piece {
    pub(crate) color: PieceColor,
}

impl Piece {
    pub fn new(color: PieceColor) -> Self {
        Self {
            color
        }
    }

    pub fn new_from_rand<R: Rng + ?Sized>(rng: &mut R) -> Self {
        Self::new(PieceColor::from_rand(rng))
    }

    pub fn base_sprite(&self) -> Sprite {
        match self.color {
            PieceColor::RED => Sprite::new((240, 240), (80, 80)),
            PieceColor::YELLOW => Sprite::new((0, 80), (80, 80)),
            PieceColor::GREEN => Sprite::new((80, 160), (80, 80)),
            PieceColor::BLUE => Sprite::new((160, 80), (80, 80)),
            PieceColor::PINK => Sprite::new((320, 160), (80, 80)),
            PieceColor::PURPLE => Sprite::new((400, 240), (80, 80)),
        }
    }

    pub fn as_drawable(&self, id: u32, coord: Coord, screen_size: (u32, u32)) -> Drawable {
        let sprite = self.base_sprite().with_z(0.5).with_id(id);

        let margin = (
            (screen_size.0 as f32 - 8.0 * 85.0) / 2.0,
            (screen_size.1 as f32 - 8.0 * 85.0) / 2.0
        );

        Drawable::new(sprite, (
            coord.0 as f32 * 85.0 + margin.0,
            coord.1 as f32 * 85.0 + margin.1
        ))
    }
}
