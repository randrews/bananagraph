use cgmath::Vector2;
use rand::Rng;
use bananagraph::Sprite;
use grid::xy;
use crate::animation::MoveAnimation;
use crate::drawable::Drawable;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PieceColor {
    RED,
    YELLOW,
    GREEN,
    BLUE,
    PINK,
    PURPLE,
    EMPTY, // A placeholder for a blank cell; should never be assigned to a component or seen
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
    pub(crate) position: Vector2<i32>
}

impl Piece {
    pub fn new(color: PieceColor, position: impl Into<Vector2<i32>>) -> Self {
        assert_ne!(color, PieceColor::EMPTY);
        Self {
            color,
            position: position.into()
        }
    }

    pub fn base_sprite(&self) -> Sprite {
        match self.color {
            PieceColor::RED => Sprite::new((240, 240), (80, 80)),
            PieceColor::YELLOW => Sprite::new((0, 80), (80, 80)),
            PieceColor::GREEN => Sprite::new((80, 160), (80, 80)),
            PieceColor::BLUE => Sprite::new((160, 80), (80, 80)),
            PieceColor::PINK => Sprite::new((320, 160), (80, 80)),
            PieceColor::PURPLE => Sprite::new((400, 240), (80, 80)),
            PieceColor::EMPTY => unreachable!()
        }
    }

    pub fn as_drawable(&self, id: u32, screen_size: impl Into<Vector2<u32>>) -> Drawable {
        let screen_size = screen_size.into();
        let sprite = self.base_sprite().with_z(0.5).with_id(id);

        let margin = Vector2::new(
            (screen_size.x as f32 - 8.0 * 85.0) / 2.0,
            (screen_size.y as f32 - 8.0 * 85.0) / 2.0
        );

        Drawable::new(sprite, (
            self.position.x as f32 * 85.0 + margin.x,
            self.position.y as f32 * 85.0 + margin.y
        ))
    }

    pub fn swap_animations(piece1: Piece, piece2: Piece) -> (MoveAnimation, MoveAnimation) {
        let d = piece2.position - piece1.position;
        (
            MoveAnimation::new(xy(85 * d.x, 85 * d.y)),
            MoveAnimation::new(xy(-85 * d.x, -85 * d.y))
        )
    }
}
