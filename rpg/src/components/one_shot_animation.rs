use std::time::Duration;
use hecs::{ComponentError, World};
use bananagraph::Sprite;
use crate::components::{Frozen, Visible};

/// An animation that runs a list of sprite frames once, and then removes itself.
/// A breathe animation has three qualities:
/// - `frames` is the list of frames, which are displayed in sequence over and over
/// - `rate` is the length of time to show each frame before passing to the next
/// - `timer` is an internal clock of how long the animation has been running
#[derive(Clone, Debug)]
pub struct OneShotAnimation {
    frames: Vec<Sprite>,
    rate: Duration,
    timer: Duration
}

impl OneShotAnimation {
    /// A new animation with a default rate of 80ms and a timer of 0.
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

    /// Run the animations:
    /// - Anything `Visible` and not `Frozen` will get updated
    /// - Anything that's displayed all the frames gets removed
    pub fn system(world: &mut World, dt: Duration) -> Result<(), ComponentError> {
        let mut graveyard = vec![];
        for (ent, (anim, visible, frozen)) in world.query_mut::<(&mut OneShotAnimation, &mut Visible, Option<&Frozen>)>() {
            if frozen.is_some() { continue } // Skip frozen ones
            anim.timer += dt;
            if let Some(frame) = anim.current_frame() {
                visible.0 = frame;
            } else {
                graveyard.push(ent);
            }
        }

        for e in graveyard.into_iter() {
            world.remove_one::<OneShotAnimation>(e).map(|_| ())?
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use hecs::Entity;
    use super::*;

    fn frames() -> Vec<Sprite> {
        vec![
            Sprite::new((0, 0), (16, 16)),
            Sprite::new((16, 0), (16, 16)),
            Sprite::new((32, 0), (16, 16)),
        ]
    }

    #[test]
    fn test_current_frame() {
        let mut anim = OneShotAnimation::new(frames());
        // Starts at frame 0
        assert_eq!(anim.current_frame(), Some(frames()[0]));

        // Increments to new frames
        anim.timer += Duration::from_millis(80);
        assert_eq!(anim.current_frame(), Some(frames()[1]));

        // Finishes the frame list
        anim.timer += Duration::from_millis(160);
        assert_eq!(anim.current_frame(), None);
    }

    fn entity_frame(world: &World, ent: Entity) -> Sprite {
        world.query_one::<&Visible>(ent).unwrap().get().unwrap().0
    }

    #[test]
    fn test_system() {
        let mut world = World::new();
        let e1 = world.spawn((Visible(frames()[0]), OneShotAnimation::new(frames())));
        let e2 = world.spawn((Visible(frames()[0]), OneShotAnimation::new(frames()), Frozen));

        // Start by ticking frames
        OneShotAnimation::system(&mut world, Duration::from_millis(80)).unwrap();
        assert_eq!(entity_frame(&world, e1), frames()[1]); // First one ticks
        assert_eq!(entity_frame(&world, e2), frames()[0]); // Second is frozen

        // After it's finished, it's removed
        OneShotAnimation::system(&mut world, Duration::from_millis(160)).unwrap();
        assert!(world.query_one::<Option<&OneShotAnimation>>(e1).unwrap().get().unwrap().is_none());
    }
}