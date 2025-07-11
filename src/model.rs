use anyhow::Result;
use std::path::Path;
use wgpu::util::DeviceExt;
use crate::world::WORLD_SIZE;
use crate::world::World;


pub mod texture {
    use super::*;
    use image::GenericImageView;
    use wgpu::util::DeviceExt; 
    pub fn from_bytes(
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        bytes: &[u8],
        label: &str,
    ) -> Result<(wgpu::Texture, wgpu::TextureView, wgpu::Sampler)> {
        let img = image::load_from_memory(bytes)?;
        let rgba = img.to_rgba8();
        let dimensions = img.dimensions();

        let texture = device.create_texture_with_data(
            queue,
            &wgpu::TextureDescriptor {
                label: Some(label),
                size: wgpu::Extent3d {
                    width: dimensions.0,
                    height: dimensions.1,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING, 
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            &rgba,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        let sampler = device.create_sampler(&wgpu::SamplerDescriptor {
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        Ok((texture, view, sampler))
    }
}

#[repr(C)]
#[derive(Copy, Clone, Debug, bytemuck::Pod, bytemuck::Zeroable)]
pub struct Vertex {
    pub position: [f32; 3],
    pub tex_coords: [f32; 2],
    pub normal: [f32; 3],
}

impl Vertex {
    const ATTRIBS: [wgpu::VertexAttribute; 3] =
        wgpu::vertex_attr_array![0 => Float32x3, 1 => Float32x2, 2 => Float32x3];
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        wgpu::VertexBufferLayout {
            array_stride: std::mem::size_of::<Vertex>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Vertex,
            attributes: &Self::ATTRIBS,
        }
    }
}

#[repr(C)]
#[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
pub struct InstanceRaw {
    pub model: [[f32; 4]; 4],
}

impl InstanceRaw {
    pub fn desc() -> wgpu::VertexBufferLayout<'static> {
        use std::mem;
        const ATTRIBS: [wgpu::VertexAttribute; 4] =
            wgpu::vertex_attr_array![5 => Float32x4, 6 => Float32x4, 7 => Float32x4, 8 => Float32x4];
        wgpu::VertexBufferLayout {
            array_stride: mem::size_of::<InstanceRaw>() as wgpu::BufferAddress,
            step_mode: wgpu::VertexStepMode::Instance,
            attributes: &ATTRIBS,
        }
    }
}

pub struct Material {
    pub name: String,
    pub diffuse_texture_view: wgpu::TextureView,
    pub diffuse_sampler: wgpu::Sampler,
    pub bind_group: wgpu::BindGroup,
}

pub struct Mesh {
    pub name: String,
    pub vertex_buffer: wgpu::Buffer,
    pub index_buffer: wgpu::Buffer,
    pub num_indices: u32,
    pub material_index: usize,
}
pub trait Drawable<'a> {
    fn draw_model(&mut self, model: &'a Model, instance_buffer: &'a wgpu::Buffer, instances: u32);
}

impl<'a, 'b> Drawable<'a> for wgpu::RenderPass<'b> where 'a: 'b {
    fn draw_model(&mut self, model: &'a Model, instance_buffer: &'a wgpu::Buffer, instances: u32) {
        self.set_vertex_buffer(1, instance_buffer.slice(..));
        for mesh in &model.meshes {
            let material = &model.materials[mesh.material_index];
            self.set_bind_group(1, &material.bind_group, &[]);
            self.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
            self.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
            self.draw_indexed(0..mesh.num_indices, 0, 0..instances);
        }
    }
}

pub struct Model {
    pub meshes: Vec<Mesh>,
    pub materials: Vec<Material>,
}

impl Model {
    pub fn from_heightmap(device: &wgpu::Device, queue: &wgpu::Queue, world: &World, material_bind_group_layout: &wgpu::BindGroupLayout) -> Result<Self> {
        let mut vertices = Vec::new();
        let mut indices = Vec::new();
        let mut current_index: u16 = 0;

        for z in 0..(WORLD_SIZE - 1) {
            for x in 0..(WORLD_SIZE - 1) {
                let y_tl = world.heightmap[x][z];
                let y_tr = world.heightmap[x + 1][z];
                let y_bl = world.heightmap[x][z + 1];
                let y_br = world.heightmap[x + 1][z + 1];

                let top_left  = Vertex { position: [x as f32, y_tl, z as f32], tex_coords: [0.0, 0.0], normal: [0.0, 1.0, 0.0] };
                let top_right = Vertex { position: [(x + 1) as f32, y_tr, z as f32], tex_coords: [1.0, 0.0], normal: [0.0, 1.0, 0.0] };
                let bottom_left = Vertex { position: [x as f32, y_bl, (z + 1) as f32], tex_coords: [0.0, 1.0], normal: [0.0, 1.0, 0.0] };
                let bottom_right= Vertex { position: [(x + 1) as f32, y_br, (z + 1) as f32], tex_coords: [1.0, 1.0], normal: [0.0, 1.0, 0.0] };

                vertices.extend_from_slice(&[top_left, top_right, bottom_left, bottom_right]);

                indices.push(current_index);
                indices.push(current_index + 2);
                indices.push(current_index + 1);
                indices.push(current_index + 1);
                indices.push(current_index + 2);
                indices.push(current_index + 3);
                current_index += 4;
            }
        }

        let vertex_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Landscape Vertex Buffer"),
            contents: bytemuck::cast_slice(&vertices),
            usage: wgpu::BufferUsages::VERTEX,
        });

        let index_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Landscape Index Buffer"),
            contents: bytemuck::cast_slice(&indices),
            usage: wgpu::BufferUsages::INDEX,
        });

        let landscape_mesh = Mesh {
            name: "landscape".to_string(),
            vertex_buffer,
            index_buffer,
            num_indices: indices.len() as u32,
            material_index: 0,
        };

        let diffuse_bytes = include_bytes!("../res/stone.png");
        let (_texture, view, sampler) = texture::from_bytes(device, queue, diffuse_bytes, "stone.png")?;
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: material_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry { binding: 0, resource: wgpu::BindingResource::TextureView(&view) },
                wgpu::BindGroupEntry { binding: 1, resource: wgpu::BindingResource::Sampler(&sampler) },
            ],
            label: Some("landscape_material_bind_group"),
        });

        let landscape_material = Material {
            name: "stone".to_string(),
            diffuse_texture_view: view,
            diffuse_sampler: sampler,
            bind_group,
        };

        Ok(Self {
            meshes: vec![landscape_mesh],
            materials: vec![landscape_material],
        })
    }
}

pub fn load_gltf<P: AsRef<Path>>(
    device: &wgpu::Device,
    queue: &wgpu::Queue,
    path: P,
) -> Result<Model> {
    let (doc, buffers, images) = gltf::import(path.as_ref())?;

    let texture_bind_group_layout =
        device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
            entries: &[
                wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Texture {
                        multisampled: false,
                        view_dimension: wgpu::TextureViewDimension::D2,
                        sample_type: wgpu::TextureSampleType::Float { filterable: true },
                    },
                    count: None,
                },
                wgpu::BindGroupLayoutEntry {
                    binding: 1,
                    visibility: wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Sampler(wgpu::SamplerBindingType::Filtering),
                    count: None,
                },
            ],
            label: Some("texture_bind_group_layout"),
        });

    let white_pixel = [255, 255, 255, 255];
    let fallback_texture = device.create_texture_with_data(
        queue,
        &wgpu::TextureDescriptor {
            size: wgpu::Extent3d { width: 1, height: 1, depth_or_array_layers: 1 },
            mip_level_count: 1, sample_count: 1, dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Rgba8UnormSrgb,
            usage: wgpu::TextureUsages::TEXTURE_BINDING,
            label: Some("fallback_white_texture"), view_formats: &[],
        },
        wgpu::util::TextureDataOrder::LayerMajor,
        &white_pixel,
    );
    let fallback_view = fallback_texture.create_view(&wgpu::TextureViewDescriptor::default());
    let fallback_sampler = device.create_sampler(&wgpu::SamplerDescriptor {
        mag_filter: wgpu::FilterMode::Nearest,
        ..Default::default()
    });

    let mut materials = Vec::new();
    for material in doc.materials() {
        let pbr = material.pbr_metallic_roughness();

        let (view, sampler) = if let Some(texture_info) = pbr.base_color_texture() {
            let texture_source = &doc.textures().nth(texture_info.texture().index()).unwrap();
            let image = &images[texture_source.source().index()];
            let (_texture, view, sampler) = texture::from_bytes(device, queue, &image.pixels, "gltf_texture")?;
            (view, sampler)
        } else {

            (fallback_view.clone(), fallback_sampler.clone())
        };

        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&sampler),
                },
            ],
            label: Some("material_bind_group"),
        });

        materials.push(Material {
            name: material.name().unwrap_or_default().to_string(),
            diffuse_texture_view: view,
            diffuse_sampler: sampler,
            bind_group,
        });
    }

    if materials.is_empty() {
        let bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &texture_bind_group_layout,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(&fallback_view),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::Sampler(&fallback_sampler),
                },
            ],
            label: Some("fallback_material_bind_group"),
        });
        materials.push(Material {
            name: "fallback_material".to_string(),
            diffuse_texture_view: fallback_view,
            diffuse_sampler: fallback_sampler,
            bind_group,
        });
    }

    let mut meshes = Vec::new();
    for scene in doc.scenes() {
        for node in scene.nodes() {
            if let Some(mesh) = node.mesh() {
                for primitive in mesh.primitives() {
                    let reader = primitive.reader(|buffer| Some(&buffers[buffer.index()]));

                    let positions: Vec<[f32; 3]> = reader.read_positions().unwrap().collect();
                    let normals: Vec<[f32; 3]> = reader.read_normals().unwrap().collect();
                    
                    let tex_coords: Vec<[f32; 2]> = match reader.read_tex_coords(0) {
                        Some(coords) => coords.into_f32().collect(),
                        None => vec![[0.0, 0.0]; positions.len()],
                    };

                    let vertices: Vec<Vertex> = positions.iter().zip(normals.iter()).zip(tex_coords.iter())
                        .map(|((pos, norm), tc)| Vertex {
                            position: *pos,
                            tex_coords: *tc,
                            normal: *norm,
                        })
                        .collect();

                    let vertex_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("GLTF Vertex Buffer"),
                            contents: bytemuck::cast_slice(&vertices),
                            usage: wgpu::BufferUsages::VERTEX,
                        });

                    let indices: Vec<u32> = reader.read_indices().unwrap().into_u32().collect();
                    let index_buffer =
                        device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("GLTF Index Buffer"),
                            contents: bytemuck::cast_slice(&indices),
                            usage: wgpu::BufferUsages::INDEX,
                        });

                    meshes.push(Mesh {
                        name: mesh.name().unwrap_or_default().to_string(),
                        vertex_buffer,
                        index_buffer,
                        num_indices: indices.len() as u32,
                        material_index: primitive.material().index().unwrap_or(0),
                    });
                }
            }
        }
    }

    Ok(Model { meshes, materials })
}