use cgmath::{Deg, ElementWise, Matrix3, Point2, Rad, SquareMatrix, Vector2};

pub type SpriteId = u32;

/// The five spritesheets we can draw sprites from, and their intended uses
#[derive(Copy, Clone, PartialEq, Debug)]
pub enum Layer {
    /// The `Sprite` sheet is for general objects in the world; if you only have one sheet use this one
    Sprite,
    
    /// The `Terrain` sheet is for terrain tiles
    Terrain,

    /// `Mob`s often have more animation frames, so we reserve an entire sheet for them
    Mob,

    /// The `Background` sheet is for larger images to be used as backgrounds, skyboxes, etc.
    Background,

    /// To display text, it's convenient to have one sheet set aside for a bitmap `Font`
    Font,
}

#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Sprite {
    pub transform: Matrix3<f32>,
    pub z: f32,
    size: Vector2<u32>,
    origin: Point2<u32>,
    layer: Layer,
    override_alpha: Option<f32>,
    id: SpriteId
}

#[derive(Copy, Clone, PartialEq, Debug, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub struct RawSprite {
    transform_i: [f32; 3],
    transform_j: [f32; 3],
    transform_k: [f32; 3],
    origin: [f32; 2],
    size: [f32; 2],
    z: f32,
    id: u32,
    override_alpha: f32,
    is_override_alpha: u32
}

impl Sprite {
    pub fn new(layer: Layer, origin: impl Into<Point2<u32>>, size: impl Into<Vector2<u32>>) -> Self {
        Self {
            z: 0.0,
            layer,
            transform: Matrix3::identity(),
            origin: origin.into(),
            size: size.into(),
            override_alpha: None,
            id: 0
        }
    }

    pub fn into_raw(self, texture_size: impl Into<Vector2<u32>>) -> RawSprite {
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
            override_alpha: self.override_alpha.unwrap_or(0.0),
            is_override_alpha: self.override_alpha.map_or(0, |_| 1)
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

    pub fn with_transform(self, transform: impl Into<Matrix3<f32>>) -> Self {
        Self {
            transform: transform.into(),
            ..self
        }
    }

    pub fn with_z(self, z: f32) -> Self {
        Self { z, ..self }
    }
    
    pub fn with_override_alpha(self, override_alpha: Option<f32>) -> Self {
        Self {
            override_alpha,
            ..self
        }
    }

    pub fn with_id(self, id: SpriteId) -> Self {
        Self {
            id,
            ..self
        }
    }
}

impl RawSprite {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
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
                    format: wgpu::VertexFormat::Float32,
                },
                wgpu::VertexAttribute {
                    offset: 64,
                    shader_location: 9,
                    format: wgpu::VertexFormat::Uint32,
                }
        ]
        }
    }
}