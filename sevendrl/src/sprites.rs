use bananagraph::Sprite;
use crate::sprites::AnimationSprites::{Enemy1, Enemy2, Enemy3};

pub trait SpriteFor {
    fn sprite(&self) -> Sprite;
}

pub enum AnimationSprites {
    Player1, // Player animation frames
    Player2,
    Player3,
    Enemy1, // Enemy breathe frames
    Enemy2,
    Enemy3,
    EnemyFade1, // Enemy fade animation frames
    EnemyFade2,
    EnemyFade3,
}

impl AnimationSprites {
    pub fn enemy_breathe() -> Vec<Sprite> {
        use AnimationSprites::*;
        [
            Enemy1,
            Enemy2,
            Enemy3,
            Enemy3,
            Enemy3,
            Enemy2,
        ].map(|a| a.sprite()).into_iter().collect()
    }

    pub fn player_breathe() -> Vec<Sprite> {
        use AnimationSprites::*;
        [
            Player1,
            Player2,
            Player3,
            Player3,
            Player3,
            Player2,
        ].map(|a| a.sprite()).into_iter().collect()
    }

    pub fn enemy_fade() -> Vec<Sprite> {
        use AnimationSprites::*;
        [
            EnemyFade1, EnemyFade2, EnemyFade3
        ].map(|a| a.sprite()).into_iter().collect()
    }
}

impl SpriteFor for AnimationSprites {
    fn sprite(&self) -> Sprite {
        use AnimationSprites::*;
        match self {
            Player1 => Sprite::new((0, 0), (16, 16)).with_layer(1),
            Player2 => Sprite::new((16, 0), (16, 16)).with_layer(1),
            Player3 => Sprite::new((32, 0), (16, 16)).with_layer(1),

            Enemy1 => Sprite::new((64, 16), (16, 16)).with_layer(4),
            Enemy2 => Sprite::new((80, 16), (16, 16)).with_layer(4),
            Enemy3 => Sprite::new((96, 16), (16, 16)).with_layer(4),

            EnemyFade1 => Sprite::new((128, 96), (16, 16)).with_layer(4),
            EnemyFade2 => Sprite::new((144, 96), (16, 16)).with_layer(4),
            EnemyFade3 => Sprite::new((160, 96), (16, 16)).with_layer(4),
        }
    }
}
