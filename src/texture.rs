use cgmath::{point2, Point2};
use wgpu::{AddressMode, Device, Extent3d, FilterMode, ImageCopyTexture, ImageDataLayout, Queue, Sampler, SamplerDescriptor, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView};
use image::{DynamicImage, GenericImageView, ImageError};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub sampler: Sampler,
    pub size: Point2<u32>
}

impl Texture {
    pub fn from_bytes(device: &Device, queue: &Queue, bytes: &[u8], label: Option<&str>) -> Result<Self, ImageError> {
        let img = image::load_from_memory(bytes)?;
        Ok(Self::from_image(device, queue, &img, label))
    }

    pub fn from_image(device: &Device, queue: &Queue, img: &DynamicImage, label: Option<&str>) -> Self {
        let diffuse_rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let size = Extent3d {
            width: dimensions.0,
            height: dimensions.1,
            depth_or_array_layers: 1,
        };

        let texture = device.create_texture(&TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format: TextureFormat::Rgba8UnormSrgb,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&Default::default());

        let sampler = device.create_sampler(&SamplerDescriptor {
            address_mode_u: AddressMode::ClampToEdge,
            address_mode_v: AddressMode::ClampToEdge,
            address_mode_w: AddressMode::ClampToEdge,
            mag_filter: FilterMode::Linear,
            min_filter: FilterMode::Nearest,
            mipmap_filter: FilterMode::Nearest,
            ..Default::default()
        });

        queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            &diffuse_rgba,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some((4 * dimensions.0)),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        Self { texture, view, sampler, size: point2(dimensions.0, dimensions.1) }
    }
}