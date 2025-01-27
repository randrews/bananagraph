use bananagraph::{DrawingContext, Sprite};
use cgmath::{Deg, Vector2, Vector4};

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Drawable {
    pub(crate) sprite: Sprite,
    pub(crate) angle: Deg<f32>,
    pub(crate) scale: Vector2<f32>,
    pub(crate) position: Vector2<f32>
}

#[allow(unused)]
impl Drawable {
    pub fn new(sprite: Sprite, position: impl Into<Vector2<f32>>) -> Self {
        Self {
            sprite,
            position: position.into(),
            angle: Deg(0.0),
            scale: (1.0, 1.0).into(),
        }
    }

    pub fn as_sprite(&self, dc: DrawingContext) -> Sprite {
        dc.place_scaled_rotated(self.sprite, self.position, self.scale, self.angle)
    }

    pub fn with_angle(self, angle: impl Into<Deg<f32>>) -> Self {
        Self {
            angle: angle.into(),
            ..self
        }
    }

    pub fn with_scale(self, scale: impl Into<Vector2<f32>>) -> Self {
        Self {
            scale: scale.into(),
            ..self
        }
    }

    pub fn with_tint(self, tint: impl Into<Vector4<f32>>) -> Self {
        Self {
            sprite: self.sprite.with_tint(tint),
            ..self
        }
    }

    pub fn with_position_delta(self, delta: impl Into<Vector2<f32>>) -> Self {
        Self {
            position: self.position + delta.into(),
            ..self
        }
    }
}