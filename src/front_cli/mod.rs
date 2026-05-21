use crate::simulation::Simulation; 

use std::{thread, time::Duration};

#[derive(Debug)]
pub struct FrontCLI {
    sim: Simulation,

    frames_per_second: f32
}

impl FrontCLI {
    pub fn new(sim: Simulation) -> Self {
        Self {
            sim,
            frames_per_second: 2.0
        }
    }

    pub fn init(&mut self) {

    }

    pub fn run(&mut self) {
        loop {
            println!("Frame!");

            self.sim.step();
            self.draw_frame();

            thread::sleep(Duration::from_secs_f32(1.0 / self.frames_per_second));
        }
    }

    pub fn draw_frame(&self) {
        let config = &self.sim.config;
        for y in 0..config.height {
            for x in 0..config.width {
                let c = if self.sim.get_cell(x,y).test {'X'} else {' '};

                print!("{}", c);
            }

            println!("");
        }
    }
}
