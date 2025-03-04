use bananagraph::Sprite;
use std::time::Duration;
use hecs::World;
use crate::components::OnMap;

#[derive(Clone, Debug)]
pub struct BreatheAnimation {
    frames: Vec<Sprite>,
    rate: Duration,
    timer: Duration
}

impl BreatheAnimation {
    pub fn new(frames: Vec<Sprite>) -> Self {
        Self {
            frames,
            rate: Duration::from_millis(200),
            timer: Duration::from_millis(0)
        }
    }

    pub fn new_with_start(frames: Vec<Sprite>, start: Duration) -> Self {
        Self {
            frames,
            rate: Duration::from_millis(200),
            timer: start
        }
    }

    pub fn current_frame(&self) -> Sprite {
        let total = self.frames.len() * self.rate.as_millis() as usize;
        let t = self.timer.as_millis() as usize % total;
        self.frames[t / self.rate.as_millis() as usize]
    }

    pub fn system(world: &mut World, dt: Duration) {
        for (_, (breathe, on_map)) in world.query_mut::<(&mut BreatheAnimation, &mut OnMap)>() {
            breathe.timer += dt;
            on_map.sprite = breathe.current_frame();
        }
    }
}