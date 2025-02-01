use cgmath::Vector2;
use rand::Rng;
use bananagraph::Sprite;
use crate::animation::MoveAnimation;
use crate::drawable::Drawable;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum PieceColor {
    Red,
    Yellow,
    Green,
    Blue,
    Pink,
    Purple,
    Empty, // A placeholder for a blank cell; should never be assigned to a component or seen
}

impl PieceColor {
    pub fn from_rand<R: Rng + ?Sized>(rng: &mut R) -> Self {
        match rng.next_u32() % 6 {
            0 => PieceColor::Red,
            1 => PieceColor::Yellow,
            2 => PieceColor::Green,
            3 => PieceColor::Blue,
            4 => PieceColor::Pink,
            5 => PieceColor::Purple,
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
        assert_ne!(color, PieceColor::Empty);
        Self {
            color,
            position: position.into()
        }
    }

    pub fn base_sprite(&self) -> Sprite {
        match self.color {
            PieceColor::Red => Sprite::new((240, 240), (80, 80)),
            PieceColor::Yellow => Sprite::new((0, 80), (80, 80)),
            PieceColor::Green => Sprite::new((80, 160), (80, 80)),
            PieceColor::Blue => Sprite::new((160, 80), (80, 80)),
            PieceColor::Pink => Sprite::new((320, 160), (80, 80)),
            PieceColor::Purple => Sprite::new((400, 240), (80, 80)),
            PieceColor::Empty => unreachable!()
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
            MoveAnimation::new((85 * d.x, 85 * d.y)),
            MoveAnimation::new((-85 * d.x, -85 * d.y))
        )
    }
}
