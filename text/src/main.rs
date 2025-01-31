mod typeface;

use bananagraph::{DrawingContext, GpuWrapper, Sprite};
use std::time::{Duration, Instant};
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::ControlFlow;
use crate::typeface::{Typeface, TypefaceBuilder};

pub async fn run_window() -> Result<(), EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop!");
    let min_size = (80, 60);

    let window = winit::window::WindowBuilder::new()
        .with_title("Text example")
        .with_inner_size(LogicalSize { width: min_size.0 * 8, height: min_size.1 * 8})
        .with_min_inner_size(LogicalSize { width: min_size.0, height: min_size.1 })
        .build(&event_loop)?;

    let mut wrapper = GpuWrapper::new(&window, min_size).await;

    let mut builder = TypefaceBuilder::new(include_bytes!("Curly-Girly.png"), [0, 0, 0, 0xff], 4, 13);
    builder.add_glyphs("abcdefgh", (7, 15), (1, 65), Some(1));
    builder.add_glyphs("ijklmnop", (7, 15), (1, 81), Some(1));
    builder.add_glyphs("qrstuvwx", (7, 15), (1, 97), Some(1));
    builder.add_glyphs("yz", (7, 15), (1, 113), Some(1));
    builder.set_x_offset('p', -3);
    builder.set_x_offset('j', -3);
    builder.add_sized_glyph(' ', (3, 1), (17, 113));
    let tf: Typeface = builder.into_typeface(&mut wrapper);

    let our_id = window.id();

    let timer_length = Duration::from_millis(20);

    event_loop.run(move |event, target| {
        match event {
            // Exit if we click the little x
            Event::WindowEvent { event: WindowEvent::CloseRequested, window_id } if window_id == our_id => {
                target.exit();
            }

            // Redraw if it's redrawing time
            Event::WindowEvent { event: WindowEvent::RedrawRequested, window_id } if window_id == our_id => {
                wrapper.redraw(redraw(&tf));
            },

            // Resize if it's resizing time
            Event::WindowEvent { event: WindowEvent::Resized(_), window_id } if window_id == our_id => wrapper.handle_resize(),

            // Start the timer on init
            Event::NewEvents(StartCause::Init) => {
                target.set_control_flow(ControlFlow::WaitUntil(Instant::now() + timer_length));
            }

            // When the timer fires, redraw thw window and restart the timer (update will go here)
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                target.set_control_flow(ControlFlow::WaitUntil(Instant::now() + timer_length));
            }

            _ => {} // toss the others
        }
    })
}

fn redraw(tf: &Typeface) -> Vec<Sprite> {
    let dc = DrawingContext::new((160.0, 120.0));
    tf.print(dc, (0.0, 40.0), "i made a thing to render\nvariable width bitmap fonts")
}

pub fn main() {
    let _ = pollster::block_on(run_window());
}
