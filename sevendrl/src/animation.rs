use bananagraph::Sprite;
use std::time::Duration;
use hecs::World;
use crate::components::OnMap;
use crate::enemy::Enemy;
use crate::scrolls::TimeFreezeEffect;

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
        let frozen = TimeFreezeEffect::time_freeze_remaining(world).is_some();
        for (_, (breathe, on_map, enemy)) in world.query_mut::<(&mut BreatheAnimation, &mut OnMap, Option<&Enemy>)>() {
            if frozen && enemy.is_some() { continue } // Enemies are frozen!
            breathe.timer += dt;
            on_map.sprite = breathe.current_frame();
        }
    }
}

#[derive(Clone, Debug)]
pub struct OneShotAnimation {
    frames: Vec<Sprite>,
    rate: Duration,
    timer: Duration
}

impl OneShotAnimation {
    pub fn new(frames: Vec<Sprite>) -> Self {
        Self {
            frames,
            rate: Duration::from_millis(80),
            timer: Duration::from_millis(0)
        }
    }

    pub fn current_frame(&self) -> Option<Sprite> {
        let t = self.timer.as_millis() as usize;
        let idx = t / self.rate.as_millis() as usize;
        self.frames.get(idx).copied()
    }

    pub fn system(world: &mut World, dt: Duration) {
        let mut graveyard = vec![];
        for (ent, (anim, on_map)) in world.query_mut::<(&mut OneShotAnimation, &mut OnMap)>() {
            anim.timer += dt;
            if let Some(frame) = anim.current_frame() {
                on_map.sprite = frame;
            } else {
                graveyard.push(ent);
            }
        }

        for e in graveyard.into_iter() {
            world.despawn(e).unwrap()
        }
    }
}
