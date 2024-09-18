use std::mem::size_of;
use wgpu::{Adapter, BindGroup, BindGroupEntry, BindGroupLayoutDescriptor, BindGroupLayoutEntry, BindingType, Buffer, BufferAddress, BufferBindingType, BufferDescriptor, BufferUsages, CommandEncoder, ComputePipeline, Device, Extent3d, Features, ImageCopyTexture, Instance, InstanceDescriptor, Limits, Origin3d, PipelineLayoutDescriptor, Queue, RequestAdapterOptions, ShaderStages, StorageTextureAccess, Surface, SurfaceConfiguration, SurfaceTexture, Texture, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension};
use wgpu::BindingResource::TextureView;
use winit::dpi::PhysicalSize;
use winit::window::Window;
use crate::window_geometry::WindowGeometry;

pub struct GpuWrapper<'a> {
    device: Device,
    queue: Queue,
    adapter: Adapter,
    pipeline: ComputePipeline,
    window: &'a Window,
    surface: Surface<'a>,
    uniform_buffer: Buffer,
    compute_texture: Texture
}

impl<'a> GpuWrapper<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let (surface, adapter, device, queue) = Self::create_device(window).await;
        let config = Self::surface_config(&surface, &adapter, window.inner_size());
        surface.configure(&device, &config);

        let uniform_buffer = Self::create_buffer(&device, "uniform-buffer", size_of::<WindowGeometry>() as BufferAddress, BufferUsages::UNIFORM | BufferUsages::COPY_DST);
        let compute_texture = device.create_texture(&TextureDescriptor {
            label: Some("compute target texture"),
            size: Extent3d {
                width: 640,
                height: 480,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8Unorm,
            usage: TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
            view_formats: &[],
        });

        let pipeline = Self::create_compute_pipeline(&device);

        Self {
            adapter,
            device,
            queue,
            pipeline,
            window,
            surface,
            uniform_buffer,
            compute_texture
        }
    }

    async fn create_device(window: &Window) -> (Surface, Adapter, Device, Queue) {
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

        let limits = Limits::downlevel_defaults();

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: Features::BGRA8UNORM_STORAGE,
                    required_limits: limits,
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await
            .unwrap();

        (surface, adapter, device, queue)
    }

    fn create_buffer(device: &Device, label: &str, size: BufferAddress, usage: BufferUsages) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        })
    }

    fn create_compute_pipeline(device: &Device) -> ComputePipeline {
        // The compute shader itself, loaded from WGSL
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("compute_shader.wgsl").into()),
        });

        // The layout for its two binds: the texture we'll render to and the uniform buffer
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

        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&pipeline_layout),
            module: &shader,
            entry_point: "pixel_shader",
            compilation_options: Default::default(),
            cache: None,
        })
    }

    pub fn handle_resize(&mut self) {
        let config = Self::surface_config(&self.surface, &self.adapter, self.window.inner_size());
        self.surface.configure(&self.device, &config);
    }

    fn surface_config(surface: &Surface, adapter: &Adapter, size: PhysicalSize<u32>) -> SurfaceConfiguration {
        let surface_caps = surface.get_capabilities(adapter);
        SurfaceConfiguration {
            usage: TextureUsages::COPY_DST,
            format: TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        }
    }

    // The bind group for the compute pass
    fn compute_bind_group(&self) -> BindGroup {
        let bind_group_layout = self.pipeline.get_bind_group_layout(0);
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: TextureView(&self.compute_texture.create_view(&TextureViewDescriptor::default())),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: self.uniform_buffer.as_entire_binding()
                }
            ],
        })
    }

    // Actually write things to the binds
    fn bind_for_compute(&self) {
        let size = self.window.inner_size();
        let geometry = WindowGeometry::new(size, None);
        self.queue.write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&geometry));
    }

    fn copy_texture_to_surface(&self, encoder: &mut CommandEncoder, tex: &SurfaceTexture) {
        encoder.copy_texture_to_texture(ImageCopyTexture {
            texture: &self.compute_texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: Default::default(),
        },
        ImageCopyTexture {
            texture: &tex.texture,
            mip_level: 0,
            origin: Origin3d::ZERO,
            aspect: Default::default(),
        }, Extent3d { width: 640, height: 480, depth_or_array_layers: 1 });
    }

    fn call_compute_shader(&self, encoder: &mut CommandEncoder) {
        let size = self.window.inner_size();

        let bind_group = self.compute_bind_group();
        self.bind_for_compute();

        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor {
            label: None,
            timestamp_writes: None,
        });
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(size.width, size.height, 1);
    }

    pub fn redraw(&self) {
        let tex = self.surface.get_current_texture().unwrap();

        let mut encoder =
            self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.call_compute_shader(&mut encoder);
        self.copy_texture_to_surface(&mut encoder, &tex);
        self.queue.submit(Some(encoder.finish()));
        tex.present();
    }
}