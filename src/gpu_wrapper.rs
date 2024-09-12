use wgpu::{Adapter, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferAddress, BufferBindingType, BufferDescriptor, BufferUsages, ComputePipeline, Device, Features, Instance, InstanceDescriptor, PipelineLayoutDescriptor, Queue, RequestAdapterOptions, ShaderStages, StorageTextureAccess, Surface, SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension};
use wgpu::BindingResource::TextureView;
use winit::window::Window;
use crate::window_geometry::WindowGeometry;

pub struct GpuWrapper<'a> {
    device: Device,
    queue: Queue,
    adapter: Adapter,
    pipeline: ComputePipeline,
    window: &'a Window,
    surface: Surface<'a>,
    uniform_buffer: Buffer
}

impl<'a> GpuWrapper<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let instance = Instance::new(InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&RequestAdapterOptions {
                power_preference: Default::default(),
                force_fallback_adapter: false,
                compatible_surface: Some(&surface),
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: Features::BGRA8UNORM_STORAGE,
                    required_limits: wgpu::Limits::downlevel_defaults(),
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);

        let surface_format = TextureFormat::Bgra8Unorm;

        let size = window.inner_size();
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let uniform_buffer = device.create_buffer(&BufferDescriptor {
            label: Some("uniform data"),
            size: size_of::<WindowGeometry>() as BufferAddress,
            usage: BufferUsages::UNIFORM | BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::StorageTexture {
                        access: StorageTextureAccess::WriteOnly,
                        format: TextureFormat::Bgra8Unorm,
                        view_dimension: TextureViewDimension::D2,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::COMPUTE,
                    ty: BindingType::Buffer {
                        ty: BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }
            ],
        });

        let pipeline_layout = device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "pixel_shader",
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            adapter,
            device,
            queue,
            pipeline,
            window,
            surface,
            uniform_buffer
        }
    }

    pub fn handle_resize(&mut self) {
        let size = self.window.inner_size();
        let surface_caps = self.surface.get_capabilities(&self.adapter);
        let surface_format = TextureFormat::Bgra8Unorm;
        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::STORAGE_BINDING | TextureUsages::TEXTURE_BINDING,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        self.surface.configure(&self.device, &config);
    }

    pub fn call_shader(&self) {
        let size = self.window.inner_size();
        let geometry = WindowGeometry::new(size, None);

        let bind_group_layout = self.pipeline.get_bind_group_layout(0);

        let tex = self.surface.get_current_texture().unwrap();
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: TextureView(&tex.texture.create_view(&TextureViewDescriptor::default())),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: self.uniform_buffer.as_entire_binding()
                }
            ],
        });

        let mut encoder =
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        {
            let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
                label: None,
                timestamp_writes: None,
            });
            cpass.set_pipeline(&self.pipeline);
            cpass.set_bind_group(0, &bind_group, &[]);
            cpass.dispatch_workgroups(size.width, size.height, 1);
        }

        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&geometry));
        self.queue.submit(Some(encoder.finish()));
        tex.present();
    }
}