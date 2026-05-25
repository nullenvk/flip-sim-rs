use bytemuck::{Pod, Zeroable};
use std::sync::Arc;
use wgpu::util::DeviceExt;

use crate::simulation::Simulation;
use crate::simulation::config::*;

// #[repr(C)] for bytemuck, reading in shader
#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct ParticleInstance {
    pub position: [f32; 2],
    pub color: [f32; 3],
}

#[repr(C)]
#[derive(Copy, Clone, Debug, Pod, Zeroable)]
pub struct Uniforms {
    pub scale: [f32; 2],
    pub is_particle: u32,
    pub _padding: u32, // 16-byte alignment
}

pub struct FrontWgpu {
    _window: Arc<winit::window::Window>, // workaround z tutoriala
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surf_config: wgpu::SurfaceConfiguration,
    phys_size: winit::dpi::PhysicalSize<u32>,

    render_pipeline: wgpu::RenderPipeline,

    particle_buffer: wgpu::Buffer,
    particle_bind_group: wgpu::BindGroup,

    grid_buffer: wgpu::Buffer,
    grid_bind_group: wgpu::BindGroup,

    grid_instances: Vec<ParticleInstance>,

    pub sim: Simulation,
    pub runtime_config: RuntimeConfig,
}

impl FrontWgpu {
    pub async fn new(
        window: Arc<winit::window::Window>,
        sim: Simulation,
        runtime_config: RuntimeConfig,
    ) -> Self {
        let phys_size = window.inner_size();

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(&window)).unwrap();
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .unwrap();

        let (device, queue) = adapter
            .request_device(&wgpu::DeviceDescriptor::default())
            .await
            .unwrap();

        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps.formats[0];
        let surf_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: phys_size.width,
            height: phys_size.height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &surf_config);

        let grid_buffer_size =
            (sim.f_num_cells * std::mem::size_of::<ParticleInstance>()) as wgpu::BufferAddress;
        let grid_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Buffer"),
            size: grid_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let particle_buffer_size = (sim.config.max_particles
            * std::mem::size_of::<ParticleInstance>())
            as wgpu::BufferAddress;
        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer"),
            size: particle_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let w = sim.config.width as f32;
        let h = sim.config.height as f32;

        let grid_scale = [(sim.h / w) * 2.0, (sim.h / h) * 2.0];
        let particle_scale = [
            (sim.config.particle_radius * 2.0 / w) * 2.0,
            (sim.config.particle_radius * 2.0 / h) * 2.0,
        ];

        let particle_uniform_buffer =
            device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Particle Uniform Buffer"),
                contents: bytemuck::cast_slice(&[Uniforms {
                    scale: particle_scale,
                    is_particle: 1,
                    _padding: 0,
                }]),
                usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
            });

        let grid_uniform_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Grid Uniform Buffer"),
            contents: bytemuck::cast_slice(&[Uniforms {
                scale: grid_scale,
                is_particle: 0,
                _padding: 0,
            }]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        let uniform_bind_group_layout =
            device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
                entries: &[wgpu::BindGroupLayoutEntry {
                    binding: 0,
                    visibility: wgpu::ShaderStages::VERTEX | wgpu::ShaderStages::FRAGMENT,
                    ty: wgpu::BindingType::Buffer {
                        ty: wgpu::BufferBindingType::Uniform,
                        has_dynamic_offset: false,
                        min_binding_size: None,
                    },
                    count: None,
                }],
                label: Some("uniform_bind_group_layout"),
            });

        let particle_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: particle_uniform_buffer.as_entire_binding(),
            }],
            label: Some("particle_bind_group"),
        });

        let grid_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &uniform_bind_group_layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: grid_uniform_buffer.as_entire_binding(),
            }],
            label: Some("grid_bind_group"),
        });

        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Some Pipeline Layout"),
            bind_group_layouts: &[Some(&uniform_bind_group_layout)],
            immediate_size: 0,
        });

        let render_pipeline = device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
            label: Some("Render Pipeline"),
            layout: Some(&pipeline_layout),
            vertex: wgpu::VertexState {
                module: &shader,
                entry_point: Some("vs_main"),
                buffers: &[wgpu::VertexBufferLayout {
                    array_stride: std::mem::size_of::<ParticleInstance>() as wgpu::BufferAddress,
                    step_mode: wgpu::VertexStepMode::Instance,
                    attributes: &[
                        wgpu::VertexAttribute {
                            offset: 0,
                            shader_location: 1,
                            format: wgpu::VertexFormat::Float32x2,
                        },
                        wgpu::VertexAttribute {
                            offset: 8,
                            shader_location: 2,
                            format: wgpu::VertexFormat::Float32x3,
                        },
                    ],
                }],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            },
            fragment: Some(wgpu::FragmentState {
                module: &shader,
                entry_point: Some("fs_main"),
                targets: &[Some(wgpu::ColorTargetState {
                    format: surf_config.format,
                    blend: Some(wgpu::BlendState::REPLACE),
                    write_mask: wgpu::ColorWrites::ALL,
                })],
                compilation_options: wgpu::PipelineCompilationOptions::default(),
            }),
            primitive: wgpu::PrimitiveState {
                topology: wgpu::PrimitiveTopology::TriangleStrip,
                ..Default::default()
            },
            depth_stencil: None,
            multisample: wgpu::MultisampleState::default(),
            multiview_mask: None,
            cache: None,
        });

        let mut grid_instances = vec![
            ParticleInstance {
                position: [0.0, 0.0],
                color: [0.0, 0.0, 0.0]
            };
            sim.f_num_cells
        ];

        for x in 0..sim.f_num_x {
            for y in 0..sim.f_num_y {
                let pos_x = ((x as f32 + 0.5) * sim.h / sim.config.width as f32) * 2.0 - 1.0;
                let pos_y = ((y as f32 + 0.5) * sim.h / sim.config.height as f32) * 2.0 - 1.0;
                grid_instances[x * sim.f_num_y + y].position = [pos_x, pos_y];
            }
        }

        Self {
            _window: window,
            surface,
            device,
            queue,
            surf_config,
            phys_size,
            render_pipeline,
            particle_buffer,
            grid_buffer,
            grid_bind_group,
            particle_bind_group,
            grid_instances,
            sim,
            runtime_config,
        }
    }

    pub fn render(&mut self) {
        let particle_instances: Vec<ParticleInstance> = self.sim.particles
            [..self.sim.num_particles]
            .iter()
            .map(|p| ParticleInstance {
                // sim coords to gpu coords ( [-1.0, 1.0] )
                position: [
                    (p.x / self.sim.config.width as f32) * 2.0 - 1.0,
                    (p.y / self.sim.config.height as f32) * 2.0 - 1.0,
                ],
                color: [p.color.0, p.color.1, p.color.2],
            })
            .collect();

        for x in 0..self.sim.f_num_x {
            for y in 0..self.sim.f_num_y {
                let idx = x * self.sim.f_num_y + y;
                let cell = &self.sim.grid[idx];
                self.grid_instances[idx].color = [cell.color.0, cell.color.1, cell.color.2];
            }
        }

        self.queue.write_buffer(
            &self.particle_buffer,
            0,
            bytemuck::cast_slice(&particle_instances),
        );

        self.queue.write_buffer(
            &self.grid_buffer,
            0,
            bytemuck::cast_slice(&self.grid_instances),
        );

        let output = match self.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            _ => return,
        };

        let view = output
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Render Encoder"),
            });

        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &view,
                    resolve_target: None,
                    depth_slice: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
                multiview_mask: None,
            });

            render_pass.set_pipeline(&self.render_pipeline);

            if self.runtime_config.draw_grid {
                render_pass.set_bind_group(0, &self.grid_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.grid_buffer.slice(..));
                render_pass.draw(0..4, 0..self.sim.f_num_cells as u32);
            }

            if self.runtime_config.draw_particles {
                render_pass.set_bind_group(0, &self.particle_bind_group, &[]);
                render_pass.set_vertex_buffer(0, self.particle_buffer.slice(..));
                render_pass.draw(0..4, 0..self.sim.num_particles as u32);
            }
        }

        self.queue.submit(std::iter::once(encoder.finish()));
        output.present();
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.phys_size = new_size;
            self.surf_config.width = new_size.width;
            self.surf_config.height = new_size.height;
            self.surface.configure(&self.device, &self.surf_config);
        }
    }
}
