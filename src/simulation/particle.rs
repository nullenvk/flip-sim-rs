#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle {
    pub x: f64,
    pub y: f64,
    pub vx: f64,
    pub vy: f64,
    pub color: (f64, f64, f64),
}

impl Default for Particle {
    fn default() -> Self {
        Particle {
            x: 0.0,
            y: 0.0,
            vx: 0.0,
            vy: 0.0,
            color: (0.0, 0.0, 1.0), // blue
        }
    }
}
