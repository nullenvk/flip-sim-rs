use crate::simulation::Simulation; 
use crate::simulation::config::*; 

pub trait FrontEnd {
    fn new(sim: Simulation, runtime_config: RuntimeConfig) -> Self;
    fn run(&mut self);
}
