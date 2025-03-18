use hecs::World;
use toml::Table;
use crate::map_file::LoadMapError::FormatError;

pub enum LoadMapError {
    FormatError(String)
}

enum MapChars {
    Wall,
    Door,
    Clear,
    Common(char), // A lowercase letter, an instance of an archetype
    Unique(char), // Uppercase letter, a unique entity
}

pub fn load_toml(world: &mut World, table: Table) -> Result<(), LoadMapError> {
    let map_section = table.get("map").ok_or(FormatError(String::from("Map section not found")))?;
    let layer1 = map_section.get("layer1").ok_or(FormatError(String::from("layer1 not found")))?;

    // Ensure map is a rectangle
    // Make a grid and assign an enum for each cell
    // Parse the archetypes
    // Validate all the commons and uniques actually exist
    // Validate uniques are unique
    // Create components for terrain
    // Create components for uniques
    // Create components for commons
    todo!()
}

#[cfg(test)]
mod tests {
    use toml::Value::String;
    use super::*;

    #[test]
    fn test_walls() {
        let table = include_str!("test_maps/test_map.toml").parse::<Table>().unwrap();
    }
}