use flip_sim_rs::simulation::*;
use flip_sim_rs::front_wgpu::*;

use std::sync::Arc;

use winit::{
    application::ApplicationHandler, 
    event::*, 
    event_loop::*,
    keyboard::{KeyCode, PhysicalKey}, 
    window::*
};


fn main() {
    let config = config::Config {
        width: 30,
        height: 30,
        spacing: 1.0,
        density: 1.0,
        particle_radius: 0.5,
        max_particles: 1000,
        // pic_flip_ratio:0.9,
        // gravity: (0.0,9.81),
        // dt: 1.0/30.0,
        // over_relaxation: 1.9
    };

    let runtime_config = config::RuntimeConfig {
        dt: 1.0 / 120.0,
        gravity: (0.0, 0.91),
        flip_ratio: 0.9,
        num_pressure_iters: 50,
        num_particle_iters: 2,
        over_relaxation: 1.9,
        compensate_drift: true,
        separate_particles: false,
        obstacle_x: 4.0,
        obstacle_y: 4.0,
        obstacle_radius: 1.5,
        obstacle_vel_x: 0.0,
        obstacle_vel_y: 0.0,
    };

    let sim = Simulation::new(&config); 

    let event_loop = EventLoop::new().unwrap();
    event_loop.set_control_flow(ControlFlow::Poll);

    let mut app = App {
        window: None,
        front: None,
        sim: Some(sim),
        runtime_config,
    };

    event_loop.run_app(&mut app).unwrap();
}

struct App {
    window: Option<Arc<Window>>,
    front: Option<FrontWgpu>,
    sim: Option<Simulation>,
    runtime_config: config::RuntimeConfig,
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
 
            WindowEvent::Resized(new_size) => {
                front.resize(new_size);
            }
 
            WindowEvent::RedrawRequested => {
                front.sim.simulate(&front.runtime_config);
                front.render();
                self.window.as_ref().unwrap().request_redraw();
            }
 
            _ => {}
        }
    }
}
 

