use std::time::Duration;
use cgmath::{Deg, Point2};
use rand::Rng;
use bananagraph::{DrawingContext, Sprite, SpriteId};
use grid::{Coord, Grid, VecGrid};

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum ClickTarget {
    SPRITE { id: SpriteId },
    LOCATION { location: Point2<f64> }
}

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
pub enum Animation {
    SPIN { angle: Deg<f32>, coord: Coord }
}

impl Animation {
    pub fn applies_to(&self, coord: Coord) -> bool {
        match self {
            Animation::SPIN { coord: c, .. } => *c == coord
        }
    }

    pub fn tick(&mut self, board: &mut VecGrid<Piece>, dt: Duration) {
        match self {
            Animation::SPIN { angle, coord } => {
                let mut new_angle = *angle + Deg(360.0 * dt.as_millis() as f32 / 1000.0);
                //println!("anim: {}, {}", coord, angle);
                if new_angle >= Deg(360.0) {
                    new_angle = Deg(360.0)
                }
                *angle = new_angle;
                board[*coord].angle = new_angle;
            }
        }
    }

    pub fn finished(&self) -> bool {
        match self {
            Animation::SPIN { angle, .. } => *angle == Deg(360.0)
        }
    }
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

pub struct GameState<'a, R: Rng> {
    board: VecGrid<Piece>,
    rng: &'a mut R,
    screen: (u32, u32),
    animations: Vec<Animation>
}

impl<'a, R: Rng> GameState<'a, R> {
    pub fn new(rng: &'a mut R, screen: (u32, u32)) -> Self {
        let mut board = VecGrid::new((8, 8).into(), Piece::new(PieceColor::RED));

        for coord in board.size() {
            board[coord] = Piece::new_from_rand(rng);
        }

        Self {
            board,
            rng,
            screen,
            animations: vec![]
        }
    }

    pub fn tick(&mut self, dt: Duration) {
        for anim in self.animations.iter_mut() {
            anim.tick(&mut self.board, dt);
        }

        self.animations = self.animations.iter().filter_map(|a| if a.finished() { None } else { Some(*a) }).collect();
    }

    pub fn redraw(&self) -> Vec<Sprite> {
        let mut sprites = vec![];
        let dc = DrawingContext::new((self.screen.0 as f32, self.screen.1 as f32));
        let margin = (
            (self.screen.0 as f32 - 8.0 * 85.0) / 2.0,
            (self.screen.1 as f32 - 8.0 * 85.0) / 2.0
            );
        for (n, coord) in self.board.size().into_iter().enumerate() {
            let piece = self.board[coord];
            let sprite = piece.as_sprite().with_z(0.5).with_id(n as u32);
            let sprite = dc.place_rotated(sprite, (
                coord.0 as f32 * 85.0 + margin.0,
                coord.1 as f32 * 85.0 + margin.1
            ), piece.angle);
            sprites.push(sprite)
        }
        sprites
    }

    pub fn click(&mut self, target: ClickTarget) {
        if let ClickTarget::SPRITE { id} = target {
            let coord = self.board.coord(id as usize);
            if self.animations.iter().find(|a| a.applies_to(coord)).is_none() {
                self.animations.push(Animation::SPIN { coord, angle: Deg(0.0)})
            }
        }
    }
}