use crate::camera::{OsrsCamera, Projection};
use crate::camera_controller::CameraController;
use crate::model::{self, InstanceRaw, Model, Vertex};
use crate::player::Player;
use crate::world::World;
use anyhow::Result;
use glam::{Mat4, Vec3};
use std::sync::Arc;
use wgpu::util::DeviceExt;
use winit::event::WindowEvent;
use winit::window::Window;

#[repr(C)]
#[derive(Debug, Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
struct CameraUniform {
    view_proj: [[f32; 4]; 4],
}

impl CameraUniform {
    fn new() -> Self {
        Self {
            view_proj: Mat4::IDENTITY.to_cols_array_2d(),
        }
    }
    fn update_view_proj(&mut self, camera: &OsrsCamera, projection: &Projection) {
        self.view_proj =
            (projection.build_projection_matrix() * camera.build_view_matrix()).to_cols_array_2d();
    }
}

pub struct State {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,
    pub size: winit::dpi::PhysicalSize<u32>,
    render_pipeline: wgpu::RenderPipeline,
    camera: OsrsCamera,
    projection: Projection,
    camera_controller: CameraController,
    camera_uniform: CameraUniform,
    camera_buffer: wgpu::Buffer,
    camera_bind_group: wgpu::BindGroup,
    depth_view: wgpu::TextureView,
    player: Player,
    world: World,
    landscape_model: Model,
    player_model: Model,
    player_instance_buffer: wgpu::Buffer,
}

impl State {
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();

        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            ..Default::default()
        });
        let surface = instance.create_surface(window)?;
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions::default())
            .await
            .unwrap();
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Device"),
                    ..Default::default()
                },
            )
            .await?;

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .copied()
            .find(|f| f.is_srgb())
            .unwrap_or(surface_caps.formats[0]);
        let alpha_mode = if surface_caps
            .alpha_modes
            .contains(&wgpu::CompositeAlphaMode::Opaque)
        {
            wgpu::CompositeAlphaMode::Opaque
        } else {
            surface_caps.alpha_modes[0]
        };
        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width,
            height: size.height,
            present_mode: wgpu::PresentMode::Fifo,
            alpha_mode,
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        let world = World::new();
        let player = Player::new(Vec3::new(32.0, 0.0, 32.0));
        let camera = OsrsCamera::new(player.position);
        let projection = Projection::new(config.width, config.height, 45.0, 0.5, 500.0);
        let camera_controller = CameraController::new(2.0, 0.2);
        let mut camera_uniform = CameraUniform::new();
        camera_uniform.update_view_proj(&camera, &projection);

        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniform]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });
        let camera_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("camera_bind_group_layout"),
            });
        let camera_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &camera_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: camera_buffer.as_entire_binding(),
            }],
            label: Some("camera_bind_group"),
        });

        let depth_view = Self::create_depth_view(&device, &config);

        let texture_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[
                    wgpu::BindGroupLayoutEntry {
                        binding: 0,
                        visibility: wgpu::ShaderStages::FRAGMENT,
                        ty: wgpu::BindingType::Texture {
                            sample_type: wgpu::TextureSampleType::Float { filterable: true },
                            view_dimension: wgpu::TextureViewDimension::D2,
                            multisampled: false,
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

        let landscape_model = Model::from_heightmap(&device, &queue, &world, &texture_bind_group_layout)?;
        let player_model: Model = model::load_gltf(&device, &queue, "res/character.glb")?;

        let player_instance_data = InstanceRaw { model: Mat4::IDENTITY.to_cols_array_2d() };
        let player_instance_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Player Instance Buffer"),
                contents: bytemuck::cast_slice(&[player_instance_data]),
                usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            });

        let shader = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some("Shader"),
            source: wgpu::ShaderSource::Wgsl(include_str!("shader.wgsl").into()),
        });

        let render_pipeline_layout =
            device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
                label: Some("Render Pipeline Layout"),
                bind_group_layouts: &[&camera_bind_group_layout, &texture_bind_group_layout],
                push_constant_ranges: &[],
            });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&render_pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[Vertex::desc(), InstanceRaw::desc()],
                compilation_options: Default::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: Default::default(),
            }),
            primitive: wgpu::PrimitiveState {
                cull_mode: Some(wgpu::Face::Back),
                ..Default::default()
            },
            depth_stencil: Some(wgpu::DepthStencilState {
                format: wgpu::TextureFormat::Depth32Float,
                depth_write_enabled: true,
                depth_compare: wgpu::CompareFunction::Less,
                stencil: wgpu::StencilState::default(),
                bias: wgpu::DepthBiasState::default(),
            }),
            multisample: wgpu::MultisampleState::default(),
            multiview: None,
            cache: None,
        });

        Ok(Self {
            surface,
            device,
            queue,
            config,
            size,
            render_pipeline,
            camera,
            projection,
            camera_controller,
            camera_uniform,
            camera_buffer,
            camera_bind_group,
            depth_view,
            player,
            world,
            landscape_model,
            player_model,
            player_instance_buffer,
        })
    }

    fn create_depth_view(
        device: &wgpu::Device,
        config: &wgpu::SurfaceConfiguration,
    ) -> wgpu::TextureView {
        let depth_texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width: config.width,
                height: config.height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        });
        depth_texture.create_view(&wgpu::TextureViewDescriptor::default())
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.size = new_size;
            self.config.width = new_size.width;
            self.config.height = new_size.height;
            self.surface.configure(&self.device, &self.config);
            self.projection.resize(new_size.width, new_size.height);
            self.depth_view = Self::create_depth_view(&self.device, &self.config);
        }
    }

    pub fn size(&self) -> winit::dpi::PhysicalSize<u32> {
        self.size
    }

    pub fn input(&mut self, event: &WindowEvent) -> bool {
        self.camera_controller.process_events(event)
    }

    pub fn mouse_motion(&mut self, delta: (f64, f64)) {
        self.camera_controller.process_mouse_motion(delta.0, delta.1);
    }

    pub fn set_player_destination(&mut self, cursor_pos: winit::dpi::PhysicalPosition<f64>) {
        let ndc_x = (2.0 * cursor_pos.x as f32) / self.size.width as f32 - 1.0;
        let ndc_y = 1.0 - (2.0 * cursor_pos.y as f32) / self.size.height as f32;

        let proj_matrix = self.projection.build_projection_matrix();
        let view_matrix = self.camera.build_view_matrix();
        let inv_proj = proj_matrix.inverse();
        let inv_view = view_matrix.inverse();

        let clip_coords = glam::Vec4::new(ndc_x, ndc_y, -1.0, 1.0);
        let mut eye_coords = inv_proj * clip_coords;
        eye_coords.z = -1.0;
        eye_coords.w = 0.0;

        let world_coords_vec = inv_view * eye_coords;
        let ray_direction = world_coords_vec.truncate().normalize();
        let ray_origin = self.camera.eye_position();

        if ray_direction.y.abs() > 0.001 {
            let t = (0.0 - ray_origin.y) / ray_direction.y;
            if t > 0.0 {
                let intersection_point = ray_origin + t * ray_direction;
                self.player.target_position = Some(intersection_point);
            }
        }
    }

    pub fn update(&mut self) {
        if let Some(target) = self.player.target_position {
            let direction = target - self.player.position;
            let distance = direction.length();
            let speed = 0.2;

            if distance < speed {
                self.player.position = target;
                self.player.target_position = None;
            } else {
                self.player.position += direction.normalize() * speed;
            }
        }

        let player_x = self.player.position.x;
        let player_z = self.player.position.z;
        self.player.position.y = self.world.get_height(player_x, player_z);

        self.camera.focus_point = self.player.position;
        self.camera_controller.update_camera(&mut self.camera);
        self.camera_uniform
            .update_view_proj(&self.camera, &self.projection);
        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniform]),
        );
    }

    pub fn render(&mut self) -> Result<(), wgpu::SurfaceError> {
        let output = self.surface.get_current_texture()?;
        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder =
            self.device
                .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                    label: Some("Render Encoder"),
                });
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color {
                            r: 0.1,
                            g: 0.2,
                            b: 0.3,
                            a: 1.0,
                        }),
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);
            render_pass.set_bind_group(0, &self.camera_bind_group, &[]);

            let landscape_matrix = Mat4::IDENTITY;
            let landscape_instance_data = InstanceRaw {
                model: landscape_matrix.to_cols_array_2d(),
            };
            let landscape_instance_buffer =
                self.device
                    .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                        label: Some("Landscape Instance Buffer"),
                        contents: bytemuck::cast_slice(&[landscape_instance_data]),
                        usage: wgpu::BufferUsages::VERTEX,
                    });
            render_pass.set_vertex_buffer(1, landscape_instance_buffer.slice(..));
            
            for mesh in &self.landscape_model.meshes {
                let material = &self.landscape_model.materials[mesh.material_index];
                render_pass.set_bind_group(1, &material.bind_group, &[]);
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint16);
                render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
            }

            let scale = Mat4::from_scale(Vec3::splat(0.01));
            let translation = Mat4::from_translation(self.player.position);
            let rotation = Mat4::from_rotation_x(std::f32::consts::FRAC_PI_2);
            let player_model_matrix = translation * rotation * scale;
            
            let player_instance_data = InstanceRaw {
                model: player_model_matrix.to_cols_array_2d(),
            };
            
            self.queue.write_buffer(
                &self.player_instance_buffer,
                0,
                bytemuck::cast_slice(&[player_instance_data]),
            );

            render_pass.set_vertex_buffer(1, self.player_instance_buffer.slice(..));

            for mesh in &self.player_model.meshes {
                if !self.player_model.materials.is_empty() {
                    let material = &self.player_model.materials[mesh.material_index];
                    render_pass.set_bind_group(1, &material.bind_group, &[]);
                }
                render_pass.set_vertex_buffer(0, mesh.vertex_buffer.slice(..));
                render_pass.set_index_buffer(mesh.index_buffer.slice(..), wgpu::IndexFormat::Uint32);
                render_pass.draw_indexed(0..mesh.num_indices, 0, 0..1);
            }
        }
        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
        Ok(())
    }
}