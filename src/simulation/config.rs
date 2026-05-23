#[derive(Debug)]
pub struct Config {
    pub width: usize,
    pub height: usize,
    pub density: f64,
    pub spacing: f64,
    pub particle_radius: f64,
    pub max_particles: f64,
    pub pic_flip_ratio: f64,
    pub gravity: (f64,f64),
    pub dt: f64,
    pub over_relaxation: f64
}
