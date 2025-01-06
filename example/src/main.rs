use bananagraph::{DrawingContext, GpuWrapper, Sprite};
use std::time::{Duration, Instant};
use cgmath::Deg;
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::{ElementState, Event, MouseButton, StartCause, WindowEvent};
use winit::event_loop::ControlFlow;

pub async fn run_window() -> Result<(), EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop!");

    let window = winit::window::WindowBuilder::new()
        .with_title("Bananagraph Example")
        .with_inner_size(LogicalSize { width: 800, height: 450 })
        .with_min_inner_size(LogicalSize { width: 800, height: 450 })
        .build(&event_loop)?;

    let mut wrapper = GpuWrapper::new(&window, (800, 450)).await;
    wrapper.add_texture(include_bytes!("cube.png"), None);
    wrapper.add_texture(include_bytes!("background.png"), None);
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
            Event::WindowEvent { event: WindowEvent::RedrawRequested, window_id } if window_id == our_id => {
                redraw(&wrapper)
            },

            // Resize if it's resizing time
            Event::WindowEvent { event: WindowEvent::Resized(_), window_id } if window_id == our_id => wrapper.handle_resize(),

            // Start the timer on init
            Event::NewEvents(StartCause::Init) => {
                target.set_control_flow(ControlFlow::WaitUntil(Instant::now() + timer_length));
            }

            // When the timer fires, redraw thw window and restart the timer (update will go here)
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                redraw(&wrapper);
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
            } if window_id == our_id => {}

            _ => {} // toss the others
        }
    })
}

fn redraw(wrapper: &GpuWrapper) {
    let (w, h) = (400.0, 225.0);
    // let sprite = Sprite::new((0, 0), (32, 32))
    //     //.translate((-0.5, -0.5))
    //     //.scale((32.0, 32.0))
    //     //.inv_scale((800.0, 450.0))
    //     .translate((-0.5, -0.5))
    //     .rotate(Deg(90.0))
    //     //.scale((2.0, 2.0))
    //     .translate((0.5, 0.5))
    //     .scale((32.0 / w, 32.0 / h))
    //     //.inv_scale((w, h))
    //     .translate((1.0 / w * 32.0, 1.0 / h * 16.0))
    //     ;

    let bg = Sprite::new((0, 0), (800, 450)).with_layer(1).with_z(0.2);
    let sprite = Sprite::new((0, 0), (32, 32)).with_z(0.1);

    // A screen the size of the window, but tilted, shrunk, and translated
    let dc = DrawingContext::new((800.0, 450.0))
        .rotate(Deg(-45.0))
        .scale((0.5, 0.5))
        .translate((0.25, 0.5));

    wrapper.redraw([
        dc.place(bg, (0.0, 0.0)), // Just draw the grayness straight
        dc.place_rotated(sprite, (0.0, 0.0), Deg(45.0)), // Rotate the cube about its center
        sprite
            .scale((32.0, 32.0)) // Scale to the sprite size in pixels. Sprite is now huge, and distorted!
            .translate((100.0, 100.0)) // Translate to pixel coords
            .rotate(Deg(10.0)) // Rotate however we like
            .scale((1.0 / 400.0, 1.0 / 225.0)) // Scale everything back down by the size of the world, also removes distortion
            .with_tint((1.0, 1.0, 0.4, 1.0))
            .with_z(0.01),
        sprite
            .scale((32.0, 32.0)) // Scale to the sprite size in pixels. Sprite is now huge, and distorted!
            .translate((-16.0, -16.0)) // Translate to put the center on the origin
            .rotate(Deg(45.0)) // Rotate however we like
            .translate((26.0, 26.0)) // translate back, and then some, in pixel coords
            .scale((1.0 / 400.0, 1.0 / 225.0)) // Scale everything back down by the size of the world, also removes distortion
            .with_z(0.008),
        bg.with_z(0.9999)
    ]);
}

pub fn main() {
    let _ = pollster::block_on(run_window());
}
