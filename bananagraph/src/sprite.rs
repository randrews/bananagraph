use cgmath::{ElementWise, Matrix3, Point2, Rad, SquareMatrix, Vector2, Vector4};

pub type SpriteId = u32;

/// A `Sprite` is the basic unit of drawing to the screen. We create a list of sprites and pass them
/// to a `GpuWrapper` to render.
/// 
/// Each sprite has a rectangular region of the source texture (see `Layer`) and a transformation
/// matrix that places it somewhere on the screen.
/// ```
/// # use bananagraph::{ Sprite };
/// # use cgmath::Deg;
/// let s = Sprite::new((100, 100), (16, 16)).translate((0.5, 0.5)).rotate(Deg(45.0));
/// ```
#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Sprite {
    transform: Matrix3<f32>,
    pub(crate) z: f32,
    pub size: Vector2<u32>,
    origin: Point2<u32>,
    pub(crate) layer: u32,
    tint: Vector4<f32>,
    pub id: SpriteId
}

#[derive(Copy, Clone, PartialEq, Debug, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub(crate) struct RawSprite {
    transform_i: [f32; 3],
    transform_j: [f32; 3],
    transform_k: [f32; 3],
    origin: [f32; 2],
    size: [f32; 2],
    z: f32,
    id: u32,
    tint: [f32; 4]
}

impl Sprite {
    /// Create a Sprite drawn from the given spritesheet, with a given origin and size in that spritesheet
    pub fn new(origin: impl Into<Point2<u32>>, size: impl Into<Vector2<u32>>) -> Self {
        Self {
            z: 0.0,
            layer: 0,
            transform: Matrix3::identity(),
            origin: origin.into(),
            size: size.into(),
            tint: (1.0, 1.0, 1.0, 1.0).into(),
            id: 0
        }
    }

    /// Convert a sprite into a `RawSprite` which can be loaded into an instance buffer and sent to the GPU
    pub(crate) fn into_raw(self, texture_size: impl Into<Vector2<u32>>) -> RawSprite {
        let [transform_i, transform_j, transform_k] = self.transform.into();
        let fsize = Point2::new(self.size.x as f32, self.size.y as f32);
        let forigin = Point2::new(self.origin.x as f32, self.origin.y as f32);
        let texture_size: Vector2<u32> = texture_size.into();
        let ftsize = Point2::new(texture_size.x as f32, texture_size.y as f32);

        let origin: [f32; 2] = forigin.div_element_wise(ftsize).into();
        let size: [f32; 2] = fsize.div_element_wise(ftsize).into();

        RawSprite {
            transform_i,
            transform_j,
            transform_k,
            origin,
            size,
            z: self.z,
            id: self.id,
            tint: self.tint.into(),
        }
    }

    /// Return a sprite with the transform matrix scaled by these factors
    pub fn scale(self, factor: impl Into<Vector2<f32>>) -> Self {
        let factor = factor.into();
        // For some reason cgmath doesn't have a helper for nonuniform scaling?
        let scale = Matrix3::new(
            factor.x, 0.0, 0.0,
            0.0, factor.y, 0.0,
            0.0, 0.0, 1.0
        );

        Self {
            transform: scale * self.transform,
            ..self
        }
    }

    /// Return a sprite with the transform matrix scaled by the reciprocal of these factors
    pub fn inv_scale(self, reciprocal: impl Into<Vector2<f32>>) -> Self {
        let reciprocal = reciprocal.into();
        self.scale((1.0 / reciprocal.x, 1.0 / reciprocal.y))
    }

    /// Return a sprite with the transform matrix translated by this vector
    pub fn translate(self, delta: impl Into<Vector2<f32>>) -> Self {
        Self {
            transform: Matrix3::from_translation(delta.into()) * self.transform,
            ..self
        }
    }

    /// Return a sprite with the transform matrix scaled by the size of the sprite (in pixels
    /// from the texture)
    pub fn size_scale(self) -> Self {
        self.scale((self.size.x as f32, self.size.y as f32))
    }

    /// Return a sprite with the transform matrix scaled by the reciprocal of the size of the
    /// sprite (in pixels from the texture)
    pub fn inv_size_scale(self) -> Self {
        self.inv_scale((self.size.x as f32, self.size.y as f32))
    }

    pub fn rotate(self, theta: impl Into<Rad<f32>>) -> Self {
        Self {
            transform: Matrix3::from_angle_z(theta) * self.transform,
            ..self
        }
    }

    /// Return a sprite with the given transform matrix (to set the matrix manually)
    pub fn with_transform(self, transform: impl Into<Matrix3<f32>>) -> Self {
        Self {
            transform: transform.into(),
            ..self
        }
    }

    /// Return a sprite with the given Z index (between 0.0 and 1.0 inclusive, with
    /// 0.0 being closest to the top and 1.0 being closest to the bottom)
    pub fn with_z(self, z: f32) -> Self {
        Self { z, ..self }
    }

    /// Change the tint of the sprite, which is an RGBA vec4 that each pixel is multiplied
    /// by on display. This is useful for fading, coloring, whatever, on a sprite without
    /// changing the spritesheet
    pub fn with_tint(self, tint: impl Into<Vector4<f32>>) -> Self {
        Self {
            tint: tint.into(),
            ..self
        }
    }

    /// Sprites can be given ids for hit detection, see `GpuWrapper::get_sprite_ids`
    pub fn with_id(self, id: SpriteId) -> Self {
        Self {
            id,
            ..self
        }
    }
    
    /// Returns a sprite with the given layer
    pub fn with_layer(self, layer: u32) -> Self {
        Self {
            layer,
            ..self
        }
    }

    /// Returns a sprite that's been positioned at the given coordinates, in a "screen" space that's
    /// the given dimensions. This is the normal way to draw a sprite to the window; if you give every
    /// sprite the same screen size then you can just treat the positions as pixel coordinates in that screen.
    pub fn with_position(self, pos: impl Into<Vector2<f32>>, screen: impl Into<Vector2<f32>>) -> Self {
        let pos = pos.into();
        let screen = screen.into();
        self
            .scale((self.size.x as f32 / screen.x, self.size.y as f32 / screen.y))
            .translate((pos.x / screen.x, pos.y / screen.y))
    }
}

impl AsRef<Sprite> for Sprite {
    fn as_ref(&self) -> &Sprite {
        &self
    }
}

impl RawSprite {
    pub(crate) fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Self>() as wgpu::BufferAddress,
            // We need to switch from using a step mode of Vertex to Instance
            // This means that our shaders will only change to use the next
            // instance when the shader starts processing a new instance
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &[
                wgpu::VertexAttribute {
                    offset: 0,
                    shader_location: 1,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 12,
                    shader_location: 2,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 24,
                    shader_location: 3,
                    format: wgpu::VertexFormat::Float32x3,
                },
                wgpu::VertexAttribute {
                    offset: 36,
                    shader_location: 4,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 44,
                    shader_location: 5,
                    format: wgpu::VertexFormat::Float32x2,
                },
                wgpu::VertexAttribute {
                    offset: 52,
                    shader_location: 6,
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: 56,
                    shader_location: 7,
                    format: wgpu::VertexFormat::Uint32,
                },
                wgpu::VertexAttribute {
                    offset: 60,
                    shader_location: 8,
                    format: wgpu::VertexFormat::Float32x4,
                },
        ]
        }
    }
}