use flip_sim_rs::simulation::*;
use flip_sim_rs::front_cli::*;
use flip_sim_rs::front::*;

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
    let mut front = FrontCLI::new(sim, runtime_config);
    front.run()
}
