use cgmath::{Matrix3, SquareMatrix, Vector2};

#[derive(Copy, Clone, Debug)]
pub struct DrawingContext {
    screen: Vector2<u32>,
    transform: Matrix3<f32>
}

impl DrawingContext {
    pub fn new(screen: impl Into<Vector2<u32>>) -> Self {
        let screen = screen.into();
        Self {
            screen,
            transform: Matrix3::identity()
        }
    }

    /// Return a sprite with the transform matrix scaled by these factors
    pub fn scale(self, factor: impl Into<Vector2<f32>>) -> Self {
        let factor = factor.into();
        // For some reason cgmath doesn't have a helper for nonuniform scaling?
        let scale = Matrix3::new(
            factor.x, 0.0, 0.0,
            0.0, factor.y, 0.0,
            0.0, 0.0, 1.0
        );

        Self {
            transform: scale * self.transform,
            ..self
        }
    }

    /// Return a sprite with the transform matrix translated by this vector
    pub fn translate(self, delta: impl Into<Vector2<f32>>) -> Self {
        Self {
            transform: Matrix3::from_translation(delta.into()) * self.transform,
            ..self
        }
    }

}