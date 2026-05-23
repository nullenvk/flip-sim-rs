#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Particle {
    pub x: f32,
    pub y: f32,
    pub vx: f32,
    pub vy: f32,
    pub color: (f32, f32, f32),
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
