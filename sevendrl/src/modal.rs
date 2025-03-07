use cgmath::Vector2;
use hecs::World;
use bananagraph::{DrawingContext, Sprite, Typeface};

#[derive(Clone, Debug, PartialEq)]
pub enum DismissType {
    Any,
    //Letter(String),
}

#[derive(Clone, Debug)]
pub enum ContentType {
    Center(String),
    Text(String),
    CenterSprite(Sprite),
}

#[derive(Clone, Debug)]
pub struct Modal {
    pub size: Vector2<i32>,
    pub contents: Vec<ContentType>,
    pub dismiss: DismissType
}

impl Modal {
    pub fn new(size: impl Into<Vector2<i32>>, contents: Vec<ContentType>, dismiss: DismissType) -> Self {
        Self {
            size: size.into(),
            contents,
            dismiss
        }
    }

    pub fn system(world: &World, typeface: &Typeface) -> Vec<Sprite> {
        if let Some((_, modal)) = world.query::<&Modal>().into_iter().next() {
            let mut sprites = vec![];
            let dims = Vector2::new(960.0 / 2.0, 544.0 / 2.0);
            let dc = DrawingContext::new(dims);

            let corners = (
                Sprite::new((54, 38), (16, 16)).with_layer(2).with_z(0.2),
                Sprite::new((90, 38), (16, 16)).with_layer(2).with_z(0.2),
                Sprite::new((54, 75), (16, 16)).with_layer(2).with_z(0.2),
                Sprite::new((90, 75), (16, 16)).with_layer(2).with_z(0.2),
                );
            let edges = (
                Sprite::new((70, 38), (16, 16)).with_layer(2).with_z(0.2),
                Sprite::new((90, 53), (16, 16)).with_layer(2).with_z(0.2),
                Sprite::new((75, 75), (16, 16)).with_layer(2).with_z(0.2),
                Sprite::new((54, 54), (16, 16)).with_layer(2).with_z(0.2),
            );
            let middle = Sprite::new((16, 48), (16, 16)).with_layer(2).with_z(0.21);

            // The screen is 30x17 tiles in size. We'll center our modal in the screen, so:
            let size = modal.size;
            let topleft = Vector2::new((30 - size.x) as f32 / 2.0, (17 - size.y) as f32 / 2.0) * 16.0;

            for y in 0..size.y {
                for x in 0..size.x {
                    let spr = if (x, y) == (0, 0) { corners.0 }
                    else if (x, y) == (size.x - 1, 0) { corners.1 }
                    else if (x, y) == (0, size.y - 1) { corners.2 }
                    else if (x, y) == (size.x - 1, size.y - 1) { corners.3 }
                    else if y == 0 { edges.0 }
                    else if x == size.x - 1 { edges.1 }
                    else if y == size.y - 1 { edges.2 }
                    else if x == 0 { edges.3 }
                    else { middle };
                    sprites.push(dc.place(spr, Vector2::new(x as f32, y as f32) * 16.0 + topleft));
                }
            }

            // Draw the contents
            let mut y = topleft.y + 4.0; // What our current y coord is
            for con in modal.contents.iter() {
                match con {
                    ContentType::Center(s) => {
                        // Draw a centered line
                        let w = typeface.width(s.as_str());
                        sprites.append(&mut typeface.print(dc, (topleft.x + size.x as f32 * 16.0 / 2.0 - w / 2.0, y + 13.0), 0.2, s.as_str()));
                        y += 13.0 + 1.0;
                    }
                    ContentType::Text(s) => {
                        // Draw some block text
                        sprites.append(&mut typeface.print(dc, (topleft.x + 8.0, y + 13.0), 0.2, s.as_str()));
                        y += (s.lines().count() as u32 * (typeface.height + 1)) as f32;
                    }
                    ContentType::CenterSprite(spr) => {
                        let x = topleft.x + size.x as f32 * 16.0 / 2.0 - spr.size.x as f32 / 2.0;
                        sprites.push(dc.place(spr.with_z(0.2), (x, y + 1.0)));
                        y += spr.size.y as f32 + 2.0
                    }
                }
            }


            sprites
        } else {
            vec![]
        }
    }
}