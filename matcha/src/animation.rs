use std::time::Duration;
use cgmath::Vector2;
use hecs::World;
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

    pub fn system(dt: Duration, world: &mut World) {
        for (_ent, (anim,)) in world.query_mut::<(&mut Pulse,)>() {
            anim.tick(dt);
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
    duration: Duration,
    elapsed: Duration
}

impl MoveAnimation {
    pub fn new(dest: Coord) -> Self {
        Self {
            dest: (dest.0 as f32, dest.1 as f32).into(),
            duration: Duration::new(0, 500000000),
            elapsed: Duration::new(0, 0)
        }
    }

    pub fn system(dt: Duration, world: &mut World) {
        let mut finished = vec![];
        for (ent, (anim,)) in world.query_mut::<(&mut MoveAnimation,)>() {
            anim.tick(dt);
            if !anim.running() { finished.push(ent) }
        }
        for ent in finished {
            world.remove_one::<MoveAnimation>(ent).unwrap();
        }
    }

    pub fn running(&self) -> bool {
        self.duration > self.elapsed
    }
}

impl Animation for MoveAnimation {
    fn tick(&mut self, dt: Duration) {
        self.elapsed = (self.elapsed + dt).min(self.duration);
    }

    fn apply_to(&self, drawable: Drawable) -> Drawable {
        let fraction = self.elapsed.as_millis() as f32 / self.duration.as_millis() as f32;
        println!("fraction: {}", fraction);
        drawable.with_position_delta(self.dest * fraction)
    }
}