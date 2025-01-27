use std::collections::BTreeMap;
use cgmath::{Point2, Vector2};
use image::{DynamicImage, GenericImage, GenericImageView};
use bananagraph::{DrawingContext, GpuWrapper, Sprite};

pub struct TypefaceBuilder {
    /// The image data, used for automatically adding glyphs
    image: DynamicImage,

    /// The map from character to Glyph
    glyphs: BTreeMap<char, Glyph>,

    /// When we print a line, the y coord will be the baseline of the text. This is
    /// how far above the bottom of each glyph's rect (in the image) the baseline is
    baseline: u32
}

#[derive(Clone)]
pub struct Typeface {
    pub(crate) glyphs: BTreeMap<char, Glyph>
}

#[derive(Copy, Clone, Debug)]
pub struct Glyph {
    /// The base sprite for the glyph
    pub sprite: Sprite,

    /// The amount we need to shift the character to account for baseline
    pub offset: Vector2<i32>,

    /// The actual size of the glyph
    pub size: Vector2<u32>
}

/// A trait to allow us to create TypefaceBuilders without a real GPU wrapper (tests)
pub trait AddTexture {
    fn add_texture_from_array(&mut self, bytes: Vec<u8>, width: u32, name: Option<&str>) -> u32;
}

impl AddTexture for GpuWrapper<'_> {
    fn add_texture_from_array(&mut self, bytes: Vec<u8>, width: u32, name: Option<&str>) -> u32 {
        self.add_texture_from_array(bytes, width, name)
    }
}

impl TypefaceBuilder {
    pub fn new(img_bytes: &[u8], baseline: u32) -> Self {
        let mut image = image::load_from_memory(img_bytes).expect("Image could not be parsed for typeface");

        // In order to have a texture we can tint, every pixel in it needs to be either transparent or pure white:
        for y in 0..image.height() {
            for x in 0..image.width() {
                let pix = image.get_pixel(x, y);
                if pix.0[3] == 0 {
                    image.put_pixel(x, y, [0, 0, 0, 0].into())
                } else {
                    image.put_pixel(x, y, [0xff, 0xff, 0xff, 0xff].into())
                }
            }
        }


        Self {
            image,
            baseline,
            glyphs: BTreeMap::new()
        }
    }

    pub fn add_glyph(&mut self, ch: char, size: impl Into<Vector2<u32>>, topleft: impl Into<Point2<u32>>) {
        let (size, topleft) = (size.into(), topleft.into());
        let mut top = -1;
        let mut bottom = -1;
        let mut right = -1;
        for y in topleft.y .. (topleft.y + size.y) {
            for x in topleft.x .. (topleft.x + size.x) {
                if self.image.get_pixel(x, y).0[0] == 0 { continue }
                let (local_x, local_y) = (x as i32 - topleft.x as i32, y as i32 - topleft.y as i32);
                if top == -1 { top = local_y; }
                if local_x > right { right = local_x; }
                bottom = local_y;
            }
        }

        let glyph = Glyph {
            sprite: Sprite::new(topleft, size),
            offset: (0, self.baseline as i32 - size.y as i32).into(),
            size: (1 + right as u32, (bottom - top) as u32).into()
        };

        self.glyphs.insert(ch, glyph);
    }

    /// Adds a glyph where the size is not shrunken to the used area; useful for things like whitespace
    /// or some punctuation
    pub fn add_sized_glyph(&mut self, ch: char, size: impl Into<Vector2<u32>>, topleft: impl Into<Point2<u32>>) {
        let (size, topleft) = (size.into(), topleft.into());

        let glyph = Glyph {
            sprite: Sprite::new(topleft, size),
            offset: (0, self.baseline as i32 - size.y as i32).into(),
            size
        };

        self.glyphs.insert(ch, glyph);
    }

    pub fn add_glyphs<'a>(&mut self, line: impl Into<&'a str>, size: impl Into<Vector2<u32>>, topleft: impl Into<Point2<u32>>, separation: Option<u32>) {
        let (size, topleft) = (size.into(), topleft.into());
        let line = line.into();
        let separation = separation.unwrap_or(0);
        for (n, ch) in line.chars().into_iter().enumerate() {
            let n = n as u32;
            let topleft = Point2::new(topleft.x + n * (size.x + separation), topleft.y);
            self.add_glyph(ch, size, topleft)
        }
    }

    pub fn into_typeface(self, gpu_wrapper: &mut impl AddTexture) -> Typeface {
        let layer = gpu_wrapper.add_texture_from_array(Vec::from(self.image.as_bytes()), self.image.width(), None);
        let glyphs = self.glyphs.into_iter().map(|(ch, glyph)| (ch, glyph.with_layer(layer))).collect();
        Typeface {
            glyphs
        }
    }
}

impl Typeface {
    pub fn print<'a>(&self, dc: DrawingContext, at: impl Into<Vector2<f32>>, s: impl Into<&'a str>) -> Vec<Sprite> {
        let mut sprites = vec![];
        let mut x = 0f32;
        let at = at.into();
        for (n, ch) in s.into().chars().into_iter().enumerate() {
            if let Some(glyph) = self.glyphs.get(&ch) {
                let sprite = dc.place(glyph.sprite, (
                    at.x + x + n as f32,
                    at.y + glyph.offset.y as f32
                ));
                sprites.push(sprite);
                x = x + glyph.size.x as f32;
            } else {
                x = x + 8.0; // Just leave a blank space...
            }
        }
        sprites
    }
}

impl Glyph {
    pub(crate) fn with_layer(self, layer: u32) -> Self {
        Self {
            sprite: self.sprite.with_layer(layer),
            ..self
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestGpu {}
    impl AddTexture for TestGpu {
        fn add_texture_from_array(&mut self, bytes: Vec<u8>, width: u32, name: Option<&str>) -> u32 {
            0
        }
    }

    fn test_create_builder() {
        let mut builder = TypefaceBuilder::new(include_bytes!("Curly-Girly.png"), 4);
        builder.add_glyph('a', (7, 15), (1, 65));
        let tf: Typeface = builder.into_typeface(&mut TestGpu {});
        let g = tf.glyphs.get(&'a').unwrap();
        assert_eq!(g.size, (5, 5).into());
        assert_eq!(g.offset, (0, 4).into());
        assert_eq!(g.sprite.layer, 0);
        assert_eq!(g.sprite.origin, (1, 65).into());
        assert_eq!(g.sprite.size, (7, 15).into());
    }

    fn test_add_glyphs() {
        let mut builder = TypefaceBuilder::new(include_bytes!("Curly-Girly.png"), 4);
        builder.add_glyphs("abcdefgh", (7, 15), (1, 65), Some(1));
        let tf: Typeface = builder.into_typeface(&mut TestGpu {});
        assert_eq!(tf.glyphs.len(), 8);

        let g = tf.glyphs.get(&'h').unwrap();
        assert_eq!(g.size.x, 5);
    }

    fn test_print() {
        let dc = DrawingContext::new((100.0, 100.0));
        let mut builder = TypefaceBuilder::new(include_bytes!("Curly-Girly.png"), 4);
        builder.add_glyphs("abcdefgh", (7, 15), (1, 65), Some(1));
        builder.add_glyphs("ijklmnop", (7, 15), (1, 81), Some(1));
        let tf: Typeface = builder.into_typeface(&mut TestGpu {});
        let sprites = tf.print(dc, (0.0, 50.0), "foo");
        assert_eq!(sprites.len(), 3);
    }
}