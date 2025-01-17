use std::time::Duration;
use cgmath::Deg;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Animation {
    SPIN { angle: Deg<f32> },
    PULSE { scale: f32, delta: f32, run: bool }
}

impl Animation {
    pub fn tick(&mut self, dt: Duration) {
        match self {
            Animation::SPIN { angle} => {
                let mut new_angle = *angle + Deg(360.0 * dt.as_millis() as f32 / 1000.0);
                if new_angle >= Deg(360.0) {
                    new_angle = Deg(360.0)
                }
                *angle = new_angle;
            },

            Animation::PULSE { scale, delta, ..} => {
                let bounds = 0.1;
                let mut new_scale = *scale + *delta * (bounds * dt.as_millis() as f32 / 200.0);
                if *delta < 0.0 && new_scale <= 1.0 - bounds {
                    new_scale = 1.0 - bounds;
                    *delta = *delta * -1.0;
                } else if *delta > 0.0 && new_scale >= 1.0 + bounds {
                    new_scale = 1.0 + bounds;
                    *delta = *delta * -1.0;
                }
                *scale = new_scale;
            }
        }
    }

    pub fn finished(&self) -> bool {
        match self {
            Animation::SPIN { angle, .. } => *angle == Deg(360.0),
            Animation::PULSE { run, .. } => !*run
        }
    }
}
