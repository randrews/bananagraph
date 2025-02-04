use std::time::Duration;
use cgmath::Vector2;
use hecs::{Component, World};
use crate::drawable::Drawable;

pub trait Animation {
    fn tick(&mut self, dt: Duration);
    fn apply_to(&self, drawable: Drawable) -> Drawable;
    fn running(&self) -> bool;
}

pub fn animation_system<T: Animation + Component + Send + Sync>(dt: Duration, world: &mut World) {
    let mut finished = vec![];
    for (ent, (anim,)) in world.query_mut::<(&mut T,)>() {
        anim.tick(dt);
        if !anim.running() { finished.push(ent) }
    }
    for ent in finished {
        world.remove_one::<T>(ent).unwrap();
    }
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
            self.delta *= -1.0;
        } else if self.delta > 0.0 && new_scale >= 1.0 + bounds {
            new_scale = 1.0 + bounds;
            self.delta *= -1.0;
        }
        self.scale = new_scale;
    }

    fn apply_to(&self, drawable: Drawable) -> Drawable {
        drawable.with_scale((self.scale, 2.0 - self.scale))
    }

    fn running(&self) -> bool {
        true
    }
}

#[derive(Copy, Clone, Debug)]
pub struct MoveAnimation {
    start: Vector2<f32>,
    duration: Duration,
    elapsed: Duration
}

impl MoveAnimation {
    pub fn new(start: impl Into<Vector2<i32>>) -> Self {
        let start = start.into();
        Self {
            start: (start.x as f32, start.y as f32).into(),
            duration: Duration::from_millis(250),
            elapsed: Duration::new(0, 0)
        }
    }
}

impl Animation for MoveAnimation {
    fn tick(&mut self, dt: Duration) {
        self.elapsed = (self.elapsed + dt).min(self.duration);
    }

    fn apply_to(&self, drawable: Drawable) -> Drawable {
        let fraction = self.elapsed.as_millis() as f32 / self.duration.as_millis() as f32;
        drawable.with_position_delta(self.start * (1.0 - fraction))
    }

    fn running(&self) -> bool {
        self.duration > self.elapsed
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Fade {
    duration: Duration,
    elapsed: Duration
}

impl Fade {
    pub fn new() -> Self {
        Self {
            duration: Duration::from_millis(250),
            elapsed: Duration::new(0, 0)
        }
    }
}

impl Animation for Fade {
    fn tick(&mut self, dt: Duration) {
        self.elapsed = (self.elapsed + dt).min(self.duration);
    }

    fn apply_to(&self, drawable: Drawable) -> Drawable {
        let fraction = self.elapsed.as_millis() as f32 / self.duration.as_millis() as f32;
        drawable.with_tint((1.0, 1.0, 1.0, 1.0 - fraction))
    }

    fn running(&self) -> bool {
        self.duration > self.elapsed
    }
}