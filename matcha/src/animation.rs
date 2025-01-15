use std::time::Duration;
use cgmath::Deg;

#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Animation {
    SPIN { angle: Deg<f32> }
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
            }
        }
    }

    pub fn finished(&self) -> bool {
        match self {
            Animation::SPIN { angle, .. } => *angle == Deg(360.0)
        }
    }
}
