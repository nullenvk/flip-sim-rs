use flip_sim_rs::simulation::*;
use flip_sim_rs::front_cli::*;

fn main() {
    let sim = Simulation::new(); 
    let front = FrontCLI::new(sim);

    println!("{:?}", front);
}
