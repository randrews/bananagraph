use crate::scale_transform;
use crate::window_geometry::WindowGeometry;
use std::default::Default;
use std::mem::size_of;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::BindingResource::TextureView;
use wgpu::{Buffer, BufferUsages, Device, Texture, TextureUsages};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use crate::vulcan_state::random_vulcan;

pub struct GpuWrapper<'a> {
    device: Device,
    queue: wgpu::Queue,
    adapter: wgpu::Adapter,
    pipeline: wgpu::ComputePipeline,
    render_pipeline: wgpu::RenderPipeline,
    window: &'a Window,
    surface: wgpu::Surface<'a>,
    vulcan_state_buffer: Buffer,
    render_uniform_buffer: Buffer,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    compute_texture: Texture,
    render_texture: Texture,
    sampler: wgpu::Sampler,
    vulcan_mem: [u8; 131072],
}

impl<'a> GpuWrapper<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let (surface, adapter, device, queue) = Self::create_device(window).await;
        let config = Self::surface_config(&surface, &adapter, window.inner_size());
        surface.configure(&device, &config);

        let vulcan_state_buffer = Self::create_buffer(&device, "vulcan-state-buffer", 131072 as wgpu::BufferAddress, BufferUsages::STORAGE | BufferUsages::COPY_DST);
        let render_uniform_buffer = Self::create_buffer(&device, "render-uniform-buffer", (16 * 4) as wgpu::BufferAddress, BufferUsages::UNIFORM | BufferUsages::COPY_DST);
        let compute_texture = Self::create_texture(&device, "compute target texture", 640, 480, TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC);
        let render_texture = Self::create_texture(&device, "render source texture", 640, 480, TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST);

        let vulcan_mem = random_vulcan(4);

        let pipeline = Self::create_compute_pipeline(&device);

        let (vertex_buffer, vertex_buffer_layout) = Self::create_vertex_buffer(&device);
        let index_buffer = Self::create_index_buffer(&device);
        let render_pipeline = Self::create_render_pipeline(&device, vertex_buffer_layout);
        let sampler = Self::create_sampler(&device);

        Self {
            adapter,
            device,
            queue,
            pipeline,
            render_pipeline,
            window,
            surface,
            vulcan_state_buffer,
            render_uniform_buffer,
            vertex_buffer,
            index_buffer,
            compute_texture,
            render_texture,
            sampler,
            vulcan_mem,
        }
    }

    async fn create_device(window: &Window) -> (wgpu::Surface, wgpu::Adapter, Device, wgpu::Queue) {
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::PRIMARY,
            ..Default::default()
        });

        let surface = instance.create_surface(window).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                compatible_surface: Some(&surface),
                ..Default::default()
            })
            .await
            .unwrap();

        let limits = wgpu::Limits {
            max_texture_dimension_2d: 8192,
            ..wgpu::Limits::downlevel_defaults()
        };

        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: None,
                    required_features: wgpu::Features::BGRA8UNORM_STORAGE,
                    required_limits: limits,
                    memory_hints: wgpu::MemoryHints::MemoryUsage,
                },
                None,
            )
            .await
            .unwrap();

        (surface, adapter, device, queue)
    }

    fn create_texture(device: &Device, label: &str, width: u32, height: u32, usage: TextureUsages) -> Texture {
        device.create_texture(&wgpu::TextureDescriptor {
            label: Some(label),
            size: wgpu::Extent3d { width, height, depth_or_array_layers: 1 },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Bgra8Unorm,
            usage,
            view_formats: &[],
        })
    }

    fn create_buffer(device: &Device, label: &str, size: wgpu::BufferAddress, usage: BufferUsages) -> Buffer {
        device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        })
    }

    fn pipeline_layout_for(device: &Device, bind_group_layout: wgpu::BindGroupLayout) -> wgpu::PipelineLayout {
        device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            bind_group_layouts: &[&bind_group_layout],
            ..Default::default()
        })
    }

    fn create_compute_pipeline(device: &Device) -> wgpu::ComputePipeline {
        // The compute shader itself, loaded from WGSL
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("compute_shader.wgsl").into()),
        });

        // The layout for its two binds: the texture we'll render to and the uniform buffer
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    // The output texture
                    binding: 0,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::StorageTexture {
                        access: wgpu::StorageTextureAccess::WriteOnly,
                        format: wgpu::TextureFormat::Bgra8Unorm,
                        view_dimension: wgpu::TextureViewDimension::D2,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // The Vulcan state buffer
                    binding: 1,
                    visibility: wgpu::ShaderStages::COMPUTE,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Storage { read_only: true },
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: Some(&Self::pipeline_layout_for(device, bind_group_layout)),
            module: &shader,
            entry_point: "pixel_shader",
            compilation_options: Default::default(),
            cache: None,
        })
    }

    fn create_vertex_buffer(device: &Device) -> (Buffer, wgpu::VertexBufferLayout) {
        let vertex_data: [[f32; 2]; 4] = [
            // There's an option here we're not using: essentially we could render one
            // triangle, but that triangle could be huge and overlap the entire window.
            // See: https://github.com/parasyte/pixels/issues/180
            // Reason I'm not doing it is it violates the Vulcan "straightforwardness"
            // principle.
            [-1.0, -1.0],
            [-1.0, 1.0],
            [1.0, 1.0],
            [1.0, -1.0],
        ];
        let vertex_data_slice = bytemuck::cast_slice(&vertex_data);

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: (vertex_data_slice.len() / vertex_data.len()) as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &[wgpu::VertexAttribute {
                format: wgpu::VertexFormat::Float32x2,
                offset: 0,
                shader_location: 0,
            }],
        };

        let vertex_buffer = device.create_buffer_init(&BufferInitDescriptor {
            label: Some("vertex_buffer"),
            contents: vertex_data_slice,
            usage: BufferUsages::VERTEX,
        });

        (vertex_buffer, vertex_buffer_layout)
    }

    fn create_index_buffer(device: &Device) -> Buffer {
        device.create_buffer_init(&BufferInitDescriptor {
            label: Some("index bfr"),
            // Two counterclockwise triangles
            contents: bytemuck::cast_slice(&[0, 2, 1, 0, 3, 2]),
            usage: BufferUsages::INDEX,
        })
    }

    fn create_render_pipeline(device: &Device, vertex_buffer_layout: wgpu::VertexBufferLayout) -> wgpu::RenderPipeline {
        // The render shader itself, loaded from WGSL
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("render_shader.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    // The input texture, copied from the compute's output texture
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        sample_type: Default::default(),
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // The nearest-neighbor sampler
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    // The transform matrix for the vertex shader
                    binding: 2,
                    visibility: wgpu::ShaderStages::FRAGMENT | wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                },
            ],
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: None,
            layout: Some(&Self::pipeline_layout_for(device, bind_group_layout)),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[vertex_buffer_layout],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: wgpu::TextureFormat::Bgra8Unorm,
                    blend: None,
                    write_mask: Default::default(),
                })],
            }),
            multiview: None,
            cache: None,
        })
    }

    // Create a texture sampler with nearest neighbor
    fn create_sampler(device: &Device) -> wgpu::Sampler {
        device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("nearest-neighbor-sampler"),
            address_mode_u: wgpu::AddressMode::ClampToEdge,
            address_mode_v: wgpu::AddressMode::ClampToEdge,
            address_mode_w: wgpu::AddressMode::ClampToEdge,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            lod_min_clamp: 0.0,
            lod_max_clamp: 1.0,
            compare: None,
            anisotropy_clamp: 1,
            border_color: None,
        })
    }

    pub fn handle_resize(&mut self) {
        let config = Self::surface_config(&self.surface, &self.adapter, self.window.inner_size());
        self.surface.configure(&self.device, &config);
    }

    fn surface_config(surface: &wgpu::Surface, adapter: &wgpu::Adapter, size: PhysicalSize<u32>) -> wgpu::SurfaceConfiguration {
        let surface_caps = surface.get_capabilities(adapter);
        wgpu::SurfaceConfiguration {
            usage: TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
            format: wgpu::TextureFormat::Bgra8Unorm,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        }
    }

    // The bind group for the compute pass
    // TODO: we may not need to create the bind groups every frame
    fn compute_bind_group(&self) -> wgpu::BindGroup {
        let bind_group_layout = self.pipeline.get_bind_group_layout(0);
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: TextureView(&self.compute_texture.create_view(&wgpu::TextureViewDescriptor::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: self.vulcan_state_buffer.as_entire_binding(),
                },
            ],
        })
    }

    // Actually write things to the binds
    fn bind_for_compute(&self) {
        let size = self.window.inner_size();
        self.queue.write_buffer(&self.vulcan_state_buffer, 0, &self.vulcan_mem);
    }

    // The bind group for the compute pass
    fn render_bind_group(&self) -> wgpu::BindGroup {
        let bind_group_layout = self.render_pipeline.get_bind_group_layout(0);
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: TextureView(&self.render_texture.create_view(&wgpu::TextureViewDescriptor::default())),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: self.render_uniform_buffer.as_entire_binding(),
                },
            ],
        })
    }

    fn bind_for_render(&self) {
        let PhysicalSize { width, height } = self.window.inner_size();
        self.queue.write_buffer(&self.render_uniform_buffer, 0, bytemuck::bytes_of(&scale_transform::transform((640, 480), (width, height))));
    }

    fn copy_texture_to_texture(&self, encoder: &mut wgpu::CommandEncoder) {
        encoder.copy_texture_to_texture(
            wgpu::ImageCopyTexture {
                texture: &self.compute_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: Default::default(),
            },
            wgpu::ImageCopyTexture {
                texture: &self.render_texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: Default::default(),
            },
            wgpu::Extent3d { width: 640, height: 480, depth_or_array_layers: 1 },
        );
    }

    fn call_compute_shader(&self, encoder: &mut wgpu::CommandEncoder) {
        let size = self.window.inner_size();

        let bind_group = self.compute_bind_group();
        self.bind_for_compute();

        let mut cpass = encoder.begin_compute_pass(&wgpu::ComputePassDescriptor::default());
        cpass.set_pipeline(&self.pipeline);
        cpass.set_bind_group(0, &bind_group, &[]);
        cpass.dispatch_workgroups(size.width, size.height, 1);
    }

    fn call_render_shader(&self, encoder: &mut wgpu::CommandEncoder, surface: &wgpu::SurfaceTexture) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface.texture.create_view(&Default::default()),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            ..Default::default()
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        rpass.set_bind_group(0, &self.render_bind_group(), &[]);
        rpass.draw_indexed(0..6, 0, 0..2);
    }

    pub fn redraw(&self) {
        let start = std::time::Instant::now();
        let tex = self.surface.get_current_texture().unwrap();

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        self.call_compute_shader(&mut encoder);
        self.copy_texture_to_texture(&mut encoder);
        self.bind_for_render();
        self.call_render_shader(&mut encoder, &tex);
        self.queue.submit(Some(encoder.finish()));
        tex.present();
        let end = std::time::Instant::now();
        println!("Duration: {}", (end - start).as_micros());
    }
}
