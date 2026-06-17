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

fn sim_to_gpu_coords(val: f32, max: f32) -> f32 {
    (val / max) * 2.0 - 1.0
}

pub struct WgpuContext {
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    surf_config: wgpu::SurfaceConfiguration,
    phys_size: winit::dpi::PhysicalSize<u32>,
}


impl WgpuContext {
    async fn new(window: &Arc<winit::window::Window>) -> Self {
        let phys_size = window.inner_size();

        let instance = wgpu::Instance::default();
        let surface = instance.create_surface(Arc::clone(window)).unwrap();
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

        Self {
            surface,
            device,
            queue,
            surf_config,
            phys_size,
        }
    }
}

pub struct FrontWgpu {
    _window: Arc<winit::window::Window>,
    ctx: WgpuContext,

    render_pipeline: wgpu::RenderPipeline,

    particle_buffer: wgpu::Buffer,
    particle_bind_group: wgpu::BindGroup,

    grid_buffer: wgpu::Buffer,
    grid_bind_group: wgpu::BindGroup,

    grid_instances: Vec<ParticleInstance>,

    pub sim: Simulation,
    pub runtime_config: RuntimeConfig,

    pub egui_ctx: egui::Context,
    pub egui_state: egui_winit::State,
    pub egui_renderer: egui_wgpu::Renderer,
}

impl FrontWgpu {
    pub async fn new(
        window: Arc<winit::window::Window>,
        sim: Simulation,
        runtime_config: RuntimeConfig,
    ) -> Self {
        let ctx = WgpuContext::new(&window).await;

        let (grid_buffer, particle_buffer) = Self::create_vertex_buffers(&ctx.device, &sim);
        let (grid_bind_group, particle_bind_group, bind_group_layout) =
            Self::create_bind_groups(&ctx.device, &sim);
        let render_pipeline = Self::create_render_pipeline(&ctx.device, &ctx.surf_config, &bind_group_layout);
        let grid_instances = Self::init_grid_instances(&sim);

        // EGUI
        let egui_ctx = egui::Context::default();
        let egui_state = egui_winit::State::new(
            egui_ctx.clone(),
            egui::ViewportId::ROOT,
            &window,
            Some(window.scale_factor() as f32),
            None,
            None
        );
        let egui_renderer = egui_wgpu::Renderer::new(
            &ctx.device,
            ctx.surf_config.format,
            egui_wgpu::RendererOptions::default(),
        );

        Self {
            _window: window,
            ctx,
            render_pipeline,
            particle_buffer,
            grid_buffer,
            grid_bind_group,
            particle_bind_group,
            grid_instances,
            sim,
            runtime_config,

            egui_ctx,
            egui_state,
            egui_renderer
        }
    }

    fn create_vertex_buffers(device: &wgpu::Device, sim: &Simulation) -> (wgpu::Buffer, wgpu::Buffer) {
        let grid_buffer_size =
            (sim.f_num_cells * std::mem::size_of::<ParticleInstance>()) as wgpu::BufferAddress;
        let grid_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Grid Buffer"),
            size: grid_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let particle_buffer_size = (sim.config.max_particles * std::mem::size_of::<ParticleInstance>())
            as wgpu::BufferAddress;
        let particle_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Particle Buffer"),
            size: particle_buffer_size,
            usage: wgpu::BufferUsages::VERTEX | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        (grid_buffer, particle_buffer)
    }

    fn create_bind_groups(
        device: &wgpu::Device,
        sim: &Simulation,
    ) -> (wgpu::BindGroup, wgpu::BindGroup, wgpu::BindGroupLayout) {
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

        let layout = device.create_bind_group_layout(&wgpu::BindGroupLayoutDescriptor {
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
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: particle_uniform_buffer.as_entire_binding(),
            }],
            label: Some("particle_bind_group"),
        });

        let grid_bind_group = device.create_bind_group(&wgpu::BindGroupDescriptor {
            layout: &layout,
            entries: &[wgpu::BindGroupEntry {
                binding: 0,
                resource: grid_uniform_buffer.as_entire_binding(),
            }],
            label: Some("grid_bind_group"),
        });

        (grid_bind_group, particle_bind_group, layout)
    }

    fn create_render_pipeline(
        device: &wgpu::Device,
        surf_config: &wgpu::SurfaceConfiguration,
        bind_group_layout: &wgpu::BindGroupLayout,
    ) -> wgpu::RenderPipeline {
        let shader = device.create_shader_module(wgpu::include_wgsl!("shader.wgsl"));

        let pipeline_layout = device.create_pipeline_layout(&wgpu::PipelineLayoutDescriptor {
            label: Some("Some Pipeline Layout"),
            bind_group_layouts: &[Some(bind_group_layout)],
            immediate_size: 0,
        });

        device.create_render_pipeline(&wgpu::RenderPipelineDescriptor {
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
        })
    }

    fn init_grid_instances(sim: &Simulation) -> Vec<ParticleInstance> {
        let mut grid_instances = vec![
            ParticleInstance {
                position: [0.0, 0.0],
                color: [0.0, 0.0, 0.0]
            };
            sim.f_num_cells
        ];

        for x in 0..sim.f_num_x {
            for y in 0..sim.f_num_y {
                let sim_pos_x = (x as f32 + 0.5) * sim.h;
                let sim_pos_y = (y as f32 + 0.5) * sim.h;

                let pos_x = sim_to_gpu_coords(sim_pos_x, sim.config.width as f32);
                let pos_y = sim_to_gpu_coords(sim_pos_y, sim.config.height as f32);

                grid_instances[x * sim.f_num_y + y].position = [pos_x, pos_y];
            }
        }
        grid_instances
    }

    fn update_buffers(&mut self) {
        let particle_instances: Vec<ParticleInstance> = self.sim.particles
            [..self.sim.num_particles]
            .iter()
            .map(|p| ParticleInstance {
                position: [
                    sim_to_gpu_coords(p.x, self.sim.config.width as f32),
                    sim_to_gpu_coords(p.y, self.sim.config.height as f32),
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

        self.ctx.queue.write_buffer(
            &self.particle_buffer,
            0,
            bytemuck::cast_slice(&particle_instances),
        );

        self.ctx.queue.write_buffer(
            &self.grid_buffer,
            0,
            bytemuck::cast_slice(&self.grid_instances),
        );
    }

    fn prepare_egui(&mut self, encoder: &mut wgpu::CommandEncoder) -> (Vec<egui::ClippedPrimitive>, egui::TexturesDelta) {
        let raw_input = self.egui_state.take_egui_input(&self._window);
        self.egui_ctx.begin_pass(raw_input);

        self.draw_ui();

        let full_output = self.egui_ctx.end_pass();
        let paint_jobs = self.egui_ctx.tessellate(full_output.shapes, self.egui_ctx.pixels_per_point());

        for (id, image_delta) in &full_output.textures_delta.set {
            self.egui_renderer.update_texture(&self.ctx.device, &self.ctx.queue, *id, image_delta);
        }

        self.egui_renderer.update_buffers(
            &self.ctx.device,
            &self.ctx.queue,
            encoder,
            &paint_jobs,
            &egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.ctx.surf_config.width, self.ctx.surf_config.height],
                pixels_per_point: self.egui_ctx.pixels_per_point(),
            },
        );

        (paint_jobs, full_output.textures_delta)
    }

    fn render_simulation(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Sim Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
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

    fn render_egui_overlay(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView, paint_jobs: &[egui::ClippedPrimitive]) {
        let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("EGUI Render Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view,
                resolve_target: None,
                depth_slice: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Load,
                    store: wgpu::StoreOp::Store,
                },
            })],
            depth_stencil_attachment: None,
            timestamp_writes: None,
            occlusion_query_set: None,
            multiview_mask: None,
        }).forget_lifetime();

        self.egui_renderer.render(
            &mut render_pass,
            paint_jobs,
            &egui_wgpu::ScreenDescriptor {
                size_in_pixels: [self.ctx.surf_config.width, self.ctx.surf_config.height],
                pixels_per_point: self.egui_ctx.pixels_per_point(),
            },
        );
    }

    pub fn handle_window_event(&mut self, window: &winit::window::Window, event: &winit::event::WindowEvent) -> bool {
        self.egui_state.on_window_event(window, event).consumed
    }

    fn draw_ui(&mut self) {
        egui::Window::new("Simulation Controls").show(&self.egui_ctx, |ui| {
            ui.heading("Time & Forces");
            ui.add(egui::Slider::new(&mut self.runtime_config.dt, 0.001..=0.05).text("Delta Time"));
            
            ui.horizontal(|ui| {
                ui.label("Gravity:");
                ui.add(egui::DragValue::new(&mut self.runtime_config.gravity.0).speed(1.0));
                ui.add(egui::DragValue::new(&mut self.runtime_config.gravity.1).speed(1.0));
            });

            ui.separator();
            ui.heading("Solver");
            ui.add(egui::Slider::new(&mut self.runtime_config.num_pressure_iters, 1..=200).text("Pressure Iters"));
            ui.add(egui::Slider::new(&mut self.runtime_config.num_particle_iters, 1..=10).text("Particle Iters"));
            ui.add(egui::Slider::new(&mut self.runtime_config.over_relaxation, 1.0..=2.0).text("Over-relaxation"));
            
            ui.separator();
            ui.heading("Rendering");
            ui.checkbox(&mut self.runtime_config.draw_grid, "Draw Grid");
            ui.checkbox(&mut self.runtime_config.draw_particles, "Draw Particles");
        });
    }

    pub fn render(&mut self) {
        self.update_buffers();

        let output = match self.ctx.surface.get_current_texture() {
            wgpu::CurrentSurfaceTexture::Success(surface_texture) => surface_texture,
            wgpu::CurrentSurfaceTexture::Suboptimal(surface_texture) => surface_texture,
            _ => return,
        };

        let view = output.texture.create_view(&wgpu::TextureViewDescriptor::default());
        let mut encoder = self.ctx.device.create_command_encoder(&wgpu::CommandEncoderDescriptor {
            label: Some("Render Encoder"),
        });

        let (paint_jobs, textures_delta) = self.prepare_egui(&mut encoder);
        self.render_simulation(&mut encoder, &view);
        self.render_egui_overlay(&mut encoder, &view, &paint_jobs);
        self.ctx.queue.submit(std::iter::once(encoder.finish()));
        output.present();

        for x in &textures_delta.free {
            self.egui_renderer.free_texture(x);
        }
    }

    pub fn resize(&mut self, new_size: winit::dpi::PhysicalSize<u32>) {
        if new_size.width > 0 && new_size.height > 0 {
            self.ctx.phys_size = new_size;
            self.ctx.surf_config.width = new_size.width;
            self.ctx.surf_config.height = new_size.height;
            self.ctx.surface.configure(&self.ctx.device, &self.ctx.surf_config);
        }
    }
}
