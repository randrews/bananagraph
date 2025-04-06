use cgmath::Point2;
use bananagraph::Sprite;

/// Which layer certain sprites are on
pub enum Layers {
    Dungeon
}

// This is not arbitrary! It maps to the order the textures are
// loaded in game_state
impl Into<u32> for Layers {
    fn into(self) -> u32 {
        use Layers::*;
        match self {
            Dungeon => 0,
        }
    }
}

/// All sprites in the game, more or less.
/// This enum is Into<Option<Sprite>>, so you can turn a valid one into a sprite.
/// The reason it's an option is that some represent animations, so they include
/// frame numbers, and there might not be a frame for that
pub enum SpriteType {
    /// A wall, including which neighbors are also walls (n / s / e / w)
    Wall((bool, bool, bool, bool)),

    /// Floor, with the number (0-1) of the variant
    Floor(u32),

    /// Door, open or not
    Door(bool),

    /// Player, with the frame number of the breathe animation
    Player(u32),
}

/// Obviously a lot of this is coupled to the exact layout of the spritesheet...
impl Into<Sprite> for SpriteType {
    fn into(self) -> Sprite {
        use SpriteType::*;
        use Layers::*;
        match self {
            Wall(nbrs) => wall_sprite(nbrs),

            Floor(variant) => {
                if variant % 2 == 0 {
                    Sprite::new((144, 128), (16, 16)).with_layer(Dungeon.into())
                } else {
                    Sprite::new((144, 96), (16, 16)).with_layer(Dungeon.into())
                }
            }

            Door(open) => {
                if open {
                    Sprite::new((96, 16), (16, 16)).with_layer(Dungeon.into())
                } else {
                    Sprite::new((96, 32), (16, 16)).with_layer(Dungeon.into())
                }
            }
            // A blank space. TODO: make a question mark or something
            _ => Sprite::new((96, 0), (16, 16)).with_layer(Dungeon.into())
        }
    }
}

/// Return the correct sprite for a wall section, given a tuple (n, s, e, w) of
/// which neighbors are also walls.
/// This is obviously highly dependent on the exact layout of tiles in dungeon.png.
/// It's broken out into a separate function because it's long.
fn wall_sprite(neighbors: (bool, bool, bool, bool)) -> Sprite {
    let origin = match neighbors {
        (false, false, false, false) => (5, 1),
        (true, true, true, true) => (5, 0),

        (true, true, false, false) => (4, 1),
        (false, false, true, true) => (3, 0),

        (true, false, false, false) => (0, 2),
        (false, false, true, false) => (2, 2),
        (false, true, false, false) => (1, 2),
        (false, false, false, true) => (3, 1),

        (false, true, true, true) => (0, 0),
        (true, true, false, true) => (1, 0),
        (true, false, true, true) => (1, 1),
        (true, true, true, false) => (0, 1),

        (false, true, true, false) => (2, 0),
        (false, true, false, true) => (4, 0),
        (true, false, true, false) => (2, 1),
        (true, false, false, true) => (4, 2),
    };

    Sprite::new(Point2::from(origin) * 16, (16, 16)).with_layer(Layers::Dungeon.into())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sprite_type() {
        // We're not gonna go nuts with this, the only way to reasonably test it is visually,
        // but we can at least spot check one:
        let spr: Sprite = SpriteType::Wall((true, false, true, false)).into();
        assert_eq!(spr.origin, Point2::from((32, 16)))
    }
}