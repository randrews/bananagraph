use std::time::Duration;
use cgmath::Point2;
use crate::{GpuWrapper, IdBuffer, SpriteId};

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum MouseButton { Left, Right }

#[derive(Copy,Clone,Debug,PartialEq)]
pub enum ElementState { Pressed, Released }

#[derive(Copy, Clone, Debug, PartialEq)]
pub struct Click {
    pub button: MouseButton,
    pub state: ElementState,
    pub mouse_pos: Point2<f64>,
    pub entity: Option<SpriteId>
}

#[derive(Copy, Clone, PartialEq)]
pub enum Dir { North, South, East, West }

#[derive(Clone, PartialEq)]
pub enum KeyEvent {
    Letter(char),
    Enter,
    Esc,
    Arrow(Dir)
}

/// A trait for handling game-level events. Bananagraph can keep track of the winit event loop
/// and translate its events into something more game-level semantic. These all have default
/// implementations so you only need to override the ones you care about, but without `redraw` and
/// `init` at minimum, you can't do very much.
pub trait WindowEventHandler {
    /// Run once at the creation of the window; put any one-time init code here, like
    fn init(&mut self, _wrapper: &mut GpuWrapper) {}

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
        match event {
            KeyEvent::Letter(c) => self.letter_key(c),
            KeyEvent::Enter => self.enter_key(),
            KeyEvent::Esc => self.esc_key(),
            KeyEvent::Arrow(dir) => self.arrow_key(dir)
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
    fn letter_key(&mut self, _c: char) {}
}
