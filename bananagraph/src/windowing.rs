use std::ops::Index;
use std::sync::Arc;
use std::time::{Duration, Instant};
use cgmath::{Point2, Vector2};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::{StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::window::{Window, WindowAttributes, WindowId};
use crate::{GpuWrapper, IdBuffer};
use crate::event_handler::{Click, ElementState, MouseButton, WindowEventHandler};

/// A struct that can impl ApplicationHandler for winit to send it events
#[cfg(not(target_arch = "wasm32"))]
struct App<'a, H> {
    /// The window can't be owned by App because it owns the GpuWrapper, which borrows the window (surface).
    /// So we store it in an Arc
    window: Option<Arc<Window>>,

    /// Neither the wrapper nor the window can be assumed to exist; we can't create them until the first Resumed event.
    /// So they're Options which start as None
    wrapper: Option<GpuWrapper<'a>>,

    /// The `WindowEventHandler` that will be sent game-logic-level events
    handler: H,

    /// Attributes to create the window with, needed until we create the window in `resumed`
    attrs: WindowAttributes,

    /// The event loop will tick at this frequency, calling `handler.tick` when this timer runs down
    timer_length: Duration,

    /// There's no built-in facility for tracking the mouse position, so we'll just store it and update it
    /// on mouse moved events
    mouse_pos: Point2<f64>,

    /// The id buffer created by bananagraph's render process
    id_buffer: Option<IdBuffer>
}

#[cfg(not(target_arch = "wasm32"))]
impl<H: WindowEventHandler> ApplicationHandler for App<'_, H> {
    // When the timer fires, redraw thw window and restart the timer (update will go here)
    fn new_events(&mut self, event_loop: &ActiveEventLoop, cause: StartCause) {
        if let StartCause::ResumeTimeReached { .. } = cause {
            self.handler.tick(self.timer_length);
            if self.handler.running() {
                self.id_buffer = self.handler.redraw(self.mouse_pos, self.wrapper.as_ref().unwrap());
                event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + self.timer_length));
            } else {
                event_loop.exit()
            }
        }
    }

    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let window = event_loop.create_window(self.attrs.clone()).unwrap();
        let window = Arc::new(window);
        self.window = Some(window.clone());
        let physical_size = window.inner_size();
        let physical_size = Vector2::from((physical_size.width, physical_size.height));
        let logical_size = window.inner_size().to_logical(window.scale_factor());
        let logical_size = Vector2::from((logical_size.width, logical_size.height));
        let mut wrapper = pollster::block_on(GpuWrapper::targeting(window.clone(), physical_size, logical_size));
        self.handler.init(&mut wrapper);
        self.wrapper = Some(wrapper);
        event_loop.set_control_flow(ControlFlow::WaitUntil(Instant::now() + self.timer_length))
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, _our_id: WindowId, event: WindowEvent) {
        match event {
            // Exit if we click the little x
            WindowEvent::CloseRequested => {
                if self.handler.exit() {
                    event_loop.exit()
                }
            },

            // Redraw if it's redrawing time
            WindowEvent::RedrawRequested => {
                self.id_buffer = self.handler.redraw(self.mouse_pos, self.wrapper.as_ref().unwrap());
            },

            // Resize if it's resizing time
            WindowEvent::Resized(new_size)  => {
                self.wrapper.as_mut().unwrap().handle_resize((new_size.width, new_size.height).into())
            }

            // Update that the mouse moved if it did
            WindowEvent::CursorMoved { position: pos, device_id: _ } => {
                self.mouse_pos = (pos.x, pos.y).into();
            }

            // Mouse clicked
            WindowEvent::MouseInput { device_id: _, state, button } => {
                // Handle the buffer not being there, and 0 isn't a valid sprite id; 0 means there's
                // no entity there / the sprite is unclickable.
                let entity = match &self.id_buffer {
                    None => None,
                    Some(buf) => {
                        let id = *buf.index(self.mouse_pos);
                        if id == 0 {
                            None
                        } else {
                            Some(id)
                        }
                    }
                };

                let state = match state {
                    winit::event::ElementState::Pressed => ElementState::Pressed,
                    winit::event::ElementState::Released => ElementState::Released
                };

                let button = match button {
                    winit::event::MouseButton::Left => MouseButton::Left,
                    winit::event::MouseButton::Right => MouseButton::Right,
                    _ => MouseButton::Left // TODO handle other buttons
                };

                self.handler.click(Click {
                    button,
                    state,
                    entity,
                    mouse_pos: self.mouse_pos,
                });
            }

            // Key pressed or released
            WindowEvent::KeyboardInput { device_id: _, event, is_synthetic } => {
                self.handler.key(event, is_synthetic);
            }

            _ => {} // toss the others
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub async fn run_window(title: &str, initial_size: Vector2<u32>, min_size: Vector2<u32>, handler: impl WindowEventHandler) -> Result<(), EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop!");
    event_loop.set_control_flow(ControlFlow::Wait);

    let attrs = Window::default_attributes()
        .with_title(title)
        .with_inner_size(LogicalSize { width: initial_size.x, height: initial_size.y })
        .with_min_inner_size(LogicalSize { width: min_size.x, height: min_size.y });

    let mut app = App {
        window: None,
        wrapper: None,
        id_buffer: None,
        handler,
        attrs,
        mouse_pos: (-1f64, -1f64).into(),
        timer_length: Duration::from_millis(20)
    };
    event_loop.run_app(&mut app)
}
