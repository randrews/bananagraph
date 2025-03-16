use bananagraph::Sprite;
use std::time::Duration;
use hecs::World;
use crate::components::{Frozen, OnMap};
use crate::components::visible::Visible;

/// An animation that runs a list of sprite frames in a loop. A breathe animation has three
/// qualities:
/// - `frames` is the list of frames, which are displayed in sequence over and over
/// - `rate` is the length of time to show each frame before passing to the next
/// - `timer` is an internal clock of how long the animation has been running
#[derive(Clone, Debug)]
pub struct BreatheAnimation {
    frames: Vec<Sprite>,
    rate: Duration,
    timer: Duration
}

impl BreatheAnimation {
    /// Create a new breathe animation with the given list of frames and default rate and timer.
    /// The default rate is 200ms per frame
    pub fn new(frames: Vec<Sprite>) -> Self {
        Self {
            frames,
            rate: Duration::from_millis(200),
            timer: Duration::from_millis(0)
        }
    }

    /// Create a new breathe animation with a given (probably random) timer
    /// value, and 200ms rate. Call this with random values to make it so
    /// things with the same animation don't all happen in unison.
    pub fn new_with_start(frames: Vec<Sprite>, start: Duration) -> Self {
        Self {
            frames,
            rate: Duration::from_millis(200),
            timer: start
        }
    }

    /// The current frame to display from the list
    pub fn current_frame(&self) -> Sprite {
        let total = self.frames.len() * self.rate.as_millis() as usize;
        let t = self.timer.as_millis() as usize % total;
        self.frames[t / self.rate.as_millis() as usize]
    }

    pub fn system(world: &mut World, dt: Duration) {
        for (_, (breathe, visible, frozen)) in world.query_mut::<(&mut BreatheAnimation, &mut Visible, Option<&Frozen>)>() {
            if frozen.is_some() { continue } // This thing isn't animating at the moment
            breathe.timer += dt;
            visible.0 = breathe.current_frame();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frames() -> Vec<Sprite> {
        vec![
            Sprite::new((0, 0), (16, 16)),
            Sprite::new((16, 0), (16, 16)),
            Sprite::new((32, 0), (16, 16)),
            Sprite::new((48, 0), (16, 16)),
            Sprite::new((64, 0), (16, 16)),
        ]
    }

    #[test]
    fn test_current_frame() {
        // Starts at 0
        let ba = BreatheAnimation::new_with_start(frames(), Duration::from_millis(0));
        assert_eq!(ba.current_frame(), frames()[0]);

        // Increases every 200ms
        let ba = BreatheAnimation::new_with_start(frames(), Duration::from_millis(200));
        assert_eq!(ba.current_frame(), frames()[1]);

        // Wraps around
        let ba = BreatheAnimation::new_with_start(frames(), Duration::from_millis(1100));
        assert_eq!(ba.current_frame(), frames()[0]);
    }

    #[test]
    fn test_system() {
        // Create one animating, one non-animating, and one frozen entity
        let mut w = World::new();
        let animating = w.spawn((BreatheAnimation::new(frames()), Visible(frames()[0])));
        let still = w.spawn((Visible(frames()[0]),));
        let frozen = w.spawn((BreatheAnimation::new(frames()), Visible(frames()[0]), Frozen));

        // Tick everyone forward a frame
        BreatheAnimation::system(&mut w, Duration::from_millis(200));

        // Animating guy is forward a frame
        assert_eq!(*w.query_one::<&Visible>(animating).unwrap().get().unwrap(), Visible(frames()[1]));

        // We didn't touch still guy or frozen guy
        assert_eq!(*w.query_one::<&Visible>(still).unwrap().get().unwrap(), Visible(frames()[0]));
        assert_eq!(*w.query_one::<&Visible>(frozen).unwrap().get().unwrap(), Visible(frames()[0]));
    }
}

/*
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
*/