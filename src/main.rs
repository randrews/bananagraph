mod gpu_wrapper;
mod scale_transform;
mod window_geometry;
mod sprite;
mod texture;
mod id_buffer;

use std::time::{Duration, Instant};
use rand::RngCore;
use crate::gpu_wrapper::GpuWrapper;
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::{ElementState, Event, MouseButton, StartCause, WindowEvent};
use winit::event_loop::ControlFlow;
use crate::sprite::Sprite;

pub async fn run_window() -> Result<(), EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop!");

    let window = winit::window::WindowBuilder::new()
        .with_title("The Thing")
        .with_inner_size(LogicalSize { width: 640, height: 480 })
        .with_min_inner_size(LogicalSize { width: 640, height: 480 })
        .build(&event_loop)?;

    let mut wrapper = GpuWrapper::new(&window).await;
    wrapper.add_texture(include_bytes!("cardsLarge_tilemap_packed.png"), Some("cards")); // normally we'd save the id but we know it's 0
    wrapper.add_texture(include_bytes!("diceRed_border.png"), Some("dice"));
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
            Event::WindowEvent { event: WindowEvent::RedrawRequested, window_id } if window_id == our_id => { wrapper.redraw(board_sprites()); },

            // Resize if it's resizing time
            Event::WindowEvent { event: WindowEvent::Resized(_), window_id } if window_id == our_id => wrapper.handle_resize(),

            // Start the timer on init
            Event::NewEvents(StartCause::Init) => {
                target.set_control_flow(ControlFlow::WaitUntil(Instant::now() + timer_length));
            }

            // When the timer fires, redraw thw window and restart the timer (update will go here)
            Event::NewEvents(StartCause::ResumeTimeReached { .. }) => {
                let time = wrapper.redraw(board_sprites());
                if time.as_millis() >= 500 { println!("Slow frame: {} ms", time.as_millis()) };
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
    let white = Sprite::new((320, 0), (32, 48)).with_layer(2);
    let black = Sprite::new((352, 0), (32, 48)).with_layer(2);

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

    let toph = Sprite::new((416, 0), (32, 48)).with_layer(2);
    let btmh = Sprite::new((384, 0), (32, 48)).with_layer(2);

    let crate_tile = Sprite::new((256, 288), (32, 48)).with_layer(2);

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

fn make_sprites() -> Vec<Sprite> {
    let crown = Sprite::new((664, 87), (16, 16));
    let card = Sprite::new((139, 130), (42, 60));
    let die = Sprite::new((0, 0), (68, 68)).with_layer(1);
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
                .with_tint((1.0, 0.8, 0.5, 0.6))
                .with_id(n + 1)
        )
    }

    // sprites.push(
    //     die
    //         .with_z(1.5 / 50.0)
    //         .translate((-0.5, -0.5))
    //         .rotate(cgmath::Deg(45.0))
    //         .translate((0.5, 0.5))
    //         .inv_scale((640.0, 480.0))
    //         .translate((1.0 / 640.0, 1.0 / 480.0))
    //         .size_scale()
    //         .scale((0.75, 0.65))
    //         .with_id(50)
    //
    // );

    // let mut rng = rand::thread_rng();
    // for n in 0..10000 {
    //     let sprite = if n % 2 < 3 { card } else { die }.with_z(n as f32 / 10000.0);
    //     sprites.push(
    //         sprite
    //             .translate((-0.5, -0.5))
    //             .rotate(cgmath::Deg((rng.next_u32() % 360) as f32))
    //             .translate((0.5, 0.5))
    //             .inv_scale((640.0, 480.0))
    //             .translate(((rng.next_u32() % 10) as f32 / 640.0, (rng.next_u32() % 10) as f32 / 480.0))
    //             .size_scale()
    //     )
    // }
    sprites
}

pub fn main() {
    env_logger::init();
    let _ = pollster::block_on(run_window());
}
