use crate::scale_transform;
use std::default::Default;
use std::sync::Arc;
use std::time::Duration;
use wgpu::util::{BufferInitDescriptor, DeviceExt};
use wgpu::{BlendState, Buffer, BufferUsages, Color, ColorWrites, CompareFunction, Device, Extent3d, ImageCopyTexture, ImageDataLayout, LoadOp, ShaderModule, StoreOp, Texture, TextureFormat, TextureUsages};
use winit::dpi::PhysicalSize;
use winit::window::Window;
use crate::id_buffer::IdBuffer;
use crate::sprite::{RawSprite, Sprite};

pub struct GpuWrapper<'a> {
    // The handles to the actual GPU hardware
    adapter: wgpu::Adapter,
    device: Device,

    // A queue to set up commands for a redraw
    queue: wgpu::Queue,

    // Two render pipelines: one for the pixel data and one for
    // sprite IDs for hit detection
    render_pipeline: wgpu::RenderPipeline,
    id_pipeline: wgpu::RenderPipeline,

    // The window and surface of that window that we're rendering to
    window: &'a Window,
    surface: wgpu::Surface<'a>,

    // The "logical" size of the window space, used for creating the
    // scale transform
    pub logical_size: (u32, u32),

    // Inputs to the render pipelines: a unit square, which we need
    // buffers to store on the GPU, and a uniform buffer with the
    // scale transform.
    vertex_buffer: Buffer,
    index_buffer: Buffer,
    render_uniform_buffer: Buffer,

    // The nearest-neighbor sampler for a sharp pixel effect
    sampler: wgpu::Sampler,

    // A texture for the pipeline to write depth data to
    depth_texture: crate::texture::Texture,

    // The textures we'll draw sprites from
    spritesheets: Vec<crate::texture::Texture>,

    // The texture the id pipeline outputs to, and the buffer
    // we read them from
    id_texture: crate::texture::Texture,
    id_buffer: Arc<Buffer>,
}

impl<'a> GpuWrapper<'a> {
    pub async fn new(window: &'a Window, logical_size: (u32, u32)) -> Self {
        let (surface, adapter, device, queue) = Self::create_device(window).await;
        let config = Self::surface_config(&surface, &adapter, window.inner_size());
        let depth_texture = crate::texture::Texture::create_depth_texture(&device, &config);
        let id_texture = crate::texture::Texture::create_id_texture(&device, &config);
        surface.configure(&device, &config);

        let render_uniform_buffer = Self::create_buffer(&device, "render-uniform-buffer", (16 * 4) as wgpu::BufferAddress, BufferUsages::UNIFORM | BufferUsages::COPY_DST);

        let (vertex_buffer, vertex_buffer_layout) = Self::create_vertex_buffer(&device);
        let index_buffer = Self::create_index_buffer(&device);
        let id_buffer = Arc::new(Self::create_id_buffer(&device, &id_texture.texture));
        let shader = Self::create_shader(&device);
        let render_pipeline = Self::create_render_pipeline(&device, vertex_buffer_layout.clone(), &shader);
        let id_pipeline = Self::create_id_pipeline(&device, vertex_buffer_layout, &shader);
        let sampler = Self::create_sampler(&device);

        Self {
            adapter,
            device,
            queue,
            render_pipeline,
            id_pipeline,
            window,
            surface,
            logical_size,
            vertex_buffer,
            index_buffer,
            render_uniform_buffer,
            sampler,
            depth_texture,
            id_texture,
            id_buffer,
            spritesheets: vec![],
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

    fn create_shader(device: &Device) -> ShaderModule {
        // The render shader itself, loaded from WGSL
        device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("render_shader.wgsl").into()),
        })
    }

    fn create_render_pipeline(device: &Device, vertex_buffer_layout: wgpu::VertexBufferLayout, shader: &ShaderModule) -> wgpu::RenderPipeline {
        let bind_group_layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            label: Some("render pipeline"),
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
                module: shader,
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
                    blend: Some(BlendState::ALPHA_BLENDING),
                    write_mask: ColorWrites::ALL,
                })],
            }),
            multiview: None,
            cache: None,
        })
    }

    /// Very similar to the render pipeline, but different in two ways:
    /// One, the fragment stage uses fs_id instead of fs_main, because we want to run a different
    /// shader. Two, the output is R32Uint, because we're just extracting one u32 out of this.
    fn create_id_pipeline(device: &Device, vertex_buffer_layout: wgpu::VertexBufferLayout, shader: &ShaderModule) -> wgpu::RenderPipeline {
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
            label: Some("id shader"),
            layout: Some(&Self::pipeline_layout_for(device, bind_group_layout)),
            vertex: wgpu::VertexState {
                module: shader,
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
                entry_point: "fs_id",
                compilation_options: Default::default(),
                targets: &[Some(wgpu::ColorTargetState {
                    format: TextureFormat::R32Uint,
                    blend: None,
                    write_mask: ColorWrites::RED,
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

    /// Call whenever the window backing all this is resized, to update the various internal
    /// textures and buffers needed for the render pipeline
    pub fn handle_resize(&mut self) {
        let config = Self::surface_config(&self.surface, &self.adapter, self.window.inner_size());
        self.depth_texture = crate::texture::Texture::create_depth_texture(&self.device, &config);
        self.id_texture = crate::texture::Texture::create_id_texture(&self.device, &config);
        self.id_buffer = Arc::new(Self::create_id_buffer(&self.device, &self.id_texture.texture));
        self.surface.configure(&self.device, &config);
    }

    /// Creates a config object for the surface given a physical size. Called by `handle_resize`
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

    /// The bind group for the render pass
    fn render_bind_groups(&self) -> Vec<wgpu::BindGroup> {
        let bind_group_layout = self.render_pipeline.get_bind_group_layout(0);

        self.spritesheets.iter().map(|sp|
            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: None,
                layout: &bind_group_layout,
                entries: &[
                    // The sampler
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: wgpu::BindingResource::Sampler(&self.sampler),
                    },
                    // The texture for the spritesheet
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: wgpu::BindingResource::TextureView(&sp.view),
                    },
                    // The uniform buffer, which contains the overall transform matrix
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.render_uniform_buffer.as_entire_binding(),
                    },
                ],
            })
        ).collect()
    }

    /// Writes the scaling transform matrix to the uniform buffer, so the render pass can pick it up
    fn bind_for_render(&self) {
        let PhysicalSize { width, height } = self.window.inner_size();
        self.queue.write_buffer(&self.render_uniform_buffer, 0, bytemuck::bytes_of(&scale_transform::transform(self.logical_size, (width, height))));
    }

    /// The instance buffer contains the packed sprite data for the render pipeline to iterate over
    fn create_instance_buffer<S: AsRef<Sprite>>(&self, sprites: Vec<S>) -> Buffer {
        let raw_sprites = sprites.into_iter().map(|s| s.as_ref().into_raw(self.spritesheets[s.as_ref().layer as usize].size)).collect::<Vec<RawSprite>>();
        self.device.create_buffer_init(
            &BufferInitDescriptor {
                label: Some("Instance Buffer"),
                contents: bytemuck::cast_slice(&raw_sprites),
                usage: BufferUsages::VERTEX,
            }
        )
    }

    /// Queues a call to an arbitrary shader pipeline, targeting an arbitrary texture view. It will
    /// iterate over the given instances for the unit-square-vertex-buffer.
    fn call_shader(&self, encoder: &mut wgpu::CommandEncoder, instances: &Buffer, layers: &Vec<u32>, pipeline: &wgpu::RenderPipeline, target: &wgpu::TextureView) {
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: target,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: StoreOp::Store,
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
        rpass.set_pipeline(pipeline);
        rpass.set_vertex_buffer(0, self.vertex_buffer.slice(..));

        rpass.set_vertex_buffer(1, instances.slice(..));

        rpass.set_index_buffer(self.index_buffer.slice(..), wgpu::IndexFormat::Uint32);

        let bind_groups = self.render_bind_groups();

        // Go through the runs of same-layer sprites and dispatch draw calls
        let mut start = 0;
        let mut end = 0;
        while start < layers.len() {
            // after this, end is the first one of the new group, start is the first of this group
            while end < layers.len() && layers[start] == layers[end] { end += 1 }

            // Bind the texture for this group
            rpass.set_bind_group(0, &bind_groups[layers[start] as usize], &[]);
            // Draw this run!
            rpass.draw_indexed(0..6, 0, start as u32..end as u32);
            start = end; // Jump to the next group
        }
    }

    /// Queues a call to the render shader, which outputs color data to the surface
    fn call_render_shader(&self, encoder: &mut wgpu::CommandEncoder, instances: &Buffer, layers: &Vec<u32>, surface: &wgpu::SurfaceTexture) {
        self.call_shader(encoder, instances, layers, &self.render_pipeline, &surface.texture.create_view(&Default::default()))
    }

    /// Queues a call to the id shader, which outputs sprite ids to id_texture
    fn call_id_shader(&self, encoder: &mut wgpu::CommandEncoder, instances: &Buffer, layers: &Vec<u32>) {
        let target = self.id_texture.texture.create_view(&wgpu::TextureViewDescriptor {
            format: Some(TextureFormat::R32Uint),
            ..Default::default()
        });

        self.call_shader(encoder, instances, layers, &self.id_pipeline, &target);
    }

    /// We can only copy textures to buffers that are multiples of `COPY_BYTES_PER_ROW_ALIGNMENT`
    /// bytes wide. This is probably 64 pixels, so, we need to round up the size of the buffer to
    /// accommodate that width. For a texture `x` pixels wide, this returns the required row width, which is at least `x`:
    fn id_buffer_width(x: u32) -> u32 {
        let pixels_per_slice = wgpu::COPY_BYTES_PER_ROW_ALIGNMENT / 4; // The texture is u32s, so 4 bytes per pixel
        let slices_per_row = x as f32 / pixels_per_slice as f32; // Figure out how many of those slices per row
        slices_per_row.ceil() as u32 * pixels_per_slice // Round that up and multiply back to pixels
    }

    /// Create a buffer we can copy the id texture into. Thus is likely to be wider than the original
    /// texture, see `id_buffer_width`.
    fn create_id_buffer(device: &Device, id_texture: &Texture) -> Buffer {
        let bpr = Self::id_buffer_width(id_texture.width()) * 4;
        device.create_buffer(&wgpu::BufferDescriptor {
            label: None,
            size: (bpr * id_texture.height()).into(),
            usage: BufferUsages::COPY_DST | BufferUsages::MAP_READ,
            mapped_at_creation: false,
        })
    }

    /// Queues reading the id texture (target of the id shader) into the id buffer.
    fn read_id_texture(&self, encoder: &mut wgpu::CommandEncoder) {
        let size = self.id_texture.size;

        let src = ImageCopyTexture {
            texture: &self.id_texture.texture,
            mip_level: 0,
            origin: Default::default(),
            aspect: Default::default(),
        };
        let dest = wgpu::ImageCopyBuffer {
            buffer: &self.id_buffer,
            layout: ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(Self::id_buffer_width(size.x) * 4),
                rows_per_image: Some(size.y),
            },
        };
        encoder.copy_texture_to_buffer(src, dest, Extent3d {
            width: size.x,
            height: size.y,
            depth_or_array_layers: 1,
        });
    }

    pub fn add_texture(&mut self, bytes: &[u8], label: Option<&str>) -> u32 {
        let spritesheet = crate::texture::Texture::from_bytes(&self.device, &self.queue, bytes, label).unwrap();
        self.spritesheets.push(spritesheet);
        self.spritesheets.len() as u32 - 1
    }

    pub fn add_texture_from_array(&mut self, bytes: Vec<u8>, width: u32, label: Option<&str>) -> u32 {
        let spritesheet = crate::texture::Texture::from_array(&self.device, &self.queue, bytes, width, label).unwrap();
        self.spritesheets.push(spritesheet);
        self.spritesheets.len() as u32 - 1
    }

    /// Sort the given sprite iterator by z and put it into an instance buffer, returning
    /// the buffer and vec of layers (so we know how many / which draw calls to make).
    /// If the iterator contains no sprites, return None
    fn set_sprites<I: IntoIterator<Item=S>,S: AsRef<Sprite>>(&self, sprites: I) -> (Buffer, Vec<u32>) {
        let mut sprites: Vec<_> = sprites.into_iter().collect();

        if !sprites.is_empty() {
            // Prep sprites by sorting them
            sprites.sort_by(|a, b| {
                let (a, b) = (a.as_ref(), b.as_ref());
                if a.z == b.z {
                    b.layer.cmp(&a.layer)
                } else {
                    b.z.total_cmp(&a.z)
                }
            });

            let layers: Vec<u32> = sprites.iter().map(|s| s.as_ref().layer).collect();


            self.bind_for_render();
            let instance_buffer = self.create_instance_buffer(sprites);
            (instance_buffer, layers)
        } else {
            let instance_buffer = self.create_instance_buffer(sprites);
            (instance_buffer, vec![])
        }
    }

    /// Redraws the display, but does not populate the id buffer, returning how long it took to do that.
    pub fn redraw<I: IntoIterator<Item=S>,S: AsRef<Sprite>>(&self, sprites: I) -> Duration {
        let start = std::time::Instant::now();
        let tex = self.surface.get_current_texture().unwrap();
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let (instance_buffer, layers) = self.set_sprites(sprites);
        self.bind_for_render();
        self.call_render_shader(&mut encoder, &instance_buffer, &layers, &tex);

        self.queue.submit(Some(encoder.finish()));
        tex.present();

        let end = std::time::Instant::now();
        end - start
    }

    /// Redraws the display and populates the id buffer, returning the buffer. This is marginally faster than
    /// calling both `redraw` and `redraw_ids` individually since it only encodes the sprites once, but, it
    /// only encodes the sprites once, so the same sprites will be used for both pipelines.
    pub fn redraw_with_ids<I: IntoIterator<Item=S>,S: AsRef<Sprite>>(&self, sprites: I) -> Result<IdBuffer, wgpu::BufferAsyncError> {
        let tex = self.surface.get_current_texture().unwrap();
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let (instance_buffer, layers) = self.set_sprites(sprites);
        self.bind_for_render();

        self.call_render_shader(&mut encoder, &instance_buffer, &layers, &tex);
        self.call_id_shader(&mut encoder, &instance_buffer, &layers);
        self.read_id_texture(&mut encoder);

        self.queue.submit(Some(encoder.finish()));
        tex.present();
        self.get_sprite_ids()
    }

    /// Populates the id buffer; does not redraw the display or run the render shader. Returns the id buffer
    /// (exactly as get_sprite_ids would)
    pub fn redraw_ids<I: IntoIterator<Item=S>,S: AsRef<Sprite>>(&self, sprites: I) -> Result<IdBuffer, wgpu::BufferAsyncError> {
        let mut encoder = self.device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let (instance_buffer, layers) = self.set_sprites(sprites);
        self.bind_for_render();

        self.call_id_shader(&mut encoder, &instance_buffer, &layers);
        self.read_id_texture(&mut encoder);

        self.queue.submit(Some(encoder.finish()));
        self.get_sprite_ids()
    }

    /// Returns the buffer of which sprite id is topmost for a given pixel, and the width of
    /// that buffer.
    /// In order to easily do hit detection, alongside drawing the sprites, we run the "id shader"
    /// to create this buffer. It's a buffer, one cell per pixel, row-major format, at least as
    /// wide as the screen, with the `id` of displayed sprites in it.
    /// - Pixels with an alpha of 0 do not count as part of a sprite
    /// - Pixels not covered by a sprite have an id of 0, so, 0 is not a valid sprite id
    pub fn get_sprite_ids(&self) -> Result<IdBuffer, wgpu::BufferAsyncError> {
        let capturable = self.id_buffer.clone();
        let result: Option<Result<Vec<u32>, wgpu::BufferAsyncError>> = None;
        let m = Arc::new(std::sync::Mutex::new(result));
        let m2 = m.clone();

        self.id_buffer.slice(..).map_async(wgpu::MapMode::Read, move|result| {
            if result.is_ok() {
                let ids: Vec<u32> = bytemuck::cast_slice(&capturable.slice(..).get_mapped_range()).to_vec();
                capturable.unmap();
                let _ = m.lock().unwrap().insert(Ok(ids));
            } else {
                let _ = m.lock().unwrap().insert(Err(result.err().unwrap()));
            }
        });

        let result = loop {
            if let Some(result) = m2.lock().unwrap().take() {
                break result
            }
            self.device.poll(wgpu::Maintain::wait()).panic_on_timeout()
        };

        let screen_width = self.id_texture.size.x;
        result.map(|data| IdBuffer::new(data, Self::id_buffer_width(screen_width), screen_width))
    }
}
