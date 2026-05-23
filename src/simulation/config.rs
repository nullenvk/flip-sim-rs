#[derive(Debug)]
pub struct Config {
    pub width: usize,
    pub height: usize,
    pub density: f64,
    pub spacing: f64,
    pub particleRadius: f64,
    pub maxParticles: f64,
    pub picFlipRaTio: f64,
    pub gravity: f64,
    pub dt: f64,
    pub overRelaxation: f64
}
