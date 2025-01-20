use std::time::Duration;
use cgmath::Vector2;
use grid::Coord;
use crate::drawable::Drawable;

pub trait Animation {
    fn tick(&mut self, dt: Duration);
    fn apply_to(&self, drawable: Drawable) -> Drawable;
}

#[derive(Copy, Clone, Debug)]
pub struct Pulse {
    scale: f32,
    delta: f32
}

impl Pulse {
    pub fn new() -> Self {
        Self {
            scale: 1.0,
            delta: 1.0
        }
    }
}

impl Animation for Pulse {
    fn tick(&mut self, dt: Duration) {
        let bounds = 0.1;
        let mut new_scale = self.scale + self.delta * (bounds * dt.as_millis() as f32 / 200.0);
        if self.delta < 0.0 && new_scale <= 1.0 - bounds {
            new_scale = 1.0 - bounds;
            self.delta = self.delta * -1.0;
        } else if self.delta > 0.0 && new_scale >= 1.0 + bounds {
            new_scale = 1.0 + bounds;
            self.delta = self.delta * -1.0;
        }
        self.scale = new_scale;
    }

    fn apply_to(&self, drawable: Drawable) -> Drawable {
        drawable.with_scale((self.scale, 2.0 - self.scale))
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MoveAnimation {
    dest: Vector2<f32>,
    current: Vector2<f32>,
    remaining_duration: Duration
}

impl MoveAnimation {
    pub fn new(dest: Coord) -> Self {
        Self {
            dest: (dest.0 as f32, dest.1 as f32).into(),
            current: (0.0, 0.0).into(),
            remaining_duration: Duration::new(1, 0)
        }
    }

    pub fn running(&self) -> bool {
        self.remaining_duration.is_zero()
    }
}

impl Animation for MoveAnimation {
    fn tick(&mut self, dt: Duration) {
        todo!()
    }

    fn apply_to(&self, drawable: Drawable) -> Drawable {
        todo!()
    }
}