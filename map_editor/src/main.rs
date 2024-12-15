use bananagraph::{ GpuWrapper, Sprite };
use std::time::{Duration, Instant};
use rand::RngCore;
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::{ElementState, Event, MouseButton, StartCause, WindowEvent};
use winit::event_loop::ControlFlow;

pub async fn run_window() -> Result<(), EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop!");

    let window = winit::window::WindowBuilder::new()
        .with_title("The Thing")
        .with_inner_size(LogicalSize { width: 640, height: 480 })
        .with_min_inner_size(LogicalSize { width: 640, height: 480 })
        .build(&event_loop)?;

    let mut wrapper = GpuWrapper::new(&window).await;
    wrapper.add_texture(include_bytes!("iso_dungeon_world.png"), Some("dungeon"));
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
            Event::WindowEvent { event: WindowEvent::RedrawRequested, window_id } if window_id == our_id => { wrapper.redraw(&board_sprites()); },

            // Resize if it's resizing time
            Event::WindowEvent { event: WindowEvent::Resized(_), window_id } if window_id == our_id => wrapper.handle_resize(),

            // Start the timer on init
            Event::NewEvents(StartCause::Init) => {
                target.set_control_flow(ControlFlow::WaitUntil(Instant::now() + timer_length));
            }

            // When the timer fires, redraw thw window and restart the timer (update will go here)
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                wrapper.redraw_with_ids(board_sprites());
                // println!("{}", time.as_micros());
                // if time.as_millis() >= 500 { println!("Slow frame: {} ms", time.as_millis()) };
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

fn board_sprites() -> Vec<Sprite> {
    let white = Sprite::new((320, 0), (32, 48));
    let black = Sprite::new((352, 0), (32, 48));

    let mut sprites = Vec::new();

    for y in 0..8 {
        for x in 0..8 {
            let tile = if y % 2 == 0 { white } else { black }
                .with_id(x + y * 8 + 100000)
                .with_z((8.0 - y as f32) / 10.0)
                .with_position((32.0 * x as f32 + 16.0 * (y % 2) as f32, 8.0 * y as f32), (320.0, 240.0));

            sprites.push(tile)
        }
    }

    let toph = Sprite::new((416, 0), (32, 48));
    let btmh = Sprite::new((384, 0), (32, 48));

    let crate_tile = Sprite::new((256, 288), (32, 48));

    let (x, y) = (3, 4);
    let z = (8.0 - y as f32) / 10.0;

    sprites.push(toph
        .with_z(z + 0.05)
        .with_position((32.0 * x as f32 + 16.0 * (y % 2) as f32, 8.0 * y as f32), (320.0, 240.0)));

    sprites.push(btmh
        .with_z(z - 0.05)
        .with_position((32.0 * x as f32 + 16.0 * (y % 2) as f32, 8.0 * y as f32), (320.0, 240.0)));


    sprites.push(crate_tile
        .with_z(z - 0.025)
        .with_position((32.0 * x as f32 + 16.0 * (y % 2) as f32, 8.0 * y as f32), (320.0, 240.0)));
    sprites
}

pub fn main() {
    env_logger::init();
    let _ = pollster::block_on(run_window());
}
