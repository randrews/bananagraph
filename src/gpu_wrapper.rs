use crate::scale_transform;
use std::default::Default;
use std::f32::consts::PI;
use std::ops::Mul;
use cgmath::{point2, point3, Deg, EuclideanSpace, Matrix3, Point2};
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BlendComponent, BlendFactor, BlendOperation, BlendState, Buffer, BufferUsages, ColorWrites, CompareFunction, Device, LoadOp, StoreOp, Texture, TextureFormat, TextureUsages};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use crate::sprite::{RawSprite, Sprite};

pub struct GpuWrapper<'a> {
    adapter: wgpu::Adapter,
    device: Device,
    queue: wgpu::Queue,
    render_pipeline: wgpu::RenderPipeline,
    window: &'a Window,
    surface: wgpu::Surface<'a>,
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    render_uniform_buffer: Buffer,
    sampler: wgpu::Sampler,
    depth_texture: crate::texture::Texture,
    spritesheet: crate::texture::Texture,
    sprites: Vec<Sprite>
}

impl<'a> GpuWrapper<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let (surface, adapter, device, queue) = Self::create_device(window).await;
        let config = Self::surface_config(&surface, &adapter, window.inner_size());
        let depth_texture = crate::texture::Texture::create_depth_texture(&device, &config);
        surface.configure(&device, &config);

        let spritesheet = crate::texture::Texture::from_bytes(&device, &queue, include_bytes!("cardsLarge_tilemap_packed.png"), Some("spritesheet")).unwrap();

        let crown = Sprite::new((664, 87), (16, 16), spritesheet.size);
        let card = Sprite::new((139, 130), (42, 60), spritesheet.size);
        let mut sprites = Vec::new();

        for n in 0..10 {
            sprites.push(
                card
                    .with_z(n as f32 / 50.0)
                    .translate((-0.5, -0.5))
                    .size_scale()
                    .rotate(Deg(10.0 * n as f32))
                    .inv_size_scale()
                    .translate((0.5, 0.5))
                    .inv_scale((640.0, 480.0))
                    .translate((n as f32 / 640.0, 0.0))
                    .size_scale()
            )
        }
        sprites.sort_by(|a, b| b.z.total_cmp(&a.z));

        let render_uniform_buffer = Self::create_buffer(&device, "render-uniform-buffer", (16 * 4) as wgpu::BufferAddress, BufferUsages::UNIFORM | BufferUsages::COPY_DST);

        let (vertex_buffer, vertex_buffer_layout) = Self::create_vertex_buffer(&device);
        let index_buffer = Self::create_index_buffer(&device);
        let render_pipeline = Self::create_render_pipeline(&device, vertex_buffer_layout);
        let sampler = Self::create_sampler(&device);

        Self {
            adapter,
            device,
            queue,
            render_pipeline,
            window,
            surface,
            vertex_buffer,
            index_buffer,
            render_uniform_buffer,
            sampler,
            depth_texture,
            spritesheet,
            sprites,
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

    fn create_vertex_buffer(device: &Device) -> (Buffer, wgpu::VertexBufferLayout) {
        let vertex_data: [[f32; 2]; 4] = [
            [0.0, 1.0],
            [0.0, 0.0],
            [1.0, 0.0],
            [1.0, 1.0],
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
                    // The nearest-neighbor sampler
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        view_dimension: wgpu::TextureViewDimension::D2,
                        multisampled: false,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true }
                    },
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
                buffers: &[vertex_buffer_layout, RawSprite::desc()],
            },
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleList,
                front_face: wgpu::FrontFace::Ccw,
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: CompareFunction::Less,
                stencil: Default::default(),
                bias: Default::default(),
            }),
            multisample: Default::default(),
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: "fs_main",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: TextureFormat::Bgra8Unorm,
                    // blend: Some(wgpu::BlendState {
                    //     color: wgpu::BlendComponent {
                    //         src_factor: BlendFactor::SrcAlpha,
                    //         dst_factor: BlendFactor::OneMinusSrcAlpha,
                    //         operation: BlendOperation::Add,
                    //     },
                    //     alpha: BlendComponent::OVER,
                    // }),
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
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
        self.depth_texture = crate::texture::Texture::create_depth_texture(&self.device, &config);
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
    fn render_bind_group(&self) -> wgpu::BindGroup {
        let bind_group_layout = self.render_pipeline.get_bind_group_layout(0);
        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::Sampler(&self.sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(&self.spritesheet.view),
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

    fn create_instance_buffer(&self) -> Buffer {
        let raw_sprites = self.sprites.iter().map(RawSprite::from).collect::<Vec<RawSprite>>();
        self.device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&raw_sprites),
                usage: BufferUsages::VERTEX,
            }
        )
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
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.depth_texture.view,
                depth_ops: Some(wgpu::Operations {
                    load: LoadOp::Clear(1.0),
                    store: StoreOp::Store,
                }),
                stencil_ops: None,
            }),
            ..Default::default()
        });
        rpass.set_pipeline(&self.render_pipeline);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        let instance_buffer = self.create_instance_buffer();
        rpass.set_vertex_buffer(1, instance_buffer.slice(..));

        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
        rpass.set_bind_group(0, &self.render_bind_group(), &[]);
        rpass.draw_indexed(0..6, 0, 0..(self.sprites.len() as u32));
    }

    pub fn redraw(&self) {
        let start = std::time::Instant::now();
        let tex = self.surface.get_current_texture().unwrap();

        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());
        self.bind_for_render();
        self.call_render_shader(&mut encoder, &tex);
        self.queue.submit(Some(encoder.finish()));
        tex.present();
        let end = std::time::Instant::now();
        println!("Duration: {}", (end - start).as_micros());
    }
}
