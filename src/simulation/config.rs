#[derive(Debug, Clone)]
pub struct Config {
    pub width: usize,
    pub height: usize,
    pub density: f64,
    pub spacing: f64,
    pub particle_radius: f64,
    pub max_particles: usize,
    // pub pic_flip_ratio: f64,
    // pub gravity: (f64,f64),
    // pub dt: f64,
    // pub over_relaxation: f64
    // Those are not in config, cause you theoretically you COULD change them mid simulation, so I decided to left it as it was in the guy's code
}

#[derive(Debug, Clone, Copy)]
pub struct RuntimeConfig {
    pub dt: f64,
    pub gravity: (f64, f64),
    pub flip_ratio: f64,
    pub num_pressure_iters: usize,
    pub num_particle_iters: usize,
    pub over_relaxation: f64,
    pub compensate_drift: bool,
    pub separate_particles: bool,
    pub obstacle_x: f64,
    pub obstacle_y: f64,
    pub obstacle_radius: f64,
    pub obstacle_vel_x: f64,
    pub obstacle_vel_y: f64,
}
