use cgmath::Vector2;
use bananagraph::{DrawingContext, Sprite};
use grid::{xy, Coord, Grid};

type PixelDimension = Vector2<u32>;

pub trait AsSprite {
    fn as_sprite(&self) -> Sprite;
}

/// An isometrically-displayed grid, made up of sprites.
/// ```text
///              sprite width
///          v-------------------v
///        > +-------------------+
///        | |                   |
///        | |                   |
///        | |                   |
///        | |                   |
///        | |                   |
///        | |         X         | <
///        | |        / \        | |
/// sprite | |       /   \       | |
/// height | |      /     \      | |
///        | |     /       \     | |
///        | |    /         \    | | base
///        | |   |           |   | | height
///        | |    \         /    | |
///        | |     \       /     | |
///        | |      \     /      | |
///        | |       \   /       | |
///        | |        \ /        | |
///        > +---------V---------+ <
///              ^-----------^
///                base width
/// ```
pub struct IsoMap<'a, T: AsSprite, G: Grid<CellType=T>> {
    /// The size of the map in cells
    grid: &'a G,

    /// The size of each sprite in pixels
    sprite_size: PixelDimension,

    /// The portion of each sprite that represents the base of the cell
    base_size: PixelDimension,
}

#[allow(unused)]
impl<'a, T: AsSprite, G: Grid<CellType=T>> IsoMap<'a, T, G> {
    pub fn new(grid: &'a G, sprite_size: impl Into<PixelDimension>, base_size: impl Into<PixelDimension>) -> Self {
        let sprite_size = sprite_size.into();
        let base_size = base_size.into();
        Self { grid, sprite_size, base_size }
    }

    /// Return the pixel dimensions of the grid when displayed isometrically.
    pub fn dimensions(&self) -> PixelDimension {
        /* We measure from a "start point" that represents the midpoint of the (0, 0) cell:

                        start point
                             |
                     v-------v-----------v
                       map   X  map width
                      height/ \
                           X   X
                       /  / \ / \
                      /  X   |   X  \
                     v  / \ / \ / \  \
                +y dir X   |   |   X  v
                      / \ /     \ / \  +x dir
                     |   X       |   X
                      \ / \       \ / \
                       |   X       |   X
                        \ / \       \ / \
                         |   X       X   |
                          \ / \     / \ /
                           |   X   X   |
                            \ / \ / \ /
                             |   X   |
                              \ / \ /
                               |   |
                                \ /
                                 V

           The map extends to the right 1/2 base width for each cell in the width, and the same
           leftward for each cell in the height.

           The height of the map extends downward for 1/2 base height each row, which means the
           sum of the number of rows and columns in the grid.

           In addition to this, each cell has a margin; the base is centered in the bottom of the
           sprite. Normally these overlap (that's the point of the margins) but they don't overlap
           on the edges of the map: the top margin is added to the height for the (0, 0) cell and
           the left and right margins are added to the width.
         */

        let (grid_width, grid_height) = self.grid.size().into();
        let margin_width = self.sprite_size.x - self.base_size.x;
        let margin_height = self.sprite_size.y - self.base_size.y;

        let base_width = self.base_size.x * (grid_width + grid_height) as u32 / 2;
        let total_width = base_width + margin_width;

        let base_height = (grid_width + grid_height) as u32 * self.base_size.y / 2;
        let total_height = base_height + margin_height;

        (total_width, total_height).into()
    }

    /// Return the pixel coordinates of the top left corner of a given cell's location
    pub fn cell_location(&self, coord: impl Into<Coord>) -> PixelDimension {
        let coord = coord.into();

        // See the diagram in #dimensions. We start at the start point and:
        // - for each unit in x, move 1/2 base width right and 1/2 base height down
        // - for each unit in y, move 1/2 base width left and 1/2 base height down
        // The start point can be calculated by starting at the left edge and moving
        // 1/2 base width right for each row but the first.

        let (grid_width, grid_height) = self.grid.size().into();
        let start_x = self.base_size.x / 2 * (grid_height as u32 - 1);

        let x = (coord.0 - coord.1) * self.base_size.x as i32 / 2 + start_x as i32;
        let y = (coord.0 + coord.1) as u32 * self.base_size.y / 2;

        (x as u32, y).into()
    }

    /// Return which "row" a cell is in:
    /// ```text
    ///         X  (0, 0) is in row 0
    ///        / \
    ///       X   X   (1, 0) and (0, 1)
    ///      / \ / \      in row 1
    ///     X   X   X
    ///    / \ / \ / \  ...and so on...
    ///   X   |   |   X
    ///  / \ / \ / \ / \
    /// |   |   |   |   |
    ///  \ / \ / \ / \ /
    ///   V   V   V   V
    /// ```
    pub fn cell_row(&self, coord: impl Into<Coord>) -> u32 {
        let coord = coord.into();
        (coord.0 + coord.1) as u32
    }

    /// Return a z coordinate between 0.0 and 1.0 that will stack these tiles prettily.
    /// These will, on a 10x10 grid, range from 0.052 to 0.99, meaning if you add or subtract a
    /// ten-thousandth from them then you'll still be in the same "rank" but tweaking the z-order
    /// within a given cell.
    /// Because this is all orthographic projection, we don't really give a damn what the z coordinate
    /// _is,_ as long as they sort correctly; we're only using it for occultation, not perspective.
    /// TODO: this needs to work for non-10x10 grids
    pub fn z_coord(&self, coord: Coord) -> f32 {
        let (x, y) = coord.into();

        // The rank of something is its x + y, because isometric view; the lower right of the board
        // is closest to the "camera". Higher z means it's farther away, so we can just divide something
        // by x + y. But! We can't have x + y == 0 obviously, so tweak it a bit, and we also can't have
        // a z of 1.0, because the 0.0..1.0 range is exclusive. So make it a little bit less, 0.99:
        0.99 / ((x + y + 1) as f32)
    }

    pub fn sprites(&self, dc: DrawingContext) -> Vec<Sprite> {
        let mut sprites = vec![];

        for (n, cell) in self.grid.iter().enumerate() {
            let coord = self.grid.coord(n);
            let (x, y) = coord.into();
            let sprite: Sprite = cell.as_sprite()
                .with_id(self.id_for(coord))
                .with_z(self.z_coord(coord));

            let translation = self.cell_location(coord);
            let tile = dc.place(sprite, (translation.x as f32, translation.y as f32));
            sprites.push(tile);
        }

        sprites
    }

    pub fn sprite(&self, sprite: Sprite, coord: Coord, dc: &DrawingContext) -> Sprite {
        let translation = self.cell_location(coord);
        dc.place(sprite, (translation.x as f32, translation.y as f32))
    }

    pub fn id_for(&self, coord: Coord) -> u32 {
        let (x, y) = coord.into();
        (x + y * self.grid.size().0 + 100000) as u32
    }

    pub fn coord_for(&self, id: u32) -> Option<Coord> {
        if id > 100000 {
            let width = self.grid.size().0;
            Some(xy((id - 100000) as i32 % width, (id - 100000) as i32 / width))
        } else {
            None
        }
    }
}