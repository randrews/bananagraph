use hecs::World;
use thiserror::Error;
use toml::Table;
use bananagraph::Sprite;
use grid::{Grid, VecGrid, Coord};
use crate::components::{Door, OnMap, Opaque, Solid, Visible};
use crate::map_file::LoadMapError::{MissingKey, NotRectangular, UnrecognizedChar};
use crate::sprites::SpriteType;

#[derive(Error, Debug, Clone)]
pub enum LoadMapError {
    #[error("Missing / mistyped toml key: {0}")]
    MissingKey(String),

    #[error("map.cells is not rectangular")]
    NotRectangular,

    #[error("Unrecognized map character '{0}'")]
    UnrecognizedChar(char),
}

#[derive(Copy, Clone, Debug, PartialEq)]
enum MapChars {
    Wall,
    Door,
    Clear,
    Common(char), // A lowercase letter, an instance of an archetype
    Unique(char), // Uppercase letter, a unique entity
}

pub fn load_toml(world: &mut World, table: Table) -> Result<(), LoadMapError> {
    let map_section = table.get("map").ok_or(MissingKey(String::from("map")))?;
    let cells = map_section.get("cells").ok_or(MissingKey(String::from("map.cells")))?;

    // Ensure map is a rectangle
    let cells = cells.as_str().ok_or(MissingKey(String::from("map.cells")))?;
    let h = cells.lines().count();
    if h == 0 { return Err(NotRectangular) }
    let mut w = None;
    for line in cells.lines() {
        let line = line.trim();
        if line.len() == 0 { continue } // Might be a trailing blank line
        if let Some(ow) = w {
            if ow != line.len() {
                return Err(NotRectangular)
            }
        } else {
            w = Some(line.len()) // Using len is fine here because it's pure ascii fun times zone
        }
    }

    // Make a grid and assign an enum for each cell
    let mut grid = VecGrid::new((w.unwrap() as i32, h as i32), MapChars::Clear);
    for (y, line) in cells.lines().enumerate() {
        for (x, ch) in line.trim().chars().enumerate() {
            grid[(x as i32, y as i32)] = match ch {
                '.' => Ok(MapChars::Clear),
                '#' => Ok(MapChars::Wall),
                '+' => Ok(MapChars::Door),
                'a' .. 'z' => Ok(MapChars::Common(ch)),
                'A' .. 'Z' => Ok(MapChars::Unique(ch)),
                _ => Err(UnrecognizedChar(ch))
            }?;
        }
    }

    // A word regarding Z levels. This is where all Z values for static elements are placed.
    // wgpu gives us Z in a range from 0.0 (highest) to 1.0 (lowest) exclusive. Z values entirely
    // determine occlusion and must be set for the things to draw correctly (transparency is, as
    // always, respected).
    // We're going to standardize on the following Z ranges:
    // The range will be divided into 10 sub-ranges, with [0.9-1.0) being the lowest:
    // - 0.99: floors
    // - 0.9: items sitting on the floor
    // - 0.8: walls and doors
    // - 0.7: the player

    // Parse the archetypes TODO
    // Validate all the commons and uniques actually exist TODO
    // Validate uniques are unique TODO

    // Create components for terrain
    for c in grid.size().iter() {
        let om = OnMap(c);

        match grid[c] {
            MapChars::Wall => {
                let ns = grid.for_neighbors(c, |_, mc| matches!(mc, MapChars::Wall | MapChars::Door));
                let vis = Visible(<SpriteType as Into<Sprite>>::into(SpriteType::Wall(ns)).with_z(0.8));
                world.spawn((om, Solid, Opaque, vis));
            }

            MapChars::Door => {
                world.spawn((
                    om,
                    Solid,
                    Opaque,
                    Visible(<SpriteType as Into<Sprite>>::into(SpriteType::Door(false)).with_z(0.8)),
                    Door::closed()));
            }

            MapChars::Clear => {
                let variant = (c.x + c.y) as u32 % 2;
                let vis = Visible(<SpriteType as Into<Sprite>>::into(SpriteType::Floor(variant)).with_z(0.99));
                world.spawn((om, vis));
            }

            MapChars::Common(_) => {
                // Implied floor tile underneath this:
                let variant = (c.x + c.y) as u32 % 2;
                let vis = Visible(<SpriteType as Into<Sprite>>::into(SpriteType::Floor(variant)).with_z(0.99));
                world.spawn((om, vis));
            }
            MapChars::Unique(_) => {
                // Implied floor tile underneath this:
                let variant = (c.x + c.y) as u32 % 2;
                let vis = Visible(<SpriteType as Into<Sprite>>::into(SpriteType::Floor(variant)).with_z(0.99));
                world.spawn((om, vis));
            }
        };
    }

    // Create components for uniques TODO
    // Create components for commons TODO

    Ok(())
}

#[cfg(test)]
mod tests {
    use crate::components::exists_at;
    use super::*;

    const test_map: &str = r#"
    [map]
    cells = """
        ################
        #l.b.#.#l.Dcc..#
        #....#.+.....A.#
        #C...+.#.......#
        ######.#########
        #l...p.p......l#
        #..............#
        ######+#########
    """
    "#;

    #[test]
    fn test_walls() {
        let table = test_map.parse::<Table>().unwrap();
        let mut world = World::new();
        load_toml(&mut world, table).unwrap();

        assert_eq!(exists_at::<(&Solid, &Opaque)>(&world, (5, 2)), true);
        assert_eq!(exists_at::<(&Solid,)>(&world, (3, 2)), false); // Clear cell
        assert_eq!(exists_at::<&Door>(&world, (5, 3)), true); // Door
    }
}