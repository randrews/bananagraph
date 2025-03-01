use std::ops::{Deref, DerefMut, Index};
use std::time::Duration;
use log::{debug, info};
use bananagraph::{Click, Dir, ElementState, GpuWrapper, IdBuffer, MouseButton, WindowEventHandler};
use wasm_bindgen::prelude::wasm_bindgen;
use crate::game_state::GameState;

/// We can't send a GpuWrapper to JS directly without it trying to generate stuff it can't generate
/// so we need to wrap it in a bindgen'd type so we can tell bindgen to skip it. We also can't expose
/// a lifetime'd type to JS so all we can really do is make it static
#[wasm_bindgen]
pub struct JsGpuWrapper {
    #[wasm_bindgen(skip)]
    pub(crate) wrapper: GpuWrapper<'static>,

    #[wasm_bindgen(skip)]
    pub(crate) handler: GameState,

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
        debug!("key: ({})", key);
        // TODO this is horribly wrong. This is the default impl of `key` in WindowEventHandler,
        // which consumes winit key events. We need to translate js key events into something we
        // can call key with, which means we need to refactor key to not expect winit events...
        // But as long as the 7drl game doesn't need "raw" kbd handling this is fine.
        match key {
            "ArrowDown" => self.handler.arrow_key(Dir::South),
            "ArrowUp" => self.handler.arrow_key(Dir::North),
            "ArrowLeft" => self.handler.arrow_key(Dir::West),
            "ArrowRight" => self.handler.arrow_key(Dir::East),
            "Enter" => self.handler.enter_key(),
            "Escape" => self.handler.esc_key(),
            _ => {
                if key.len() == 1 {
                    self.handler.letter_key(key)
                }
            }
        }
    }

    pub fn redraw(&mut self, dt: f64) {
        let dt = Duration::from_millis(dt as u64);
        // TODO normally we'd have some logic about exiting the game here, but, we're in a browser,
        // so exiting the game just means closing the tab, which we have no control over.
        self.handler.tick(dt);
        self.ids = self.handler.redraw((0.0, 0.0).into(), &self.wrapper)
    }
}