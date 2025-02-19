use std::ops::Index;
use std::time::{Duration, Instant};
use cgmath::{Point2, Vector2};
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::{ElementState, Event, KeyEvent, MouseButton, StartCause, WindowEvent};
use winit::event_loop::ControlFlow;
use winit::keyboard::{Key, NamedKey};
use crate::{GpuWrapper, IdBuffer, Sprite, SpriteId};

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Click {
    button: MouseButton,
    state: ElementState,
    mouse_pos: Point2<f64>,
    entity: Option<SpriteId>
}

#[derive(Copy, Clone, PartialEq)]
pub enum Dir { North, South, East, West }

pub trait WindowEventHandler {
    /// Run once at the creation of the window; put any one-time init code here
    fn init(&mut self, _wrapper: &mut GpuWrapper) {}

    /// Run periodically to create the list of sprites to draw.
    /// The default implementation returns an empty list
    fn redraw(&self) -> Vec<Sprite> { vec![] }

    /// Called at about 60 fps, with the actual duration between calls passed
    /// as a parameter.
    fn tick(&mut self, _dt: Duration) {}

    /// Called when the user tries to close the window. The default implementation
    /// returns true, which will terminate the window, but if this returns false then
    /// you can prevent the window being closed (to bring up a confirm dialog?)
    fn exit(&mut self) -> bool { true }

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
