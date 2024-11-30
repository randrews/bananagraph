mod gpu_wrapper;
mod scale_transform;
mod window_geometry;
mod sprite;
mod texture;
mod id_buffer;

use std::time::{Duration, Instant};
use cgmath::{Point2, Vector2};
use crate::gpu_wrapper::GpuWrapper;
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::{ElementState, Event, MouseButton, StartCause, WindowEvent};
use winit::event_loop::ControlFlow;
use crate::sprite::{Layer, Sprite};

pub async fn run_window() -> Result<(), EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop!");

    let window = winit::window::WindowBuilder::new()
        .with_title("The Thing")
        .with_inner_size(LogicalSize { width: 640, height: 480 })
        .with_min_inner_size(LogicalSize { width: 640, height: 480 })
        .build(&event_loop)?;

    let mut wrapper = GpuWrapper::new(&window).await;
    let our_id = window.id();

    let timer_length = Duration::from_millis(20);

    // The mouse position is a float, but seems to still describe positions within the same coord
    // space as the window, so just floor()ing it gives you reasonable coordinates
    let mut mouse_pos: (f64, f64) = (-1f64, -1f64);

    event_loop.run(move |event, target| {
        match event {
            // Exit if we click the little x
            Event::WindowEvent { event: WindowEvent::CloseRequested, window_id } if window_id == our_id => {
                target.exit();
            }

            // Redraw if it's redrawing time
            Event::WindowEvent { event: WindowEvent::RedrawRequested, window_id } if window_id == our_id => { wrapper.redraw(make_sprites()); },

            // Resize if it's resizing time
            Event::WindowEvent { event: WindowEvent::Resized(_), window_id } if window_id == our_id => wrapper.handle_resize(),

            // Start the timer on init
            Event::NewEvents(StartCause::Init) => {
                target.set_control_flow(ControlFlow::WaitUntil(Instant::now() + timer_length));
            }

            // When the timer fires, redraw thw window and restart the timer (update will go here)
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                wrapper.redraw(make_sprites());
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
                println!("Sprite id: {}", id);
            }

            _ => {} // toss the others
        }
    })
}

fn make_sprites() -> Vec<Sprite> {
    let crown = Sprite::new(Layer::Sprite,(664, 87), (16, 16));
    let card = Sprite::new(Layer::Sprite,(139, 130), (42, 60));
    let mut sprites = Vec::new();

    for n in 0..10 {
        sprites.push(
            card
                .with_z(n as f32 / 50.0)
                .translate((-0.5, -0.5))
                .size_scale()
                .rotate(cgmath::Deg(10.0 * n as f32))
                .inv_size_scale()
                .translate((0.5, 0.5))
                .inv_scale((640.0, 480.0))
                .translate((n as f32 / 640.0, 0.0))
                .size_scale()
                .with_id(n + 1)
        )
    }
    sprites.sort_by(|a, b| b.z.total_cmp(&a.z));
    sprites
}

pub fn main() {
    env_logger::init();
    let _ = pollster::block_on(run_window());
}
