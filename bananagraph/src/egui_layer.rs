use std::num::NonZero;
use cgmath::Vector2;
use egui::{pos2, Rect};
use wgpu::{Color, Device, LoadOp, Queue, TextureFormat, TextureUsages};

pub struct EguiLayer {
    context: egui::Context,
    renderer: egui_wgpu::Renderer,
    texture: crate::texture::Texture
}

impl EguiLayer {
    pub fn new(device: &Device, size: Vector2<u32>, output_format: TextureFormat) -> Self {
        let mut context = egui::Context::default();
        context.options_mut(|opts| {
            opts.max_passes = NonZero::new(2usize).unwrap();
        });
        context.set_visuals(egui::Visuals::default());
        //let egui_state = egui::State::new(egui_context.clone(), id, &window, None, None);
        let renderer = egui_wgpu::Renderer::new(device, output_format, None, 1, false);

        let texture = crate::texture::Texture::generic_texture(device, size, Some("egui texture"), output_format, TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC);

        Self {
            context,
            renderer,
            texture
        }
    }

    pub fn render(&mut self, device: &Device, queue: &Queue) {
        let mut encoder = device.create_command_encoder(&wgpu::CommandEncoderDescriptor::default());

        let size = self.texture.size;
        println!("{:?}", size);
        let raw_input = egui::RawInput {
            screen_rect: Some(Rect::from_min_max(pos2(0.0, 0.0), pos2(size.x as f32, size.y as f32))),
            ..Default::default()
        };
        let full_output = self.context.run(raw_input, |ctx| {
            println!("Running pass, {:?}", ctx.screen_rect());
            egui::Window::new("My Window").show(ctx, |ui| {
                ui.label("Hello World!");
            });
            // egui::CentralPanel::default().show(&ctx, |ui| {
            //     ui.label("Hello world!");
            //     if ui.button("Click me").clicked() {
            //         println!("you clicked it")
            //     }
            // });
        });
        //handle_platform_output(full_output.platform_output);

        let screen_descriptor = egui_wgpu::ScreenDescriptor {
            size_in_pixels: [size.x, size.y],
            pixels_per_point: 1.0,
        };
        let tris = self.context.tessellate(full_output.shapes, full_output.pixels_per_point);

        for (id, image_delta) in &full_output.textures_delta.set {
            self.renderer.update_texture(&device, &queue, *id, &image_delta);
        }

        self.renderer.update_buffers(&device, &queue, &mut encoder, &tris, &screen_descriptor);
        let mut rpass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.texture.texture.create_view(&Default::default()),
                resolve_target: None,
                ops: wgpu::Operations {
                    load: LoadOp::Clear(Color::BLACK),
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            label: Some("egui main render pass"),
            timestamp_writes: None,
            occlusion_query_set: None,
        }).forget_lifetime();
        self.renderer.render(&mut rpass, tris.as_slice(), &screen_descriptor);
        for x in &full_output.textures_delta.free {
            self.renderer.free_texture(x)
        }
        drop(rpass);
        queue.submit(Some(encoder.finish()));
    }

    pub fn texture(&self) -> &crate::texture::Texture {
        &self.texture
    }
}