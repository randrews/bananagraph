use bananagraph::{ GpuWrapper, Sprite };
use std::time::{Duration, Instant};
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::{ElementState, Event, MouseButton, StartCause, WindowEvent};
use winit::event_loop::ControlFlow;
use grid::{xy, Grid, GridMut};
use crate::board::{Board, Cell};

mod board;

pub async fn run_window() -> Result<(), EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop!");

    let window = winit::window::WindowBuilder::new()
        .with_title("The Thing")
        .with_inner_size(LogicalSize { width: 640, height: 480 })
        .with_min_inner_size(LogicalSize { width: 640, height: 480 })
        .build(&event_loop)?;

    let mut wrapper = GpuWrapper::new(&window).await;
    wrapper.add_texture(include_bytes!("iso_dungeon_world.png"), Some("dungeon"));
    let our_id = window.id();

    let timer_length = Duration::from_millis(20);

    // The mouse position is a float, but seems to still describe positions within the same coord
    // space as the window, so just floor()ing it gives you reasonable coordinates
    let mut mouse_pos: (f64, f64) = (-1f64, -1f64);

    // Make a 10x10 board:
    let mut board = Board::new(10, 10);

    event_loop.run(move |event, target| {
        match event {
            // Exit if we click the little x
            Event::WindowEvent { event: WindowEvent::CloseRequested, window_id } if window_id == our_id => {
                target.exit();
            }

            // Redraw if it's redrawing time
            Event::WindowEvent { event: WindowEvent::RedrawRequested, window_id } if window_id == our_id => {
                redraw_window(&wrapper, &board, mouse_pos);
            },

            // Resize if it's resizing time
            Event::WindowEvent { event: WindowEvent::Resized(_), window_id } if window_id == our_id => wrapper.handle_resize(),

            // Start the timer on init
            Event::NewEvents(StartCause::Init) => {
                target.set_control_flow(ControlFlow::WaitUntil(Instant::now() + timer_length));
            }

            // When the timer fires, redraw thw window and restart the timer (update will go here)
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                redraw_window(&wrapper, &board, mouse_pos);
                target.set_control_flow(ControlFlow::WaitUntil(Instant::now() + timer_length));
            }

            // Update that the mouse moved if it did
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position: pos, device_id: _ },
                window_id
            } if window_id == our_id => {
                mouse_pos = (pos.x, pos.y);
            }

            Event::WindowEvent {
                window_id, event: WindowEvent::MouseInput { device_id: _, state: ElementState::Pressed, button: MouseButton::Left }
            } if window_id == our_id => {
                let ids = wrapper.get_sprite_ids().unwrap();
                let id = ids[mouse_pos.into()];
                toggle_wall(id, &mut board)
            }

            _ => {} // toss the others
        }
    })
}

fn toggle_wall(id: u32, board: &mut Board) {
    if id >= 100000 {
        let coord = sprite_id_to_coord(id, board.size().0);
        let cell = board.get(coord.into()).unwrap();
        let (x, y) = coord;

        *board.get_mut(coord.into()).unwrap() = match cell {
            Cell::Black | Cell::White => {
                if x == 0 || y == 0 {
                    Cell::TallWall
                } else {
                    Cell::ShortWall
                }
            },
            Cell::ShortWall | Cell::TallWall => {
                Board::square_color(coord)
            },
            _ => unreachable!()
        }
    }
}

fn redraw_window(wrapper: &GpuWrapper, board: &Board, mouse_pos: (f64, f64)) {
    let mut sprites = board_sprites(board);
    let buffer = wrapper.redraw_ids(&sprites).expect("Drawing error");

    if buffer.contains((mouse_pos.0 as u32, mouse_pos.1 as u32).into()) {
        let id = buffer[mouse_pos.into()];
        if id >= 100000 {
            let board_coord = sprite_id_to_coord(id, board.size().0);
            match board.get(board_coord.into()) {
                Some(Cell::White | Cell::Black) => {
                    let highlight = highlight_sprites(board_coord);
                    sprites.push(highlight.0);
                    sprites.push(highlight.1);
                },
                _ => {}
            }
        }
    }

    wrapper.redraw(&sprites);
}

fn sprite_id_to_coord(id: u32, width: i32) -> (i32, i32) {
    ((id - 100000) as i32 % width, (id - 100000) as i32 / width)
}

fn coord_to_iso(coord: (i32, i32)) -> (f32, f32) {
    let (x, y) = coord;
    let width = 320; // (logical) screen width
    let (dw, dh) = (16, 8); // how much to shift for one increment of "down" vs "lateral". Depends on the exact tile art
    let (basex, basey) = ((width / 2 - dw), 0); // Coord of (0, 0) tile at top center

    // for each x, we go right dw and down dh
    // for each y, we go _left_ dw and down dh
    ((basex + (x - y) * dw) as f32, (basey + (x + y) * dh) as f32)
}

/// Return a z coordinate between 0.0 and 1.0 that will stack these tiles prettily.
/// These will, on a 10x10 grid, range from 0.052 to 0.99, meaning if you add or subtract a
/// ten-thousandth from them then you'll still be in the same "rank" but tweaking the z-order
/// within a given cell.
/// Because this isa ll orthographic projection, we don't really give a damn what the z coordinate
/// _is,_ as long as they sort correctly; we're only using it for occultation, not perspective.
fn coord_to_z(coord: (i32, i32)) -> f32 {
    let (x, y) = coord;

    // The rank of something is its x + y, because isometric view; the lower right of the board
    // is closest to the "camera". Higher z means it's farther away, so we can just divide something
    // by x + y. But! We can't have x + y == 0 obviously, so tweak it a bit, and we also can't have
    // a z of 1.0, because the 0.0..1.0 range is exclusive. So make it a little bit less, 0.99:
    0.99 / ((x + y + 1) as f32)
}

fn board_sprites(board: &Board) -> Vec<Sprite> {
    let mut sprites = Vec::new();

    for (n, cell) in board.iter().enumerate() {
        let (x, y) = board.coord(n).into();
        let tile: Sprite = (*cell).into();
        let screen_pos = coord_to_iso((x, y));
        let tile = tile.with_id((x + y * board.size().0 + 100000) as u32)
            .with_z(coord_to_z((x, y)))
            .with_position(screen_pos, (320.0, 240.0));
        sprites.push(tile);
    }

    let crate_tile = Sprite::new((256, 288), (32, 48));
    let (x, y) = (3, 4);
    let screen_pos = coord_to_iso((x, y));
    let z = coord_to_z((x, y)) - 0.0002; // Move it two epsilon toward the camera, so it's atop the tile itself as well as the (possible) back highlighting

    sprites.push(crate_tile
        .with_z(z)
        .with_position(screen_pos, (320.0, 240.0)));

    sprites
}

fn highlight_sprites(coord: (i32, i32)) -> (Sprite, Sprite) {
    let toph = Sprite::new((416, 0), (32, 48));
    let btmh = Sprite::new((384, 0), (32, 48));
    let screen_pos = coord_to_iso(coord);
    let z = coord_to_z(coord);
    (
        toph.with_position(screen_pos, (320.0, 240.0)).with_z(z - 0.0001),
        btmh.with_position(screen_pos, (320.0, 240.0)).with_z(z - 0.0003)
    )
}

pub fn main() {
    env_logger::init();
    let _ = pollster::block_on(run_window());
}
