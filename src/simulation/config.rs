#[derive(Debug)]
pub struct Config {
    pub width: usize,
    pub height: usize,
    pub density: f32,
    pub spacing: f32,
    pub particle_radius: f32,
    pub max_particles: usize,
    // pub pic_flip_ratio: f32,
    // pub gravity: (f32,f32),
    // pub dt: f32,
    // pub over_relaxation: f32
    // Those are not in config, cause you theoretically you COULD change them mid simulation, so I decided to left it as it was in the guy's code
}
