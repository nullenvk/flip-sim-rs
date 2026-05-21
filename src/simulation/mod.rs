#[derive(Debug)]
pub struct Simulation {
    pub x: f32
}

impl Simulation {
    pub fn new() -> Simulation {
        Simulation {
            x: 0.0
        }
    }
}
