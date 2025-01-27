mod game_state;
mod piece;
mod animation;
mod drawable;
mod matcha_board;

use std::ops::Index;
use bananagraph::{GpuWrapper, IdBuffer};
use std::time::{Duration, Instant};
use cgmath::Point2;
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::{ElementState, Event, MouseButton, StartCause, WindowEvent};
use winit::event_loop::ControlFlow;
use crate::game_state::{ClickTarget, GameState};

pub async fn run_window() -> Result<(), EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop!");
    let size = (1000, 800);

    let window = winit::window::WindowBuilder::new()
        .with_title("Matcha!")
        .with_inner_size(LogicalSize { width: size.0, height: size.1 })
        .with_min_inner_size(LogicalSize { width: size.0, height: size.1 })
        .build(&event_loop)?;

    let mut wrapper = GpuWrapper::new(&window, size).await;
    wrapper.add_texture(include_bytes!("shapes.png"), None);
    let our_id = window.id();
    let mut rng = rand::rng();
    let mut game_state = GameState::new(&mut rng, size);
    let mut id_buffer: Option<IdBuffer> = None;

    let timer_length = Duration::from_millis(20);

    // The mouse position is a float, but seems to still describe positions within the same coord
    // space as the window, so just floor()ing it gives you reasonable coordinates
    let mut mouse_pos: Point2<f64> = (-1f64, -1f64).into();

    event_loop.run(move |event, target| {
        match event {
            // Exit if we click the little x
            Event::WindowEvent { event: WindowEvent::CloseRequested, window_id } if window_id == our_id => {
                target.exit();
            }

            // Redraw if it's redrawing time
            Event::WindowEvent { event: WindowEvent::RedrawRequested, window_id } if window_id == our_id => {
                wrapper.redraw(game_state.redraw());
            },

            // Resize if it's resizing time
            Event::WindowEvent { event: WindowEvent::Resized(_), window_id } if window_id == our_id => wrapper.handle_resize(),

            // Start the timer on init
            Event::NewEvents(StartCause::Init) => {
                target.set_control_flow(ControlFlow::WaitUntil(Instant::now() + timer_length));
            }

            // When the timer fires, redraw thw window and restart the timer (update will go here)
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                game_state.tick(timer_length);
                id_buffer = Some(wrapper.redraw_with_ids(game_state.redraw()).expect("Drawing error"));
                target.set_control_flow(ControlFlow::WaitUntil(Instant::now() + timer_length));
            }

            // Update that the mouse moved if it did
            Event::WindowEvent {
                event: WindowEvent::CursorMoved { position: pos, device_id: _ },
                window_id
            } if window_id == our_id => {
                mouse_pos = (pos.x, pos.y).into();
            }

            Event::WindowEvent {
                window_id, event: WindowEvent::MouseInput { device_id: _, state: ElementState::Pressed, button: MouseButton::Left }
            } if window_id == our_id => {
                game_state.click(match &id_buffer {
                    None => ClickTarget::Location { location: mouse_pos },
                    Some(buf) => {
                        let id = *buf.index(mouse_pos);
                        if id == 0 {
                            ClickTarget::Location { location: mouse_pos }
                        } else {
                            ClickTarget::Sprite { id }
                        }
                    }
                })
            }

            _ => {} // toss the others
        }
    })
}

pub fn main() {
    let _ = pollster::block_on(run_window());
}
