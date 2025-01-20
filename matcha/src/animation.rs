use std::time::Duration;
use crate::drawable::Drawable;

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

    pub fn tick(&mut self, dt: Duration) {
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

    pub fn apply_to(&self, drawable: Drawable) -> Drawable {
        drawable.with_scale((self.scale, 2.0 - self.scale))
    }
}