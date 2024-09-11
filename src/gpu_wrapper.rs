use wgpu::{BindGroupEntry, ComputePipeline, Device, Features, Instance, InstanceDescriptor, Queue, RequestAdapterOptions, Surface, SurfaceConfiguration, TextureFormat, TextureUsages, TextureViewDescriptor};
use wgpu::BindingResource::TextureView;
use winit::window::Window;

pub struct GpuWrapper<'a> {
    device: Device,
    queue: Queue,
    pipeline: ComputePipeline,
    window: &'a Window,
    surface: Surface<'a>
}

impl<'a> GpuWrapper<'a> {
    pub async fn new(window: &'a Window) -> Self {
        let size = window.inner_size();
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

        let config = SurfaceConfiguration {
            usage: TextureUsages::RENDER_ATTACHMENT | TextureUsages::STORAGE_BINDING,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: surface_caps.present_modes[0],
            desired_maximum_frame_latency: 2,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
        };

        surface.configure(&device, &config);

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: None,
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let pipeline = device.create_compute_pipeline(&wgpu::ComputePipelineDescriptor {
            label: None,
            layout: None,
            module: &shader,
            entry_point: "pixel_shader",
            compilation_options: Default::default(),
            cache: None,
        });

        Self {
            device,
            queue,
            pipeline,
            window,
            surface
        }
    }

    pub fn call_shader(&self) {
        let bind_group_layout = self.pipeline.get_bind_group_layout(0);
        let tex = self.surface.get_current_texture().unwrap();
        let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: None,
            layout: &bind_group_layout,
            entries: &[
                BindGroupEntry {
                    binding: 0,
                    resource: TextureView(&tex.texture.create_view(&TextureViewDescriptor::default())),
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
            cpass.dispatch_workgroups(self.window.inner_size().width, self.window.inner_size().height, 1);
        }

        self.queue.submit(Some(encoder.finish()));
        tex.present();
    }
}