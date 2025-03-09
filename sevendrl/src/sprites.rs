use cgmath::Vector2;
use hecs::{Entity, World};
use bananagraph::{DrawingContext, Sprite};
use crate::animation::OneShotAnimation;
use crate::components::OnMap;
use crate::enemy::{Enemy, EnemyType};
use crate::sprites::AnimationSprites::{Mimic1, MimicFade1, Shove1};
use crate::sprites::Items::Crystal;
use crate::terrain::Opaque;

pub trait SpriteFor {
    fn sprite(&self) -> Sprite;
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AnimationSprites {
    Player1, // Player animation frames
    Player2,
    Player3,
    Enemy1, // Enemy breathe frames
    Enemy2,
    Enemy3,
    Mimic1, // Mimic breathe frames
    Mimic2,
    Mimic3,
    EnemyFade1, // Enemy fade animation frames
    EnemyFade2,
    EnemyFade3,
    MimicFade1, // Mimic fade animation frames
    MimicFade2,
    MimicFade3,
    Shove1, // Shove ability effect animation
    Shove2,
    Shove3,
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

    pub fn mimic_breathe() -> Vec<Sprite> {
        use AnimationSprites::*;
        [
            Mimic1,
            Mimic2,
            Mimic3,
            Mimic2,
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

    pub fn mimic_fade() -> Vec<Sprite> {
        use AnimationSprites::*;
        [
            MimicFade1, MimicFade2, MimicFade3
        ].map(|a| a.sprite()).into_iter().collect()
    }

    pub fn enemy_fade_at(world: &mut World, enemy: Entity, at: impl Into<Vector2<i32>>) {
        let anim = match world.query_one::<&Enemy>(enemy).unwrap().get().unwrap().enemy_type {
            EnemyType::Normal => OneShotAnimation::new(Self::enemy_fade()),
            EnemyType::Mimic => OneShotAnimation::new(Self::mimic_fade()),
        };

        world.spawn((
            OnMap { location: at.into(), sprite: AnimationSprites::EnemyFade1.sprite() },
            anim,
            Opaque
            ));
    }

    pub fn shove_at(world: &mut World, at: impl Into<Vector2<i32>>) {
        let at = at.into();
        let anim = OneShotAnimation::new(Self::shove());
        world.spawn((
            anim,
            OnMap { location: at, sprite: Self::Shove1.sprite() }
        ));
    }

    pub fn shove() -> Vec<Sprite> {
        use AnimationSprites::*;
        [
            Shove1,
            Shove2,
            Shove3
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

            Mimic1 => Sprite::new((128, 16), (16, 16)).with_layer(4),
            Mimic2 => Sprite::new((144, 16), (16, 16)).with_layer(4),
            Mimic3 => Sprite::new((160, 16), (16, 16)).with_layer(4),

            EnemyFade1 => Sprite::new((128, 96), (16, 16)).with_layer(4),
            EnemyFade2 => Sprite::new((144, 96), (16, 16)).with_layer(4),
            EnemyFade3 => Sprite::new((160, 96), (16, 16)).with_layer(4),

            MimicFade1 => Sprite::new((128, 128), (16, 16)).with_layer(4),
            MimicFade2 => Sprite::new((144, 128), (16, 16)).with_layer(4),
            MimicFade3 => Sprite::new((160, 128), (16, 16)).with_layer(4),

            Shove1 => Sprite::new((128, 112), (16, 16)).with_layer(4),
            Shove2 => Sprite::new((144, 112), (16, 16)).with_layer(4),
            Shove3 => Sprite::new((160, 112), (16, 16)).with_layer(4),
        }
    }
}

#[derive(Debug, Copy, Clone, PartialEq)]
pub enum UiFrame {
    NwCorner,
    NeCorner,
    SeCorner,
    SwCorner,
    NEdge,
    EEdge,
    SEdge,
    WEdge,
    Middle
}

impl SpriteFor for UiFrame {
    fn sprite(&self) -> Sprite {
        use UiFrame::*;
        match self {
            NwCorner => Sprite::new((54, 134), (16, 16)).with_z(0.9).with_layer(2),
            NeCorner => Sprite::new((90, 134), (16, 16)).with_z(0.9).with_layer(2),
            SeCorner => Sprite::new((54, 171), (16, 16)).with_z(0.9).with_layer(2),
            SwCorner => Sprite::new((90, 171), (16, 16)).with_z(0.9).with_layer(2),
            NEdge => Sprite::new((70, 134), (16, 16)).with_z(0.9).with_layer(2),
            EEdge => Sprite::new((90, 150), (16, 16)).with_z(0.9).with_layer(2),
            SEdge => Sprite::new((74, 171), (16, 16)).with_z(0.9).with_layer(2),
            WEdge => Sprite::new((54, 150), (16, 16)).with_z(0.9).with_layer(2),
            Middle => Sprite::new((16, 144), (16, 16)).with_z(0.9).with_layer(2)
        }
    }
}

impl UiFrame {
    pub fn draw_frame(dc: DrawingContext, topleft: impl Into<Vector2<f32>>, tile_size: impl Into<Vector2<i32>>, z: f32) -> Vec<Sprite> {
        let (topleft, size) = (topleft.into(), tile_size.into());
        use UiFrame::*;
        let mut sprites = vec![];

        for y in 0..size.y {
            for x in 0..size.x {
                let spr = if (x, y) == (0, 0) { NwCorner }
                else if (x, y) == (size.x - 1, 0) { NeCorner }
                else if (x, y) == (0, size.y - 1) { SeCorner }
                else if (x, y) == (size.x - 1, size.y - 1) { SwCorner }
                else if y == 0 { NEdge }
                else if x == size.x - 1 { EEdge }
                else if y == size.y - 1 { SEdge }
                else if x == 0 { WEdge }
                else { Middle };
                sprites.push(dc.place(spr.sprite().with_z(z), Vector2::new(x as f32, y as f32) * 16.0 + topleft));
            }
        }

        sprites
    }
}

pub enum Items {
    HealthPotion,
    EnergyPotion,
    Scroll1,
    Scroll2,
    Scroll3,
    Scroll4,
    Chest,
    Crystal,
    Mushroom
}

impl SpriteFor for Items {
    fn sprite(&self) -> Sprite {
        use Items::*;
        match self {
            HealthPotion => Sprite::new((0, 0), (16, 16)).with_layer(5),
            EnergyPotion => Sprite::new((32, 0), (16, 16)).with_layer(5),
            Scroll1 => Sprite::new((0, 112), (16, 16)).with_layer(5),
            Scroll2 => Sprite::new((48, 112), (16, 16)).with_layer(5),
            Scroll3 => Sprite::new((64, 112), (16, 16)).with_layer(5),
            Scroll4 => Sprite::new((128, 112), (16, 16)).with_layer(5),
            Chest => Sprite::new((64, 128), (16, 16)).with_z(0.7),
            Crystal => Sprite::new((32, 160), (16, 16)).with_layer(5).with_z(0.7),
            Mushroom => Sprite::new((48, 128), (16, 16)).with_layer(5).with_z(0.7),
        }
    }
}

pub enum MapCells {
    Fog,
}

impl SpriteFor for MapCells {
    fn sprite(&self) -> Sprite {
        use MapCells::*;
        match self {
            Fog => Sprite::new((80, 64), (16, 16)),
        }
    }
}