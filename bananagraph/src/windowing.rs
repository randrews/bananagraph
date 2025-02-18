use std::ops::Index;
use std::time::{Duration, Instant};
use cgmath::{Point2, Vector2};
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::{ElementState, Event, KeyEvent, MouseButton, StartCause, WindowEvent};
use winit::event_loop::ControlFlow;
use crate::{GpuWrapper, IdBuffer, Sprite, SpriteId};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Click {
    button: MouseButton,
    state: ElementState,
    mouse_pos: Point2<f64>,
    entity: Option<SpriteId>
}

pub trait WindowEventHandler {
    fn init(&mut self, wrapper: &mut GpuWrapper);
    fn redraw(&self) -> Vec<Sprite>;
    fn tick(&mut self, dt: Duration);
    fn exit(&mut self) -> bool;
    fn click(&mut self, event: Click);
    fn key(&mut self, event: KeyEvent, is_synthetic: bool);
}

pub async fn run_window(title: &str, size: Vector2<u32>, min_size: Vector2<u32>, handler: &mut impl WindowEventHandler) -> Result<(), EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop!");

    let window = winit::window::WindowBuilder::new()
        .with_title(title)
        .with_inner_size(LogicalSize { width: size.x, height: size.y })
        .with_min_inner_size(LogicalSize { width: min_size.x, height: min_size.y })
        .build(&event_loop)?;

    let mut wrapper = GpuWrapper::new(&window, size.into()).await;
    handler.init(&mut wrapper);
    let our_id = window.id();
    let mut id_buffer: Option<IdBuffer> = None;

    let timer_length = Duration::from_millis(20);

    // The mouse position is a float, but seems to still describe positions within the same coord
    // space as the window, so just floor()ing it gives you reasonable coordinates
    let mut mouse_pos: Point2<f64> = (-1f64, -1f64).into();

    event_loop.run(move |event, target| {
        match event {
            // Exit if we click the little x
            Event::WindowEvent { event: WindowEvent::CloseRequested, window_id } if window_id == our_id => {
                if handler.exit() {
                    target.exit();
                }
            }

            // Redraw if it's redrawing time
            Event::WindowEvent { event: WindowEvent::RedrawRequested, window_id } if window_id == our_id => {
                id_buffer = Some(wrapper.redraw_with_ids(handler.redraw()).expect("Drawing error"));
            },

            // Resize if it's resizing time
            Event::WindowEvent { event: WindowEvent::Resized(_), window_id } if window_id == our_id => wrapper.handle_resize(),

            // Start the timer on init
            Event::NewEvents(StartCause::Init) => {
                target.set_control_flow(ControlFlow::WaitUntil(Instant::now() + timer_length));
            }

            // When the timer fires, redraw thw window and restart the timer (update will go here)
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                handler.tick(timer_length);
                id_buffer = Some(wrapper.redraw_with_ids(handler.redraw()).expect("Drawing error"));
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
                window_id, event: WindowEvent::MouseInput { device_id: _, state, button }
            } if window_id == our_id => {
                let entity = id_buffer.as_ref().map(|buf| *buf.index(mouse_pos));
                handler.click(Click {
                    button,
                    state,
                    entity,
                    mouse_pos,
                });
            }

            Event::WindowEvent {
                window_id, event: WindowEvent::KeyboardInput { device_id: _, event, is_synthetic }
            } if window_id == our_id => {
                handler.key(event, is_synthetic);
            }

            _ => {} // toss the others
        }
    })
}
