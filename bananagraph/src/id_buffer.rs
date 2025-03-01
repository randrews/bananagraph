use std::ops::Index;
use cgmath::Point2;
use crate::sprite::SpriteId;

pub struct IdBuffer {
    data: Vec<SpriteId>,
    width: u32,
    screen_width: u32
}

impl IdBuffer {
    pub fn new(data: Vec<SpriteId>, width: u32, screen_width: u32) -> Self {
        Self { data, width, screen_width }
    }

    /// Returns whether a given point is within the logical area of the screen
    /// (the total id buffer will be larger than this, probably)
    pub fn contains(&self, pt: Point2<u32>) -> bool {
        pt.x < self.screen_width && pt.y < self.data.len() as u32 / self.width
    }
}

impl Index<Point2<f64>> for IdBuffer {
    type Output = SpriteId;

    fn index(&self, index: Point2<f64>) -> &Self::Output {
        let i = index.x as usize + index.y as usize * self.width as usize;
        if i < self.data.len() {
            &self.data[i]
        } else {
            &0
        }
    }
}