use bananagraph::Sprite;

/// Anything that is displayed on the screen. _Where_ it's displayed is the
/// job of combinations of components; OnMap + Visible is a map cell, Inventory + Visible, etc.
#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Visible(pub Sprite);