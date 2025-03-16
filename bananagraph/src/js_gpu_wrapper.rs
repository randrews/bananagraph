use std::ops::{Deref, DerefMut, Index};
use std::time::Duration;
use crate::{Click, Dir, ElementState, GpuWrapper, IdBuffer, MouseButton, WindowEventHandler};
use wasm_bindgen::prelude::wasm_bindgen;
use crate::event_handler::KeyEvent;

/// We can't send a GpuWrapper to JS directly without it trying to generate stuff it can't generate
/// so we need to wrap it in a bindgen'd type so we can tell bindgen to skip it. We also can't expose
/// a lifetime'd type to JS so all we can really do is make it static
#[wasm_bindgen]
pub struct JsGpuWrapper {
    #[wasm_bindgen(skip)]
    pub(crate) wrapper: GpuWrapper<'static>,

    #[wasm_bindgen(skip)]
    pub(crate) handler: Box<dyn WindowEventHandler>,

    #[wasm_bindgen(skip)]
    pub(crate) ids: Option<IdBuffer>
}

impl Deref for JsGpuWrapper {
    type Target = GpuWrapper<'static>;

    fn deref(&self) -> &Self::Target {
        &self.wrapper
    }
}

impl DerefMut for JsGpuWrapper {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.wrapper
    }
}

impl JsGpuWrapper {
    pub fn new(wrapper: GpuWrapper<'static>, handler: impl WindowEventHandler + 'static) -> Self {
        Self {
            wrapper,
            handler: Box::from(handler),
            ids: None,
        }
    }
}

#[wasm_bindgen]
impl JsGpuWrapper {
    /// Take an event type (mousedown, mouseup, mousemove) and a coord pair and
    /// call the appropriate method on the gamestate (translate between js and windoweventhandler
    /// mouse events).
    pub fn mouse_event(&mut self, event_type: &str, x: f64, y: f64) {
        let mouse_pos = (x, y).into();
        let entity = match &self.ids {
            None => None,
            Some(buf) => {

                let id = *buf.index((x, y).into());
                if id == 0 {
                    None
                } else {
                    Some(id)
                }
            }
        };

        match event_type {
            "mousedown" => {
                self.handler.click(Click {
                    button: MouseButton::Left,
                    state: ElementState::Pressed,
                    mouse_pos,
                    entity
                })
            }
            "mouseup" => {
                self.handler.click(Click {
                    button: MouseButton::Left,
                    state: ElementState::Released,
                    mouse_pos,
                    entity
                })
            }
            "mousemove" => {
                // TODO
            }
            _ => {}
        }
    }

    pub fn key(&mut self, key: &str) {
        to_banana_key(key).map(|ev| self.handler.key(ev));
    }

    pub fn redraw(&mut self, dt: f64) {
        let dt = Duration::from_millis(dt as u64);
        // TODO normally we'd have some logic about exiting the game here, but, we're in a browser,
        // so exiting the game just means closing the tab, which we have no control over.
        self.handler.tick(dt);
        self.ids = self.handler.redraw((0.0, 0.0).into(), &self.wrapper)
    }
}

fn to_banana_key(key: &str) -> Option<KeyEvent> {
    match key {
        "ArrowDown" => Some(KeyEvent::Arrow(Dir::South)),
        "ArrowUp" => Some(KeyEvent::Arrow(Dir::North)),
        "ArrowLeft" => Some(KeyEvent::Arrow(Dir::West)),
        "ArrowRight" => Some(KeyEvent::Arrow(Dir::East)),
        "Enter" => Some(KeyEvent::Enter),
        "Escape" => Some(KeyEvent::Esc),
        _ => {
            // Turn the string into chars, which are unicode scalar values, which isn't
            // perfect but is better than using bytes or something.
            // If there's exactly one (extremely normal case, US keyboard letter key)
            // then wrap it as an event and return it
            let ch: Vec<_> = key.chars().collect();
            if ch.len() == 1 {
                Some(KeyEvent::Letter(ch[0]))
            } else {
                None
            }
        }
    }
}