use cgmath::Deg;
use rand::{Rng, RngCore};
use bananagraph::Sprite;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PieceColor {
    RED,
    YELLOW,
    GREEN,
    BLUE,
    PINK,
    PURPLE
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Piece {
    color: PieceColor,
    pub(crate) angle: Deg<f32>
}

impl Piece {
    pub fn new(color: PieceColor) -> Self {
        Self {
            color,
            angle: Deg(0.0)
        }
    }

    pub fn new_from_rand<R: Rng + ?Sized>(rng: &mut R) -> Self {
        match rng.next_u32() % 6 {
            0 => Self::new(PieceColor::RED),
            1 => Self::new(PieceColor::YELLOW),
            2 => Self::new(PieceColor::GREEN),
            3 => Self::new(PieceColor::BLUE),
            4 => Self::new(PieceColor::PINK),
            5 => Self::new(PieceColor::PURPLE),
            _ => unreachable!()
        }
    }

    pub fn as_sprite(&self) -> Sprite {
        match self.color {
            PieceColor::RED => Sprite::new((240, 240), (80, 80)),
            PieceColor::YELLOW => Sprite::new((0, 80), (80, 80)),
            PieceColor::GREEN => Sprite::new((80, 160), (80, 80)),
            PieceColor::BLUE => Sprite::new((160, 80), (80, 80)),
            PieceColor::PINK => Sprite::new((320, 160), (80, 80)),
            PieceColor::PURPLE => Sprite::new((400, 240), (80, 80)),
        }
    }
}
