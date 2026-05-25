use flip_sim_rs::simulation::config::RuntimeConfig;
use flip_sim_rs::simulation::*;
use flip_sim_rs::front_wgpu::*;
use winit::dpi::PhysicalSize;

use std::sync::Arc;
use std::time::Instant;

use winit::{
    application::ApplicationHandler, 
    event::*, 
    event_loop::*,
    keyboard::{KeyCode, PhysicalKey}, 
    window::*
};

const MAX_GRAVITY: f32 = 200.0;

fn main() {
    let config = config::Config {
        width: 160,
        height: 90,
        spacing: 1.0,
        density: 1000.0,
        particle_radius: 0.3,
        max_particles: 10000,
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
        draw_particles: false,
    };

    let mut sim = Simulation::new(&config); 

            // tank boundaries
         for x in 0..sim.f_num_x {
            for y in 0..sim.f_num_y {
                let mut s = 1.0; 

                if x == 0 || x == sim.f_num_x - 1 || y == 0 {   
                    s = 0.0;
                }
                
                let cell_nr = x * sim.f_num_y + y;
                sim.grid[cell_nr].s = s;
                if s == 0.0 {
                    sim.grid[cell_nr].cell_type = cell::CellTypes::Solid;
                }
            }
        }

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
            window_size: PhysicalSize { width: 0, height: 0 },
            sim: Some(sim),
            runtime_config,
            last_frame: Instant::now(),
            frame_acc: 0.0
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
        let front = pollster::block_on(FrontWgpu::new(Arc::clone(&window), sim, self.runtime_config.clone()));
 
        self.last_frame = Instant::now();
        self.window_size = window.inner_size();
        self.window = Some(window);
        self.front = Some(front);
    }
 
    fn window_event(&mut self, event_loop: &ActiveEventLoop, _window_id: winit::window::WindowId, event: WindowEvent) {
        let Some(front) = self.front.as_mut() else { return };
 
        match event {
            WindowEvent::CloseRequested => { event_loop.exit(); }
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
            WindowEvent::CursorMoved { device_id: _, position } => {
                front.runtime_config.gravity = (
                    ((position.x as f32 - (self.window_size.width as f32 / 2.0)) / (self.window_size.width as f32 / 2.0)) * MAX_GRAVITY,
                    -((position.y as f32 - (self.window_size.width as f32 / 2.0)) / (self.window_size.width as f32 / 2.0)) * MAX_GRAVITY,
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
 

