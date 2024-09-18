use crate::window_geometry::WindowGeometry;
use std::mem::size_of;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::BindingResource::TextureView;
use wgpu::{
    BindGroup, BindGroupEntry, BindGroupLayout, BindGroupLayoutDescriptor, BindGroupLayoutEntry,
    BindingResource, BindingType, Buffer, BufferAddress, BufferBindingType, BufferDescriptor,
    BufferUsages, ColorTargetState, CommandEncoder, ComputePipeline, Device, Extent3d, Features,
    FragmentState, ImageCopyTexture, IndexFormat, Instance, InstanceDescriptor, Origin3d,
    PipelineLayout, PipelineLayoutDescriptor, Queue, RenderPipeline, RenderPipelineDescriptor,
    RequestAdapterOptions, Sampler, SamplerBindingType, ShaderStages, StorageTextureAccess,
    Surface, SurfaceConfiguration, SurfaceTexture, Texture, TextureDescriptor, TextureDimension,
    TextureFormat, TextureUsages, TextureViewDescriptor, TextureViewDimension, VertexBufferLayout,
    VertexState,
};
use winit::dpi::PhysicalSize;
use winit::window::Window;

pub struct GpuWrapper<'a> {
    device: Device,
    queue: Queue,
    adapter: wgpu::Adapter,
    pipeline: ComputePipeline,
    render_pipeline: RenderPipeline,
    window: &'a Window,
    surface: Surface<'a>,
    uniform_buffer: Buffer,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    compute_texture: Texture,
    render_texture: Texture,
    sampler: Sampler,
}

impl<'a> GpuWrapper<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let (surface, adapter, device, queue) = Self::create_device(window).await;
        let config = Self::surface_config(&surface, &adapter, window.inner_size());
        surface.configure(&device, &config);

        let uniform_buffer = Self::create_buffer(
            &device,
            "uniform-buffer",
            size_of::<WindowGeometry>() as BufferAddress,
            BufferUsages::UNIFORM | BufferUsages::COPY_DST,
        );
        let compute_texture = Self::create_texture(
            &device,
            "compute target texture",
            640,
            480,
            TextureUsages::STORAGE_BINDING | TextureUsages::COPY_SRC,
        );
        let render_texture = Self::create_texture(
            &device,
            "render source texture",
            640,
            480,
            TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
        );

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
            uniform_buffer,
            vertex_buffer,
            index_buffer,
            compute_texture,
            render_texture,
            sampler,
        }
    }

    async fn create_device(window: &Window) -> (Surface, wgpu::Adapter, Device, Queue) {
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

        let limits = wgpu::Limits {
            max_texture_dimension_2d: 8192,
            ..wgpu::Limits::downlevel_defaults()
        };

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

    fn create_texture(
        device: &Device,
        label: &str,
        width: u32,
        height: u32,
        usage: TextureUsages,
    ) -> Texture {
        device.create_texture(&TextureDescriptor {
            label: Some(label),
            size: Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8Unorm,
            usage,
            view_formats: &[],
        })
    }

    fn create_buffer(
        device: &Device,
        label: &str,
        size: BufferAddress,
        usage: BufferUsages,
    ) -> Buffer {
        device.create_buffer(&BufferDescriptor {
            label: Some(label),
            size,
            usage,
            mapped_at_creation: false,
        })
    }

    fn pipeline_layout_for(device: &Device, bind_group_layout: BindGroupLayout) -> PipelineLayout {
        device.create_pipeline_layout(&PipelineLayoutDescriptor {
            label: None,
            bind_group_layouts: &[&bind_group_layout],
            push_constant_ranges: &[],
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

    fn create_vertex_buffer(device: &Device) -> (Buffer, VertexBufferLayout) {
        let vertex_data: [[f32; 2]; 4] = [
            // See: https://github.com/parasyte/pixels/issues/180
            [-1.0, -1.0],
            [-1.0, 1.0],
            [1.0, 1.0],
            [1.0, -1.0],
        ];
        let vertex_data_slice = bytemuck::cast_slice(&vertex_data);

        let vertex_buffer_layout = wgpu::VertexBufferLayout {
            array_stride: (vertex_data_slice.len() / vertex_data.len()) as BufferAddress,
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
            //contents: bytemuck::cast_slice(&[0, 1, 2, 2, 3, 0]),
            contents: bytemuck::cast_slice(&[0, 2, 1, 0, 3, 2]),
            usage: BufferUsages::INDEX,
        })
    }

    fn create_render_pipeline(
        device: &Device,
        vertex_buffer_layout: VertexBufferLayout,
    ) -> RenderPipeline {
        // The render shader itself, loaded from WGSL
        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("render_shader.wgsl").into()),
        });

        let bind_group_layout = device.create_bind_group_layout(&BindGroupLayoutDescriptor {
            label: None,
            entries: &[
                BindGroupLayoutEntry {
                    binding: 0,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Texture {
                        sample_type: Default::default(),
                        view_dimension: TextureViewDimension::D2,
                        multisampled: false,
                    },
                    count: None,
                },
                BindGroupLayoutEntry {
                    binding: 1,
                    visibility: ShaderStages::FRAGMENT,
                    ty: BindingType::Sampler(SamplerBindingType::Filtering),
                    count: None,
                },
                // BindGroupLayoutEntry {
                //     binding: 2,
                //     visibility: ShaderStages::FRAGMENT | ShaderStages::VERTEX,
                //     ty: BindingType::Buffer {
                //         ty: BufferBindingType::Uniform,
                //         has_dynamic_offset: false,
                //         min_binding_size: None,
                //     },
                //     count: None,
                // },
            ],
        });

        device.create_render_pipeline(&RenderPipelineDescriptor {
            label: None,
            layout: Some(&Self::pipeline_layout_for(device, bind_group_layout)),
            vertex: VertexState {
                module: &shader,
                entry_point: "vs_main",
                compilation_options: Default::default(),
                buffers: &[vertex_buffer_layout],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                strip_index_format: None,
                front_face: wgpu::FrontFace::Ccw,
                cull_mode: None,
                unclipped_depth: false,
                polygon_mode: Default::default(),
                conservative: false,
            },
            depth_stencil: None,
            multisample: Default::default(),
            fragment: Some(FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(ColorTargetState {
                    format: TextureFormat::Bgra8Unorm,
                    blend: None,
                    write_mask: Default::default(),
                })],
            }),
            multiview: None,
            cache: None,
        })
    }

    // Create a texture sampler with nearest neighbor
    fn create_sampler(device: &Device) -> Sampler {
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

    fn surface_config(
        surface: &Surface,
        adapter: &wgpu::Adapter,
        size: PhysicalSize<u32>,
    ) -> SurfaceConfiguration {
        let surface_caps = surface.get_capabilities(adapter);
        SurfaceConfiguration {
            usage: TextureUsages::COPY_DST | TextureUsages::RENDER_ATTACHMENT,
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
                    resource: TextureView(
                        &self
                            .compute_texture
                            .create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: self.uniform_buffer.as_entire_binding(),
                },
            ],
        })
    }

    // Actually write things to the binds
    fn bind_for_compute(&self) {
        let size = self.window.inner_size();
        let geometry = WindowGeometry::new(size, None);
        self.queue
            .write_buffer(&self.uniform_buffer, 0, bytemuck::bytes_of(&geometry));
    }

    // The bind group for the compute pass
    fn render_bind_group(&self) -> BindGroup {
        let bind_group_layout = self.render_pipeline.get_bind_group_layout(0);
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: TextureView(
                        &self
                            .render_texture
                            .create_view(&TextureViewDescriptor::default()),
                    ),
                },
                BindGroupEntry {
                    binding: 1,
                    resource: BindingResource::Sampler(&self.sampler),
                },
            ],
        })
    }

    #[allow(unused)]
    fn copy_texture_to_surface(&self, encoder: &mut CommandEncoder, tex: &SurfaceTexture) {
        encoder.copy_texture_to_texture(
            ImageCopyTexture {
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
            },
            Extent3d {
                width: 640,
                height: 480,
                depth_or_array_layers: 1,
            },
        );
    }

    fn copy_texture_to_texture(&self, encoder: &mut CommandEncoder) {
        encoder.copy_texture_to_texture(
            ImageCopyTexture {
                texture: &self.compute_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: Default::default(),
            },
            ImageCopyTexture {
                texture: &self.render_texture,
                mip_level: 0,
                origin: Origin3d::ZERO,
                aspect: Default::default(),
            },
            Extent3d {
                width: 640,
                height: 480,
                depth_or_array_layers: 1,
            },
        );
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

    fn call_render_shader(&self, encoder: &mut CommandEncoder, surface: &SurfaceTexture) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: None,
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &surface.texture.create_view(&Default::default()),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color::BLUE),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));
        rpass.set_index_buffer(self.index_buffer.slice(..), IndexFormat::Uint32);
        rpass.set_bind_group(0, &self.render_bind_group(), &[]);
        rpass.draw_indexed(0..3, 0, 0..2);
    }

    pub fn redraw(&self) {
        let tex = self.surface.get_current_texture().unwrap();

        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor { label: None });
        self.call_compute_shader(&mut encoder);
        self.copy_texture_to_texture(&mut encoder);
        self.call_render_shader(&mut encoder, &tex);
        self.queue.submit(Some(encoder.finish()));
        tex.present();
    }
}
