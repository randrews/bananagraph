use cgmath::{ElementWise, Matrix3, Point2, SquareMatrix};

#[derive(Copy, Clone, PartialEq, Debug)]
#[repr(C)]
pub struct Sprite {
    transform: Matrix3<f32>,
    size: Point2<u32>,
    origin: Point2<u32>,
    texture_size: Point2<u32>
}

#[derive(Copy, Clone, PartialEq, Debug, bytemuck::Zeroable, bytemuck::Pod)]
#[repr(C)]
pub struct RawSprite {
    transform_i: [f32; 3],
    transform_j: [f32; 3],
    transform_k: [f32; 3],
    origin: [f32; 2],
    size: [f32; 2]
}

impl From<&Sprite> for RawSprite {
    fn from(value: &Sprite) -> Self {
        let [transform_i, transform_j, transform_k] = value.transform.into();
        let fsize = Point2::new(value.size.x as f32, value.size.y as f32);
        let forigin = Point2::new(value.origin.x as f32, value.origin.y as f32);
        let ftsize = Point2::new(value.texture_size.x as f32, value.texture_size.y as f32);

        let origin: [f32; 2] = forigin.div_element_wise(ftsize).into();
        let size: [f32; 2] = fsize.div_element_wise(ftsize).into();

        Self {
            transform_i,
            transform_j,
            transform_k,
            origin,
            size
        }
    }
}

impl Sprite {
    pub fn new(origin: Point2<u32>, size: Point2<u32>, texture_size: Point2<u32>) -> Self {
        Self {
            transform: Matrix3::identity(),
            origin,
            size,
            texture_size
        }
    }
}

impl RawSprite {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: size_of::<Sprite>() as wgpu::BufferAddress,
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
            ]
        }
    }
}