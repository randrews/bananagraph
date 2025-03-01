use std::collections::BTreeSet;
use std::time::Duration;
use cgmath::{Point2, Vector2};
use hecs::{Entity, World};
use rand::Rng;
use bananagraph::{Click, DrawingContext, GpuWrapper, IdBuffer, WindowEventHandler, ElementState};
use grid::{Coord, Grid, VecGrid};
use crate::animation::{animation_system, Animation, Fade, MoveAnimation, Pulse};
use crate::game_state::CaptureSteps::{FadeAnimation, FallAnimation, PieceSelection, SwapAnimation};
use crate::matcha_board::MatchaBoard;
use crate::piece::{Piece, PieceColor};
use crate::piece::PieceColor::Empty;

#[derive(Copy, Clone, Debug, PartialEq)]
pub enum CaptureSteps {
    PieceSelection,
    SwapAnimation,
    FadeAnimation,
    FallAnimation
}

pub struct GameState<'a, R: Rng> {
    world: World,
    rng: &'a mut R,
    screen: (u32, u32),
    selected: Option<Entity>,
    step: CaptureSteps
}

impl<R: Rng> WindowEventHandler for GameState<'_, R> {
    fn init(&mut self, wrapper: &mut GpuWrapper) {
        wrapper.add_texture(include_bytes!("shapes.png"), None);
    }

    fn tick(&mut self, dt: Duration) {
        // Go through all the animation types
        animation_system::<Pulse>(dt, &mut self.world);
        animation_system::<MoveAnimation>(dt, &mut self.world);
        animation_system::<Fade>(dt, &mut self.world);
        // Pulse::system(dt, &mut self.world);
        // MoveAnimation::system(dt, &mut self.world);
        // Fade::system(dt, &mut self.world);

        // Only way this happens is the first tick after the move animations finish, so,
        // it's time to actually swap the pieces and do captures:
        if !self.animation_blocked() {
            match self.step {
                PieceSelection => {} // Do nothing
                SwapAnimation => {
                    self.fade_pieces();
                    self.step = FadeAnimation;
                }
                FadeAnimation => {
                    // Fade animations have now finished, clear the pieces and perform fall animations
                    self.fall_pieces();
                    self.step = FallAnimation;
                }
                FallAnimation => {
                    // Fall animations have now finished, see if there are more matches:
                    self.step = if self.any_matches() {
                        self.fade_pieces();
                        FadeAnimation
                    } else {
                        PieceSelection
                    };
                }
            }
        }
    }

    fn redraw(&self, _mouse_pos: Point2<f64>, wrapper: &GpuWrapper) -> Option<IdBuffer> {
        let mut sprites = vec![];
        let dc = DrawingContext::new((self.screen.0 as f32, self.screen.1 as f32));

        let mut query = self.world.query::<(&Piece, Option<&Pulse>, Option<&MoveAnimation>, Option<&Fade>)>();
        for (ent, (piece, pulse, move_anim, fade)) in query.into_iter() {
            // hecs will give us 0 as a sprite id, but bananagraph can't abide that, so, add something to it to
            // ensure we can hear clicks on the sprite
            let mut drawable = piece.as_drawable(ent.id() + 1000, self.screen);

            if let Some(p) = pulse { drawable = p.apply_to(drawable) };
            if let Some(m) = move_anim { drawable = m.apply_to(drawable) };
            if let Some(f) = fade { drawable = f.apply_to(drawable) };

            sprites.push(drawable.as_sprite(dc))
        }

        wrapper.redraw_with_ids(sprites).ok()
    }

    fn click(&mut self, click: Click) {
        if click.state == ElementState::Released { return }

        if let Some(id) = click.entity {
            // If we're waiting on animations,
            if self.step != PieceSelection || self.animation_blocked() { return }

            // This is only ever clicked sprite ids, which are always an entity id + 1000: hecs will give 0 as
            // entity ids, which bananagraph interprets as an empty sprite id
            let ent = unsafe {
                self.world.find_entity_from_id(id - 1000)
            };

            if let Some(selected) = self.selected {
                let new_piece = *self.world.get::<&Piece>(ent).unwrap();
                let selected_piece = *self.world.get::<&Piece>(selected).unwrap();

                // Valid move?
                if self.valid_move(selected_piece.position, new_piece.position) || self.valid_move(new_piece.position, selected_piece.position) {
                    // Create and attach animations
                    let (anim1, anim2) = Piece::swap_animations(selected_piece, new_piece);
                    self.world.insert_one(selected, anim2).unwrap();
                    self.world.insert_one(ent, anim1).unwrap();

                    // Actually swap the pieces
                    let pos_selected = selected_piece.position;
                    let pos_new = new_piece.position;
                    self.world.get::<&mut Piece>(selected).unwrap().position = pos_new;
                    self.world.get::<&mut Piece>(ent).unwrap().position = pos_selected;
                    self.selected = None;

                    // Increment the step
                    self.step = SwapAnimation
                } else {
                    // Invalid, clear the selection
                    self.selected = None;
                }
                // Either way stop pulsing
                self.world.remove_one::<Pulse>(selected).unwrap();
            } else {
                self.selected = Some(ent);
                self.world.insert_one(ent, Pulse::new()).unwrap()
            }
        }
    }
}

impl<'a, R: Rng> GameState<'a, R> {
    pub fn new(rng: &'a mut R, screen: (u32, u32)) -> Self {
        let mut world = World::new();

        let board = initialize_board(rng);

        for (n, color) in board.iter().enumerate() {
            let c = board.coord(n);
            world.spawn((Piece::new(*color, c),));
        }

        Self {
            world,
            rng,
            screen,
            selected: None,
            step: PieceSelection
        }
    }

    pub fn any_matches(&mut self) -> bool {
        let board = self.board_from_world();
        board.find_match().is_some()
    }

    /// Apply a fade animation to all captured pieces
    pub fn fade_pieces(&mut self) {
        let captured = self.all_captured();
        let entity_board = self.entity_grid_from_world();

        for pt in captured.into_iter() {
            let c = Vector2::from((pt.x, pt.y));
            self.world.insert_one(entity_board[c], Fade::new()).unwrap()
        }
    }

    pub fn fall_pieces(&mut self) {
        let captured = self.all_captured();
        let mut board = self.board_from_world();
        let entity_grid = self.entity_grid_from_world();

        // First, clear out everything that was captured:
        for pt in captured.into_iter() {
            let c = Vector2::from((pt.x, pt.y));
            board[c] = Empty;
            self.world.despawn(entity_grid[c]).unwrap()
        }

        // Make a grid with how far each piece needs to fall:
        let mut falls = VecGrid::new(Grid::size(&board), 0);
        let height = falls.size().y;
        for c in falls.size().iter() {
            if board[c] == Empty { continue } // don't bother counting the empties
            let mut n = 0;
            for y in c.y .. height {
                if board[(c.x, y)] == Empty { n += 1 }
            }

            falls[c] = n;
        }

        // Now, apply move animations:
        // We'll go ahead and logically move the pieces now, and have them "fall" from an artificially higher place
        for c in falls.size().iter() {
            if falls[c] != 0 {
                let anim = MoveAnimation::new((0, -falls[c] * 85));
                self.world.insert_one(entity_grid[c], anim).unwrap();
                let piece = self.world.query_one_mut::<&mut Piece>(entity_grid[c]).unwrap();
                piece.position.y += falls[c];
            }
        }

        // Now that everything has logically fallen, we'll recreate the entity board, which will now have empty cells on
        // tops of columns:
        let entity_grid = self.entity_grid_from_world();
        // A vec of the number of empty cells at the top of each column
        let empty_heights: Vec<_> = (0..entity_grid.size().x).map(|x| {
            let mut n = 0;
            for y in 0..height {
                if entity_grid[(x, y)] == Entity::DANGLING { n += 1 }
            }
            n
        }).collect();
        for c in entity_grid.size().iter() {
            if entity_grid[c] == Entity::DANGLING {
                // We need to create a new thing here!
                let new_piece = Piece::new(PieceColor::from_rand(self.rng), c);
                let anim = MoveAnimation::new((0, -empty_heights[c.x as usize] * 85));
                self.world.spawn((new_piece, anim));
            }
        }
    }

    /// Find the coords of all the captured pieces:
    fn all_captured(&self) -> Vec<Vector2<i32>> {
        let board = self.board_from_world();

        let mut captured = BTreeSet::new();
        for (n, _color) in board.iter().enumerate() {
            let c = board.coord(n);
            if let Some(pieces) = board.is_match(c) {
                for c in pieces.into_iter() {
                    captured.insert(board.nth(c).unwrap());
                }
            }
        }

        captured.into_iter().map(|n| {
            let c = board.coord(n);
            (c.x, c.y).into()
        }).collect()
    }

    fn board_from_world(&self) -> VecGrid<PieceColor> {
        let mut board = VecGrid::new((8, 8), Empty);
        for (_ent, piece) in self.world.query::<&Piece>().into_iter() {
            board[piece.position] = piece.color;
        }
        board
    }

    fn entity_grid_from_world(&self) -> VecGrid<Entity> {
        let mut grid = VecGrid::new((8, 8), Entity::DANGLING);
        for (ent, piece) in self.world.query::<&Piece>().into_iter() {
            grid[piece.position] = ent;
        }
        grid
    }

    fn valid_move(&self, p1: impl Into<Vector2<i32>>, p2: impl Into<Vector2<i32>>) -> bool {
        let (p1, p2) = (p1.into(), p2.into());
        let board = self.board_from_world();
        board.valid_move((p1.x, p1.y), (p2.x, p2.y))
    }

    pub fn animation_blocked(&self) -> bool {
        self.world.query::<&MoveAnimation>().into_iter().next().is_some() ||
            self.world.query::<&Fade>().into_iter().next().is_some()
    }
}

fn initialize_board<R: Rng + Sized>(rng: &mut R) -> VecGrid<PieceColor> {
    let mut board = VecGrid::new((8, 8), Empty);
    loop {
        // board is a temporary vecgrid of just piece colors, until we can create a valid
        // field, then we'll reify it into entities
        for coord in Grid::size(&board).iter() {
            board[coord] = PieceColor::from_rand(rng)
        }

        // Clear out all the matches:
        while let Some(coords) = board.find_match() {
            board.scramble_match(coords, rng);
        }

        if board.has_move() { break }
    }

    board
}
