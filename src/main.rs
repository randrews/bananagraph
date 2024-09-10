mod gpu_wrapper;

use std::borrow::Cow;
use std::sync::{Arc, Mutex};
use wgpu::{BufferAsyncError, Maintain};
use wgpu::util::DownloadBuffer;
use winit::dpi::LogicalSize;
use winit::error::EventLoopError;
use winit::event::{Event, WindowEvent};
use crate::gpu_wrapper::GpuWrapper;

async fn run() {
    let steps = execute_gpu(65535).await.unwrap();

    let (n, steps) = steps.iter().enumerate().fold((0usize, 0u32), |acc, el| {
        if acc.1 <= *el.1 {
            (el.0, *el.1)
        } else {
            acc
        }
    });

    println!("Max steps: {} at {}", n, steps);
}

async fn execute_gpu(max: u32) -> Option<Vec<u32>> {
    let instance = wgpu::Instance::default();

    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await?;

    let (device, queue) = adapter
        .request_device(
            &wgpu::DeviceDescriptor {
                label: None,
                required_features: wgpu::Features::empty(),
                required_limits: wgpu::Limits::downlevel_defaults(),
                memory_hints: wgpu::MemoryHints::MemoryUsage,
            },
            None,
        )
        .await
        .unwrap();

    execute_gpu_inner(&device, &queue, max).await
}

async fn execute_gpu_inner(device: &wgpu::Device, queue: &wgpu::Queue, max: u32) -> Option<Vec<u32>> {
    // Loads the shader from WGSL
    let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    // Gets the size in bytes of the buffer.
    let size = (4 * max) as wgpu::BufferAddress;

    // Create a buffer for the shader to store the values it computes.
    // This can be used as storage and a copy source (needed for DownloadBuffer to read it)
    let storage_buffer = device.create_buffer(&wgpu::BufferDescriptor {
        label: Some("Storage Buffer"),
        size,
        usage: wgpu::BufferUsages::STORAGE
            | wgpu::BufferUsages::COPY_SRC,
        mapped_at_creation: false
    });

    let compute_pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
        label: None,
        layout: None,
        module: &cs_module,
        entry_point: "main",
        compilation_options: Default::default(),
        cache: None,
    });

    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: storage_buffer.as_entire_binding(),
        }],
    });

    let mut encoder =
        device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
    {
        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cpass.set_pipeline(&compute_pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.insert_debug_marker("compute collatz iterations");
        cpass.dispatch_workgroups(max, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
    }

    // Submits command encoder for processing
    queue.submit(Some(encoder.finish()));

    let result: Option<Result<DownloadBuffer, BufferAsyncError>> = None;
    let m = Arc::new(Mutex::new(result));
    let m2 = m.clone();
    DownloadBuffer::read_buffer(device, queue, &storage_buffer.slice(..), move|x| { let _ = m.lock().unwrap().insert(x); });
    let result = loop {
        if let Some(result) = m2.lock().unwrap().take() {
            break result
        }
        device.poll(Maintain::wait()).panic_on_timeout()
    };
    Some(bytemuck::cast_slice(&result.unwrap()).to_vec())
}

pub async fn run_window() -> Result<(), EventLoopError> {
    let event_loop = winit::event_loop::EventLoop::new().expect("Failed to create event loop!");

    let window = winit::window::WindowBuilder::new()
        .with_title("The Thing")
        .with_inner_size(LogicalSize{ width: 640, height: 480 })
        .with_min_inner_size(LogicalSize { width: 640, height: 480 })
        .build(&event_loop)?;

    let wrapper = GpuWrapper::new(&window).await;
    let our_id = window.id();

    event_loop.run(move |event, target| {
        match event {
            // Exit if we click the little x
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                window_id,
            } if window_id == our_id => { target.exit(); }

            // Redraw if it's redrawing time
            Event::WindowEvent {
                event: WindowEvent::RedrawRequested,
                window_id,
            } if window_id == our_id => {
                wrapper.call_shader()
            }

            _ => {} // toss the others
        }
    })

}

pub fn main() {
    env_logger::init();
    pollster::block_on(run_window());
}
