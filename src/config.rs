use crate::simulation::config::*;

pub const CONFIG: Config = Config {
        width: 9,
        height: 10,
        spacing: 1.0,
        density: 100.0,
        particle_radius: 0.25,
        max_particles: 100,
    };

pub const INITIAL_RUNTIME_CONFIG: RuntimeConfig = RuntimeConfig {
        dt: 1.0 / 45.0,
        gravity: (10.0, 10.0),
        flip_ratio: 0.8,
        num_pressure_iters: 10,
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