mod gpu_wrapper;
mod scale_transform;
mod window_geometry;
mod vulcan_state;

use std::time::{Duration, Instant};
use crate::gpu_wrapper::GpuWrapper;
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::{Event, StartCause, WindowEvent};
use winit::event_loop::ControlFlow;

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

    event_loop.run(move |event, target| {
        match event {
            // Exit if we click the little x
            Event::WindowEvent { event: WindowEvent::CloseRequested, window_id } if window_id == our_id => {
                target.exit();
            }

            // Redraw if it's redrawing time
            Event::WindowEvent { event: WindowEvent::RedrawRequested, window_id } if window_id == our_id => wrapper.redraw(),

            // Redraw if it's redrawing time
            Event::WindowEvent { event: WindowEvent::Resized(_), window_id } if window_id == our_id => wrapper.handle_resize(),

            // Start the timer on init
            Event::NewEvents(StartCause::Init) => {
                target.set_control_flow(ControlFlow::WaitUntil(Instant::now() + timer_length));
            }

            // When the timer fires, redraw thw window and restart the timer (update will go here)
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                wrapper.redraw();
                target.set_control_flow(ControlFlow::WaitUntil(Instant::now() + timer_length));
            }

            _ => {} // toss the others
        }
    })
}

pub fn main() {
    env_logger::init();
    let _ = pollster::block_on(run_window());
}
