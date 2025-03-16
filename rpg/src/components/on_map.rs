use cgmath::Vector2;
use hecs::{Entity, Query, World};

/// We'll give these two convenient names
pub type Loc = Vector2<i32>;

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct OnMap(pub Loc);

/// Gets a list of entities that match a certain query and additionally have a certain map location
pub fn find_at<T: Query>(world: &World, loc: impl Into<Loc>) -> Vec<Entity> {
    let loc = loc.into();
    world.query::<(T, &OnMap)>().iter().filter_map(
        |(e, (_, &om))| {
            if om.0 == loc {
                Some(e)
            } else {
                None
            }
        }
    ).collect()
}

/// Returns whether any entity with a given combination of components exists at the map location
pub fn exists_at<T: Query>(world: &World, loc: impl Into<Loc>) -> bool {
    let loc = loc.into();
    world.query::<(T, &OnMap)>().iter().any(
        |(_, (_, &om))| om.0 == loc
    )
}

#[cfg(test)]
mod tests {
    use crate::components::{Opaque, Solid};
    use super::*;

    fn test_world() -> (Vec<Entity>, World) {
        let mut w = World::new();
        let mut v = vec![];
        v.push(w.spawn((Opaque, Solid, OnMap((2, 3).into()))));
        v.push(w.spawn((Opaque, OnMap((2, 3).into()))));
        v.push(w.spawn((Opaque, Solid, OnMap((1, 1).into()))));
        v.push(w.spawn((Opaque, OnMap((5, 5).into()))));
        v.sort(); // Just in case?
        (v, w)
    }

    #[test]
    fn test_find_at() {
        let (ents, world) = test_world();

        // Finds both opaques
        let mut found = find_at::<(&Opaque,)>(&world, (2, 3));
        found.sort();
        assert_eq!(found, ents[0..2]);

        // Finds only the one solid
        let mut found = find_at::<(&Opaque, &Solid)>(&world, (2, 3));
        found.sort();
        assert_eq!(found, ents[0..1]);
    }

    #[test]
    fn test_exists_at() {
        let (ents, world) = test_world();

        // There are two opaques here
        assert!(exists_at::<&Opaque>(&world, (2, 3)));
        // No solid here
        assert!(!exists_at::<&Solid>(&world, (5, 5)));
        // Nothing at all here
        assert!(!exists_at::<&Opaque>(&world, (3, 3)));
    }
}