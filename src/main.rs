use flip_sim_rs::simulation::*;
use flip_sim_rs::front_cli::*;

fn main() {
    let config = config::Config {
        width: 8,
        height: 8,
    };

    let sim = Simulation::new(config); 
    let mut front = FrontCLI::new(sim);

    front.init();
    front.run()
}
