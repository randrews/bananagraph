use cgmath::{Deg, Matrix3, Rad, SquareMatrix, Vector2};
use crate::Sprite;

#[derive(Copy, Clone, Debug)]
pub struct DrawingContext {
    pub screen: Vector2<f32>,
    transform: Matrix3<f32>
}

impl DrawingContext {
    pub fn new(screen: impl Into<Vector2<f32>>) -> Self {
        let screen = screen.into();
        Self {
            screen,
            transform: Matrix3::identity()
        }
    }

    pub fn place(&self, sprite: impl Into<Sprite>, position: impl Into<Vector2<f32>>) -> Sprite {
        self.place_scaled_rotated(sprite, position, (1.0, 1.0), Deg(0.0))
    }

    pub fn place_rotated(&self, sprite: impl Into<Sprite>, position: impl Into<Vector2<f32>>, rotation: impl Into<Rad<f32>>) -> Sprite {
        self.place_scaled_rotated(sprite, position, (1.0, 1.0), rotation)
    }

    pub fn place_scaled(&self, sprite: impl Into<Sprite>, position: impl Into<Vector2<f32>>, scale: impl Into<Vector2<f32>>) -> Sprite {
        self.place_scaled_rotated(sprite, position, scale, Deg(0.0))
    }

    pub fn place_scaled_rotated(&self, sprite: impl Into<Sprite>, position: impl Into<Vector2<f32>>, scale: impl Into<Vector2<f32>>, rotation: impl Into<Rad<f32>>) -> Sprite {
        let sprite = sprite.into();
        // This is the transform we will eventually apply to the sprite
        let mut t = Matrix3::identity(); //self.transform;
        let scale = scale.into();
        let rotation = rotation.into();

        let scale = Matrix3::from_nonuniform_scale(scale.x, scale.y);
        let rotation = Matrix3::from_angle_z(rotation);

        let aspect_scale = Matrix3::from_nonuniform_scale(sprite.size.x as f32, sprite.size.y as f32);
        let invert_aspect_scale = Matrix3::invert(&aspect_scale).unwrap();

        // Translate so the center is on the origin, rotate, scale, and translate back.
        t = Matrix3::from_translation((-0.5, -0.5).into()) * t;
        t = invert_aspect_scale * scale * rotation * aspect_scale * t;
        t = scale * t;
        t = Matrix3::from_translation((0.5, 0.5).into()) * t;

        // We need to scale the sprite to the correct size:
        t = Matrix3::from_nonuniform_scale(sprite.size.x as f32 / self.screen.x, sprite.size.y as f32 / self.screen.y) * t;

        // Translate it to the coords in context space:
        let position = position.into();
        t = Matrix3::from_translation((1.0 / self.screen.x * position.x, 1.0 / self.screen.y * position.y).into()) * t;
        sprite.with_transform(self.transform * t)
    }

    /// Return a drawing context with the transform matrix scaled by these factors
    pub fn scale(self, factor: impl Into<Vector2<f32>>) -> Self {
        let factor = factor.into();
        Self {
            transform: Matrix3::from_nonuniform_scale(factor.x, factor.y) * self.transform,
            ..self
        }
    }

    /// Return a drawing context with the transform matrix translated by this vector
    pub fn translate(self, delta: impl Into<Vector2<f32>>) -> Self {
        Self {
            transform: Matrix3::from_translation(delta.into()) * self.transform,
            ..self
        }
    }

    /// Return a drawing context with the transform matrix rotated by this angle
    /// We have to temporarily scale it with the aspect ratio of the screen, or doing this
    /// distorts the context
    pub fn rotate(self, theta: impl Into<Rad<f32>>) -> Self {
        let rotation = Matrix3::from_angle_z(theta);
        let scale = Matrix3::from_nonuniform_scale(self.screen.x, self.screen.y);
        let invert_scale = Matrix3::invert(&scale).unwrap();

        Self {
            transform: invert_scale * rotation * scale * self.transform,
            ..self
        }
    }
}
