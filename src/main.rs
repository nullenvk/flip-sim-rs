use flip_sim_rs::front_wgpu::*;
use flip_sim_rs::simulation::config::RuntimeConfig;
use flip_sim_rs::simulation::*;
use winit::dpi::PhysicalSize;

use std::sync::Arc;
use std::time::Instant;

use winit::{
    application::ApplicationHandler,
    event::*,
    event_loop::*,
    keyboard::{KeyCode, PhysicalKey},
    window::*,
};

const MAX_GRAVITY: f32 = 60.0;

fn main() {
    let config = config::Config {
        width: 80,
        height: 80,
        spacing: 1.0,
        density: 1000.0,
        particle_radius: 0.35,
        max_particles: 1600,
    };

    let runtime_config = config::RuntimeConfig {
        dt: 1.0 / 60.0,
        gravity: (0.0, -100.0),
        flip_ratio: 0.9,
        num_pressure_iters: 100,
        num_particle_iters: 2,
        over_relaxation: 1.9,
        compensate_drift: true,
        separate_particles: true,
        obstacle_x: 0.0,
        obstacle_y: 0.0,
        obstacle_radius: 0.0,
        obstacle_vel_x: 0.0,
        obstacle_vel_y: 0.0,

        draw_grid: true,
        draw_particles: true,
        mono_mode: true,
    };

    let mut sim = Simulation::new(&config);

    // ---------- NOWY KSZTAŁT: KOŁO ----------
    let cx = sim.f_num_x as f32 * sim.h * 0.5; // środek domeny X
    let cy = sim.f_num_y as f32 * sim.h * 0.5; // środek domeny Y
    let radius = (sim.f_num_x.min(sim.f_num_y) as f32 * sim.h) * 0.45; // 45% krótszego boku

    // Ustaw komórki: wewnątrz koła -> s=1.0, na zewnątrz -> s=0.0 (Solid)
    for x in 0..sim.f_num_x {
        for y in 0..sim.f_num_y {
            let cell_center_x = (x as f32 + 0.5) * sim.h;
            let cell_center_y = (y as f32 + 0.5) * sim.h;
            let dx = cell_center_x - cx;
            let dy = cell_center_y - cy;
            let in_circle = dx * dx + dy * dy <= radius * radius;

            let cell_nr = x * sim.f_num_y + y;
            sim.grid[cell_nr].s = if in_circle { 1.0 } else { 0.0 };
            sim.grid[cell_nr].cell_type = if in_circle {
                cell::CellTypes::Gas
            } else {
                cell::CellTypes::Solid
            };
        }
    }

    // ---------- NOWE CZĄSTKI W KOLE ----------
    // Wyczyść stare cząstki
    sim.num_particles = 0;
    let r = config.particle_radius;
    let dx = 2.0 * r;
    let dy = (3.0_f32).sqrt() / 2.0 * dx;

    // Ile cząstek zmieści się w prostokącie opisującym koło (przybliżenie)
    let num_x = ((2.0 * radius - 2.0 * r) / dx).floor() as usize;
    let num_y = ((2.0 * radius - 2.0 * r) / dy).floor() as usize;
    let start_x = cx - radius + r;
    let start_y = cy - radius + r;

    let mut p_idx = 0;
    'spawn: for j in 0..num_y {
        for i in 0..num_x {
            if p_idx >= config.max_particles {
                break 'spawn;
            }
            let px = start_x + dx * i as f32 + if j % 2 == 0 { 0.0 } else { r };
            let py = start_y + dy * j as f32;

            // sprawdź, czy cząstka jest wewnątrz koła
            if (px - cx) * (px - cx) + (py - cy) * (py - cy) <= (radius - r) * (radius - r) {
                let jitter = if p_idx % 2 == 0 { 1e-4 } else { -1e-4 };
                sim.particles[p_idx].x = px + jitter;
                sim.particles[p_idx].y = py;
                sim.particles[p_idx].color = (0.0, 0.5, 1.0);
                p_idx += 1;
            }
        }
    }
    sim.num_particles = p_idx;

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App::new(sim, runtime_config);

    event_loop.run_app(&mut app).unwrap();
}

struct App {
    window: Option<Arc<Window>>,
    window_size: PhysicalSize<u32>,
    front: Option<FrontWgpu>,
    sim: Option<Simulation>,
    runtime_config: config::RuntimeConfig,

    last_frame: Instant,
    frame_acc: f32,
}

impl App {
    pub fn new(sim: Simulation, runtime_config: RuntimeConfig) -> Self {
        Self {
            window: None,
            front: None,
            window_size: PhysicalSize {
                width: 0,
                height: 0,
            },
            sim: Some(sim),
            runtime_config,
            last_frame: Instant::now(),
            frame_acc: 0.0,
        }
    }
}
impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        if self.window.is_some() {
            return;
        }

        let window = Arc::new(
            event_loop
                .create_window(
                    WindowAttributes::default()
                        .with_title("flip-sim-rs")
                        .with_inner_size(winit::dpi::LogicalSize::new(800, 800)),
                )
                .unwrap(),
        );

        let sim = self.sim.take().unwrap();
        let front = pollster::block_on(FrontWgpu::new(
            Arc::clone(&window),
            sim,
            self.runtime_config.clone(),
        ));

        self.last_frame = Instant::now();
        self.window_size = window.inner_size();
        self.window = Some(window);
        self.front = Some(front);
    }

    fn window_event(
        &mut self,
        event_loop: &ActiveEventLoop,
        _window_id: winit::window::WindowId,
        event: WindowEvent,
    ) {
        let Some(front) = self.front.as_mut() else {
            return;
        };

        match event {
            WindowEvent::CloseRequested => {
                event_loop.exit();
            }
            WindowEvent::KeyboardInput {
                event:
                    KeyEvent {
                        physical_key: PhysicalKey::Code(key),
                        state: ElementState::Pressed,
                        ..
                    },
                ..
            } => match key {
                KeyCode::Escape => event_loop.exit(),
                _ => {}
            },
            WindowEvent::CursorMoved {
                device_id: _,
                position,
            } => {
                front.runtime_config.gravity = (
                    ((position.x as f32 - (self.window_size.width as f32 / 2.0))
                        / (self.window_size.width as f32 / 2.0))
                        * MAX_GRAVITY,
                    -((position.y as f32 - (self.window_size.width as f32 / 2.0))
                        / (self.window_size.width as f32 / 2.0))
                        * MAX_GRAVITY,
                );
                println!("Gravity: {:?}", front.runtime_config.gravity);
            }

            WindowEvent::Resized(new_size) => {
                front.resize(new_size);
            }

            WindowEvent::RedrawRequested => {
                let now = Instant::now();
                let elapsed = (now - self.last_frame).as_secs_f32().min(0.25);
                self.last_frame = now;

                self.frame_acc += elapsed;
                let dt = front.runtime_config.dt;
                while self.frame_acc >= dt {
                    front.sim.simulate(&front.runtime_config);
                    self.frame_acc -= dt;
                }

                front.render();
                self.window.as_ref().unwrap().request_redraw();
            }

            _ => {}
        }
    }
}
