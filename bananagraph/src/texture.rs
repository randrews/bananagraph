use cgmath::Vector2;
use wgpu::{Device, Extent3d, ImageCopyTexture, ImageDataLayout, Queue, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages, TextureView};
use image::{GenericImageView, ImageError, RgbaImage};

pub struct Texture {
    pub texture: wgpu::Texture,
    pub view: TextureView,
    pub size: Vector2<u32>
}

impl Texture {
    pub fn from_bytes(device: &Device, queue: &Queue, bytes: &[u8], label: Option<&str>) -> Result<Self, ImageError> {
        let img = image::load_from_memory(bytes)?;
        Ok(Self::from_image(device, queue, &img.to_rgba8(), label))
    }

    pub fn from_array(device: &Device, queue: &Queue, bytes: Vec<u8>, width: u32, label: Option<&str>) -> Result<Self, ImageError> {
        let img: RgbaImage = RgbaImage::from_raw(width, bytes.len() as u32 / 4 / width, bytes).unwrap();
        Ok(Self::from_image(device, queue, &img, label))
    }

    pub fn from_image(device: &Device, queue: &Queue, img: &RgbaImage, label: Option<&str>) -> Self {
        let diffuse_rgba = img;
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
            format: TextureFormat::Rgba8Unorm,
            usage: TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_DST,
            view_formats: &[],
        });

        let view = texture.create_view(&Default::default());

        queue.write_texture(
            ImageCopyTexture {
                texture: &texture,
                mip_level: 0,
                origin: wgpu::Origin3d::ZERO,
                aspect: wgpu::TextureAspect::All,
            },
            diffuse_rgba,
            ImageDataLayout {
                offset: 0,
                bytes_per_row: Some(4 * dimensions.0),
                rows_per_image: Some(dimensions.1),
            },
            size,
        );

        Self { texture, view, size: Vector2::new(dimensions.0, dimensions.1) }
    }

    /// Create a texture the size of the surface, with a given format and label
    pub fn generic_texture(device: &Device, config: &wgpu::SurfaceConfiguration, label: Option<&str>, format: TextureFormat, usage: TextureUsages) -> Self {
        let size = Extent3d {
            width: config.width.max(1),
            height: config.height.max(1),
            depth_or_array_layers: 1,
        };

        let desc = TextureDescriptor {
            label,
            size,
            mip_level_count: 1,
            sample_count: 1,
            dimension: TextureDimension::D2,
            format,
            usage,
            view_formats: &[],
        };
        let texture = device.create_texture(&desc);

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        Self { texture, view, size: (size.width, size.height).into() }
    }

    /// Create a texture suitable for use as a depth texture
    pub fn create_depth_texture(device: &Device, config: &wgpu::SurfaceConfiguration) -> Self {
        Self::generic_texture(device, config, Some("depth texture"), TextureFormat::Depth32Float, TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING)
    }

    /// Create a texture for the ID shader to use as its output
    pub fn create_id_texture(device: &Device, config: &wgpu::SurfaceConfiguration) -> Self {
        Self::generic_texture(device, config, Some("id texture"), TextureFormat::R32Uint, TextureUsages::RENDER_ATTACHMENT | TextureUsages::TEXTURE_BINDING | TextureUsages::COPY_SRC)
    }
}