use flip_sim_rs::simulation::*;
use flip_sim_rs::front_cli::*;
use flip_sim_rs::front::*;

fn main() {
    let config = config::Config {
        width: 8,
        height: 8,
        spacing: 1.0,
        density: 1.0,
        particle_radius: 1.0,
        max_particles: 100.0,
        pic_flip_ratio:0.9,
        gravity: (0.0,9.81),
        dt: 1.0/30.0,
        over_relaxation: 1.9
    };

    let sim = Simulation::new(config); 
    let mut front = FrontCLI::new(sim);

    front.init();
    front.run()
}
