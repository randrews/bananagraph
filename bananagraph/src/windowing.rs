use std::ops::Index;
use std::sync::Arc;
use std::time::{Duration, Instant};
use cgmath::{Point2, Vector2};
use winit::application::ApplicationHandler;
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::{ElementState, KeyEvent, MouseButton, StartCause, WindowEvent};
use winit::event_loop::{ActiveEventLoop, ControlFlow};
use winit::keyboard::{Key, NamedKey};
use winit::window::{Window, WindowAttributes, WindowId};
use crate::{GpuWrapper, IdBuffer, SpriteId};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Click {
    pub button: MouseButton,
    pub state: ElementState,
    pub mouse_pos: Point2<f64>,
    pub entity: Option<SpriteId>
}

#[derive(Copy, Clone, PartialEq)]
pub enum Dir { North, South, East, West }

/// A trait for handling game-level events. Bananagraph can keep track of the winit event loop
/// and translate its events into something more game-level semantic. These all have default
/// implementations so you only need to override the ones you care about, but without `redraw` and
/// `init` at minimum, you can't do very much.
pub trait WindowEventHandler {
    /// Run once at the creation of the window; put any one-time init code here, like
    fn init(&mut self, _wrapper: &mut GpuWrapper, _window: Arc<Window>) {}

    /// Run periodically to redraw the window. If this returns Some, then the given `IdBuffer` is used to
    /// handle future click events.
    fn redraw(&self, mouse_pos: Point2<f64>, wrapper: &GpuWrapper) -> Option<IdBuffer>;

    /// Called at about 60 fps, with the actual duration between calls passed
    /// as a parameter.
    fn tick(&mut self, _dt: Duration) {}

    /// Called when the user tries to close the window. The default implementation
    /// returns true, which will terminate the window, but if this returns false then
    /// you can prevent the window being closed (to bring up a confirm dialog?)
    fn exit(&mut self) -> bool { true }

    /// Called every tick after `tick`. If this returns false, then we'll close the application.
    /// Otherwise, we'll redraw.
    fn running(&self) -> bool { true }

    /// Called when the user clicks the mouse somewhere in the window
    fn click(&mut self, _event: Click) {}

    /// Called on every key event in the window. The default implementation parses
    /// pressed events for the arrow keys, printable characters including space, and
    /// the enter and esc keys. If you override this, you can get access to the raw
    /// (from winit, anyway) key events and handle more. But, if you override this,
    /// you'll need to handle calling arrow_key, enter_key, etc yourself if you want
    /// to use those as well.
    fn key(&mut self, event: KeyEvent, _is_synthetic: bool) {
        // We can ignore release events...
        if event.state == ElementState::Released { return }

        match event.logical_key {
            Key::Named(NamedKey::ArrowDown) => self.arrow_key(Dir::South),
            Key::Named(NamedKey::ArrowUp) => self.arrow_key(Dir::North),
            Key::Named(NamedKey::ArrowLeft) => self.arrow_key(Dir::West),
            Key::Named(NamedKey::ArrowRight) => self.arrow_key(Dir::East),
            Key::Named(NamedKey::Enter) => self.enter_key(),
            Key::Named(NamedKey::Escape) => self.esc_key(),
            Key::Character(s) => self.letter_key(s.as_str()),
            Key::Named(NamedKey::Space) => self.letter_key(" "),
            _ => {}
        }
    }

    /// Called when an arrow key is pressed, with which arrow key it was.
    fn arrow_key(&mut self, _dir: Dir) {}

    /// Called when the enter key is pressed
    fn enter_key(&mut self) {}

    /// Called when the esc key is pressed
    fn esc_key(&mut self) {}

    /// Called when any printable key is pressed, with the string of what was typed. This
    /// can include shift chars like @, unicode characters from non-US keyboards, etc.
    fn letter_key(&mut self, _s: &str) {}
}

/// A struct that can impl ApplicationHandler for winit to send it events
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

impl<'a, H: WindowEventHandler> ApplicationHandler for App<'a, H> {
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

        let mut wrapper = pollster::block_on(GpuWrapper::new(window.clone()));
        self.handler.init(&mut wrapper, window.clone());
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
                self.wrapper.as_mut().unwrap().handle_resize(new_size)
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
