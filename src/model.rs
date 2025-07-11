use wgpu::util::DeviceExt;

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    position: [f32; 3],
    tex_coords: [f32; 2],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 2] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2];

    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

pub struct Model {
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
}

impl Model {
    pub fn new_cube(device: &wgpu::Device) -> Self {
        const VERTICES: &[Vertex] = &[

            Vertex { position: [-0.5, -0.5, 0.5], tex_coords: [0.0, 1.0] },
            Vertex { position: [0.5, -0.5, 0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [0.5, 0.5, 0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [-0.5, 0.5, 0.5], tex_coords: [0.0, 0.0] },

            Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [-0.5, 0.5, -0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [0.5, 0.5, -0.5], tex_coords: [0.0, 0.0] },
            Vertex { position: [0.5, -0.5, -0.5], tex_coords: [0.0, 1.0] },

            Vertex { position: [-0.5, 0.5, -0.5], tex_coords: [0.0, 1.0] },
            Vertex { position: [-0.5, 0.5, 0.5], tex_coords: [0.0, 0.0] },
            Vertex { position: [0.5, 0.5, 0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [0.5, 0.5, -0.5], tex_coords: [1.0, 1.0] },

            Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [0.5, -0.5, -0.5], tex_coords: [0.0, 1.0] },
            Vertex { position: [0.5, -0.5, 0.5], tex_coords: [0.0, 0.0] },
            Vertex { position: [-0.5, -0.5, 0.5], tex_coords: [1.0, 0.0] },

            Vertex { position: [0.5, -0.5, -0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [0.5, 0.5, -0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [0.5, 0.5, 0.5], tex_coords: [0.0, 0.0] },
            Vertex { position: [0.5, -0.5, 0.5], tex_coords: [0.0, 1.0] },

            Vertex { position: [-0.5, -0.5, -0.5], tex_coords: [0.0, 1.0] },
            Vertex { position: [-0.5, -0.5, 0.5], tex_coords: [1.0, 1.0] },
            Vertex { position: [-0.5, 0.5, 0.5], tex_coords: [1.0, 0.0] },
            Vertex { position: [-0.5, 0.5, -0.5], tex_coords: [0.0, 0.0] },
        ];

        const INDICES: &[u16] = &[
             0,  1,  2,  2,  3,  0, 
             4,  5,  6,  6,  7,  4, 
             8,  9, 10, 10, 11,  8, 
            12, 13, 14, 14, 15, 12, 
            16, 17, 18, 18, 19, 16, 
            20, 21, 22, 22, 23, 20, 
        ];

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Vertex Buffer"),
            contents: bytemuck::cast_slice(VERTICES),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Index Buffer"),
            contents: bytemuck::cast_slice(INDICES),
            usage: wgpu::BufferUsages::INDEX,
        });

        Self {
            vertex_buffer,
            index_buffer,
            num_indices: INDICES.len() as u32,
        }
    }
}