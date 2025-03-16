// "Traits" go here, which are very simple (often zero-sized) components.

/// A "Solid" entity can't be moved through; an attempt to move through a
/// solid entity generates a bump event instead.
#[derive(Copy, Clone, Debug)]
pub struct Solid;

/// An "Opaque" entity blocks line of sight
#[derive(Copy, Clone, Debug)]
pub struct Opaque;

/// A trait that makes an entity not animate, if it otherwise has an animation
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Frozen;

/// If any entities with this trait exist, we shouldn't let the player take more turns
/// (think, one-shot animations)
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Blocking;
