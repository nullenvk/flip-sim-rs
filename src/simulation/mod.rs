pub mod config;
pub mod cell;

use config::Config;
use cell::*;

use rand::prelude::*;

#[derive(Debug)]
pub struct Simulation {
    pub config: Config,
    pub grid: Vec<Cell>,
    pub f_num_x: usize,
    pub f_num_y: usize,
    pub f_num_cells: usize,
    pub h: f32,
    pub f_inv_spacing: f32,

    pub u: Vec<f32>,
    pub v: Vec<f32>,
    pub du: Vec<f32>,
    pub dv: Vec<f32>,
    pub prev_u: Vec<f32>,
    pub prev_v: Vec<f32>,
    pub p: Vec<f32>,
    pub s: Vec<f32>,

    pub particle_pos: Vec<f32>,
    pub particle_vel: Vec<f32>,
    pub particle_density: Vec<f32>,
    pub particle_rest_density: f32,

    pub p_inv_spacing: f32,
    pub p_num_x: usize,
    pub p_num_y: usize,
    pub p_num_cells: usize,

    pub num_cell_particles: Vec<i32>,
    pub first_cell_particle: Vec<i32>,
    pub cell_particle_ids: Vec<i32>,

    pub num_particles: i32



}

impl Simulation {
    pub fn new(config: Config) -> Simulation {

        // let total_cells = config.width * config.height;
        let f_num_x = (config.width as f32 / config.spacing).floor() as usize + 1;
        let f_num_y = (config.height as f32 / config.spacing).floor() as usize + 1;
        let f_num_cells = f_num_x * f_num_y;
        let h = f32::max(config.width as f32 / f_num_x as f32, config.height as f32 / f_num_y as f32);
        let f_inv_spacing = 1.0 / h;

        let u = vec![0.0;f_num_cells];
        let v = vec![0.0;f_num_cells];
        let du = vec![0.0;f_num_cells];
        let dv = vec![0.0;f_num_cells];
        let prev_u = vec![0.0;f_num_cells];
        let prev_v = vec![0.0;f_num_cells];
        let p = vec![0.0;f_num_cells];
        let s = vec![0.0;f_num_cells];

        let particle_pos = vec![0.0; 2 * config.max_particles];
        let particle_vel = vec![0.0; 2 * config.max_particles];
        let particle_density = vec![0.0; f_num_cells];

        let p_inv_spacing = 1.0 / (2.2 * config.particle_radius);
        let p_num_x = (config.width as f32 * p_inv_spacing).floor() as usize + 1;
        let p_num_y = (config.height as f32 * p_inv_spacing).floor() as usize + 1;
        let p_num_cells = p_num_x * p_num_y;
        let first_cell_particle = vec![0;config.max_particles + 1];
        let cell_particle_ids = vec![0;config.max_particles as usize];

        let mut sim = Simulation {
            config,
            grid: vec![Cell::default(); f_num_cells],
            f_num_x,
            f_num_y,
            f_num_cells,
            h,
            f_inv_spacing,

            u,
            v,
            du,
            dv,
            prev_u,
            prev_v,
            p,
            s,

            particle_pos,
            particle_vel,
            particle_density,
            particle_rest_density: 0.0,
            p_inv_spacing,
            p_num_x,
            p_num_y,
            p_num_cells,
            num_cell_particles: vec![0;p_num_cells],
            first_cell_particle,
            cell_particle_ids,
            num_particles: 0
        };

        sim.grid[0] = Cell { cell_type: CellTypes::Solid, color: (0,0,0) };

        sim
    }

    pub fn get_cell(&self, x: usize, y: usize) -> &Cell {
        &self.grid[y * self.config.height + x]
    }

    pub fn step(&mut self) {
        let mut rng = rand::rng();

        self.grid.shuffle(&mut rng);
    }

    pub fn integrate_particles(&mut self, dt:f32,gravity: (f32,f32)) {
        let (gx,gy) = gravity;
        for i in 0..self.config.max_particles{

            let idx_x = 2 * i as usize;
            let idx_y = (2 * i + 1 )as usize;

            self.particle_vel[idx_x] += dt * gx;
            self.particle_vel[idx_y] += dt * gy;

            self.particle_pos[idx_x] += self.particle_vel[idx_x] * dt;
            self.particle_pos[idx_y] += self.particle_vel[idx_y] * dt;
            
        }

    }

    // TODO: Push Particles Apart
    // TODO: Handle Particle Collisions
    // TODO: Transfer Velocities
    // TODO: Update particle density
    // TODO: solve incompressibility
    // TODO: update cell colors
    // TODO: Set Sci Color whatever that means
    // TODO: simulate
}
