pub mod config;
pub mod cell;
use crate::simulation::cell::CellTypes;

use config::Config;
use cell::Cell;

use rand::prelude::*;

#[derive(Debug)]
pub struct Simulation {
    pub config: Config,
    pub grid: Vec<Cell>,

}

impl Simulation {
    pub fn new(config: Config) -> Simulation {
        let total_cells = config.width * config.height;


        let mut sim = Simulation {
            config,
            grid: vec![Cell::default(); total_cells]
        };

        sim.grid[0] = Cell { cell_type: CellTypes::Solid };

        sim
    }

    pub fn get_cell(&self, x: usize, y: usize) -> &Cell {
        &self.grid[y * self.config.height + x]
    }

    pub fn step(&mut self) {
        let mut rng = rand::rng();

        self.grid.shuffle(&mut rng);
    }
}
