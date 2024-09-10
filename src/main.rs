use std::{borrow::Cow, mem::size_of_val, str::FromStr};
use std::ops::Range;
use std::sync::{Arc, Mutex};
use wgpu::{BufferAddress, BufferAsyncError, Maintain};
use wgpu::util::{BufferInitDescriptor, DeviceExt, DownloadBuffer};

// Indicates a u32 overflow in an intermediate Collatz value
const OVERFLOW: u32 = 0xffffffff;

#[cfg_attr(test, allow(dead_code))]
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

#[cfg_attr(test, allow(dead_code))]
async fn execute_gpu(max: u32) -> Option<Vec<u32>> {
    // Instantiates instance of WebGPU
    let instance = wgpu::Instance::default();

    // `request_adapter` instantiates the general connection to the GPU
    let adapter = instance
        .request_adapter(&wgpu::RequestAdapterOptions::default())
        .await?;

    // `request_device` instantiates the feature specific connection to the GPU, defining some parameters,
    //  `features` being the available features.
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

async fn execute_gpu_inner(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    max: u32
) -> Option<Vec<u32>> {
    // Loads the shader from WGSL
    let cs_module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
        label: None,
        source: wgpu::ShaderSource::Wgsl(Cow::Borrowed(include_str!("shader.wgsl"))),
    });

    // // Gets the size in bytes of the buffer.
    let size = (4 * max) as wgpu::BufferAddress;

    // Instantiates buffer with data (`numbers`).
    // Usage allowing the buffer to be:
    //   A storage buffer (can be bound within a bind group and thus available to a shader).
    //   The destination of a copy.
    //   The source of a copy.
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

    // Instantiates the bind group, once again specifying the binding of buffers.
    let bind_group_layout = compute_pipeline.get_bind_group_layout(0);
    let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
        label: None,
        layout: &bind_group_layout,
        entries: &[wgpu::BindGroupEntry {
            binding: 0,
            resource: storage_buffer.as_entire_binding(),
        }],
    });

    // A command encoder executes one or many pipelines.
    // It is to WebGPU what a command buffer is to Vulkan.
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
        cpass.dispatch_workgroups(max as u32, 1, 1); // Number of cells to run, the (x,y,z) size of item being processed
    }

    // Submits command encoder for processing
    queue.submit(Some(encoder.finish()));

    let result: Option<Result<DownloadBuffer, BufferAsyncError>> = None;
    let mut m = Arc::new(Mutex::new(result));
    let mut m2 = m.clone();
    DownloadBuffer::read_buffer(&device, &queue, &storage_buffer.slice(..), move|x| { m.lock().unwrap().insert(x); });
    let result = loop {
        if let Some(result) = m2.lock().unwrap().take() {
            break result
        }
        device.poll(Maintain::wait()).panic_on_timeout()
    };
    Some(bytemuck::cast_slice(&result.unwrap()).to_vec())
}

pub fn main() {
    env_logger::init();
    pollster::block_on(run());
}
