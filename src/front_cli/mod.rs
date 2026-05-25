use crate::front;
use crate::simulation::Simulation;
use crate::simulation::cell::*;
use crate::simulation::config::*;

use std::{thread, time::Duration};

#[derive(Debug)]
pub struct FrontCLI {
    sim: Simulation,
    runtime_config: RuntimeConfig,
}

impl front::FrontEnd for FrontCLI {
    fn new(sim: Simulation, runtime_config: RuntimeConfig) -> Self {
        Self {
            sim,
            runtime_config,
        }
    }

    fn run(&mut self) {
        loop {
            println!("Frame!");

            self.sim.simulate(&self.runtime_config);
            self.draw_frame();

            thread::sleep(Duration::from_secs_f32(self.runtime_config.dt));
        }
    }
}

impl FrontCLI {
    fn draw_frame(&self) {
        let (w, h) = (self.sim.f_num_x, self.sim.f_num_y);

        // clear screen
        print!("\x1B[2J\x1B[1;1H");

        for y in (0..h).rev() {
            for x in 0..w {
                let cell = self.sim.get_cell(x, y);

                let c = match cell.cell_type {
                    CellTypes::Solid => '#',
                    CellTypes::Liquid => '~',
                    CellTypes::Gas => ' ',
                };

                print!("{}{}", c, c);
            }

            println!("");
        }
    }
}
