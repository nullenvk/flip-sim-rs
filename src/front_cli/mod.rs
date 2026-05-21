use crate::simulation::Simulation; 

#[derive(Debug)]
pub struct FrontCLI {
    sim: Simulation
}

impl FrontCLI {
    pub fn new(sim: Simulation) -> Self {
        Self { sim }
    }
}
