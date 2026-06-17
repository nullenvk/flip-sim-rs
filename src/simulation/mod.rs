pub mod cell;
pub mod config;
pub mod particle;

use cell::*;
use config::*;
use particle::*;

#[derive(Debug)]
pub struct Simulation {
    pub config: Config,

    pub grid: Vec<Cell>,

    pub f_num_x: usize,
    pub f_num_y: usize,
    pub f_num_cells: usize,
    pub h: f32,
    pub f_inv_spacing: f32,

    pub particles: Vec<Particle>,

    pub particle_rest_density: f32,
    pub p_inv_spacing: f32,
    pub p_num_x: usize,
    pub p_num_y: usize,
    pub p_num_cells: usize,

    pub num_cell_particles: Vec<i32>,
    pub first_cell_particle: Vec<usize>,
    pub cell_particle_ids: Vec<usize>,
    pub particle_cell_nrs: Vec<usize>,

    pub num_particles: usize,
}

impl Simulation {
    pub fn new(config: &Config) -> Simulation {
        // let total_cells = config.width * config.height;
        let f_num_x = (config.width as f32 / config.spacing).floor() as usize + 1;
        let f_num_y = (config.height as f32 / config.spacing).floor() as usize + 1;
        let f_num_cells = f_num_x * f_num_y;
        let h = f32::max(
            config.width as f32 / f_num_x as f32,
            config.height as f32 / f_num_y as f32,
        );
        let f_inv_spacing = 1.0 / h;

        let p_inv_spacing = 1.0 / (2.2 * config.particle_radius);
        let p_num_x = (config.width as f32 * p_inv_spacing).floor() as usize + 1;
        let p_num_y = (config.height as f32 * p_inv_spacing).floor() as usize + 1;
        let p_num_cells = p_num_x * p_num_y;

        let grid = vec![Cell::default(); f_num_cells];

        let mut particles = vec![Particle::default(); config.max_particles];

        // spwaning
        // let mut num_particles = 0;
        // let r = config.particle_radius;
        // let dx = 2.0 * r;
        // let dy = f32::sqrt(3.0) / 2.0 * dx;

        // let mut p_idx = 0;
        // let particles_per_row = 50; // Adjust to make the fluid block wider or narrower
        // let spawn_height = config.max_particles / particles_per_row + 1 + 200;

        // 'spawn: for j in (config.height as i32/2)..spawn_height as i32 {
        //     for i in 0..particles_per_row {
        //         if p_idx >= config.max_particles {
        //             break 'spawn;
        //         }

        //         let jitter = if p_idx % 2 == 0 { 1e-4 } else { -1e-4 };

        //         particles[p_idx].x = h + r + dx * i as f32 + if j % 2 == 0 { 0.0 } else { r } + jitter;
        //         particles[p_idx].y = h + r + dy * j as f32;
        //         particles[p_idx].color = (0.0, 0.5, 1.0);

        //         p_idx += 1;
        //         num_particles += 1;
        //     }
        // }

        let r = config.particle_radius;
        let h = f32::max(
            config.width as f32 / f_num_x as f32,
            config.height as f32 / f_num_y as f32,
        );
        let dx = 2.0 * r;
        let dy = (3.0_f32).sqrt() / 2.0 * dx;

        let blob_width_ratio = 0.4;
        let blob_height_ratio = 0.3;

        let max_x =
            ((blob_width_ratio * config.width as f32 - 2.0 * h - 2.0 * r) / dx).floor() as i32;
        let max_y =
            ((blob_height_ratio * config.height as f32 - 2.0 * h - 2.0 * r) / dy).floor() as i32;

        let num_particles_to_spawn = (max_x as usize * max_y as usize).min(config.max_particles);
        let num_y = (num_particles_to_spawn as f32 / max_x as f32).ceil() as usize;
        let num_x = max_x as usize;

        let start_x = (config.width as f32 - (num_x as f32 * dx)) / 2.0;
        let start_y = (config.height as f32 - (num_y as f32 * dy)) / 2.0;

        let mut p_idx = 0;
        let mut num_particles = 0;

        'spawn: for j in 0..num_y {
            for i in 0..num_x {
                if p_idx >= config.max_particles {
                    break 'spawn;
                }

                let jitter = if p_idx % 2 == 0 { 1e-4 } else { -1e-4 };

                particles[p_idx].x =
                    start_x + r + dx * i as f32 + if j % 2 == 0 { 0.0 } else { r } + jitter;
                particles[p_idx].y = start_y + r + dy * j as f32;
                particles[p_idx].color = (0.0, 0.5, 1.0);

                p_idx += 1;
                num_particles += 1;
            }
        }

        Simulation {
            config: config.clone(),
            grid,
            particles,

            f_num_x,
            f_num_y,
            f_num_cells,
            h,
            f_inv_spacing,

            particle_rest_density: 0.0,
            p_inv_spacing,
            p_num_x,
            p_num_y,
            p_num_cells,

            num_cell_particles: vec![0; p_num_cells],
            first_cell_particle: vec![0; p_num_cells + 1],
            cell_particle_ids: vec![0; config.max_particles],
            particle_cell_nrs: vec![0; config.max_particles],
            num_particles,
        }
    }

    pub fn get_cell(&self, x: usize, y: usize) -> &Cell {
        &self.grid[x * self.f_num_y + y]
    }

    pub fn integrate_particles(&mut self, dt: f32, gravity: (f32, f32)) {
        let (gx, gy) = gravity;
        // we need take() because num_particles != max_particles
        for part in self.particles.iter_mut().take(self.num_particles) {
            part.vx += dt * gx;
            part.vy += dt * gy;
            part.x += part.vx * dt;
            part.y += part.vy * dt;
        }
    }

    // todo: jakoś to zrefactorować
    pub fn push_particles_apart(&mut self, num_iters: usize) {
        let min_dist = 2.0 * self.config.particle_radius;
        let min_dist2 = min_dist * min_dist;

        for _ in 0..num_iters {
            self.num_cell_particles.fill(0);

            for i in 0..self.num_particles {
                let p = &self.particles[i];
                let xi = ((p.x * self.p_inv_spacing).floor() as i32)
                    .clamp(0, (self.p_num_x - 1) as i32) as usize;
                let yi = ((p.y * self.p_inv_spacing).floor() as i32)
                    .clamp(0, (self.p_num_y - 1) as i32) as usize;
                let cell_nr = xi * self.p_num_y + yi;

                self.num_cell_particles[cell_nr] += 1;
                self.particle_cell_nrs[i] = cell_nr;
            }

            let mut first = 0;
            for i in 0..self.p_num_cells {
                first += self.num_cell_particles[i];
                self.first_cell_particle[i] = first as usize;
            }
            self.first_cell_particle[self.p_num_cells] = first as usize;

            for i in 0..self.num_particles {
                let cell_nr = self.particle_cell_nrs[i];
                self.first_cell_particle[cell_nr] -= 1;
                let idx = self.first_cell_particle[cell_nr];
                self.cell_particle_ids[idx] = i;
            }

            for i in 0..self.num_particles {
                let mut px = self.particles[i].x;
                let mut py = self.particles[i].y;

                let pxi =
                    ((px * self.p_inv_spacing).floor() as i32).clamp(0, (self.p_num_x - 1) as i32);
                let pyi =
                    ((py * self.p_inv_spacing).floor() as i32).clamp(0, (self.p_num_y - 1) as i32);

                let x0 = (pxi - 1).max(0);
                let y0 = (pyi - 1).max(0);
                let x1 = (pxi + 1).min(self.p_num_x as i32 - 1);
                let y1 = (pyi + 1).min(self.p_num_y as i32 - 1);

                for xi in x0..=x1 {
                    for yi in y0..=y1 {
                        let cell_nr = (xi * self.p_num_y as i32 + yi) as usize;
                        let start = self.first_cell_particle[cell_nr];
                        let end = self.first_cell_particle[cell_nr + 1];

                        for j in start..end {
                            let id2 = self.cell_particle_ids[j];
                            if i == id2 {
                                continue;
                            }

                            let px2 = self.particles[id2].x;
                            let py2 = self.particles[id2].y;

                            let dx = px - px2;
                            let dy = py - py2;
                            let d2 = dx * dx + dy * dy;

                            if d2 < min_dist2 && d2 > 1e-8 {
                                let d = d2.sqrt();
                                let push = 0.5 * (min_dist - d) / d;

                                // rozsuń obie cząstki
                                if i < id2 {
                                    let (left, right) = self.particles.split_at_mut(id2);
                                    left[i].x += dx * push;
                                    left[i].y += dy * push;
                                    right[0].x -= dx * push;
                                    right[0].y -= dy * push;
                                    px += dx * push;
                                    py += dy * push;
                                } else {
                                    let (left, right) = self.particles.split_at_mut(i);
                                    right[0].x += dx * push;
                                    right[0].y += dy * push;
                                    left[id2].x -= dx * push;
                                    left[id2].y -= dy * push;
                                    px += dx * push;
                                    py += dy * push;
                                }
                            }
                        }
                    }
                }

                self.particles[i].x = px;
                self.particles[i].y = py;
            }
        }
    }

    pub fn update_particle_density(&mut self) {
        let n = self.f_num_y;
        let h = self.h;
        let h1 = self.f_inv_spacing;
        let h2 = 0.5 * h;

        self.grid.iter_mut().for_each(|cell| {
            cell.particle_density = 0.0;
        });

        for i in 0..self.num_particles {
            let p = &self.particles[i];
            let max_bound_x = f32::max(h, (self.f_num_x as f32 - 1.0) * h);
            let max_bound_y = f32::max(h, (self.f_num_y as f32 - 1.0) * h);
            let x = p.x.clamp(h, max_bound_x);
            let y = p.y.clamp(h, max_bound_y);

            let x0 = (((x - h2) * h1).floor() as usize).min(self.f_num_x.saturating_sub(2));
            let tx = ((x - h2) - x0 as f32 * h) * h1;
            let x1 = (x0 + 1).min(self.f_num_x.saturating_sub(2));

            let y0 = (((y - h2) * h1).floor() as usize).min(self.f_num_y.saturating_sub(2));
            let ty = ((y - h2) - y0 as f32 * h) * h1;
            let y1 = (y0 + 1).min(self.f_num_y.saturating_sub(2));

            let sx = 1.0 - tx;
            let sy = 1.0 - ty;

            self.grid[x0 * n + y0].particle_density += sx * sy;
            self.grid[x1 * n + y0].particle_density += tx * sy;
            self.grid[x1 * n + y1].particle_density += tx * ty;
            self.grid[x0 * n + y1].particle_density += sx * ty;
        }

        if self.particle_rest_density == 0.0 {
            let mut sum = 0.0;
            let mut num_fluid_cells = 0;

            for i in 0..self.f_num_cells {
                if self.grid[i].cell_type == CellTypes::Liquid {
                    sum += self.grid[i].particle_density;
                    num_fluid_cells += 1;
                }
            }
            if num_fluid_cells > 0 {
                self.particle_rest_density = sum / num_fluid_cells as f32;
            }
        }
    }

    pub fn handle_particle_collisions(&mut self, obstacle_x: f32, obstacle_y: f32, obstacle_radius: f32, obstacle_vel_x: f32, obstacle_vel_y: f32) {
        // Parametry okrągłego zbiornika – muszą być zgodne z tymi w main.rs
        let cx = (self.f_num_x as f32) * self.h * 0.5;
        let cy = (self.f_num_y as f32) * self.h * 0.5;
        let radius = (self.f_num_x.min(self.f_num_y) as f32 * self.h) * 0.45;
        let inner_radius = radius - self.config.particle_radius;   // max odległość środka cząstki od centrum

        for i in 0..self.num_particles {
            let mut x = self.particles[i].x;
            let mut y = self.particles[i].y;
            // Kolizja z okrągłą ścianą
            let dx = x - cx;
            let dy = y - cy;
            let dist = dx.hypot(dy);
            if dist > inner_radius {
                // Wypchnij cząstkę do środka
                let nx = dx / dist;
                let ny = dy / dist;
                x = cx + nx * inner_radius;
                y = cy + ny * inner_radius;
                // Zatrzymaj prędkość (możesz też tylko stłumić, ale zerowanie działa stabilniej)
                self.particles[i].vx = 0.0;
                self.particles[i].vy = 0.0;
            }

            // Aktualizacja pozycji
            self.particles[i].x = x;
            self.particles[i].y = y;
        }
    }

    //oryginalna, przepisana z js
    // pub fn handle_particle_collisions(&mut self, obstacle_x: f32, obstacle_y: f32, obstacle_radius: f32, obstacle_vel_x: f32, obstacle_vel_y: f32) {
    //     let h = 1.0 / self.f_inv_spacing;
    //     let r = self.config.particle_radius;
    //     let min_dist = obstacle_radius + r;
    //     let min_dist2 = min_dist * min_dist;

    //     let min_x = h + r;
    //     let max_x = (self.f_num_x as f32 - 1.0) * h - r;
    //     let min_y = h + r;
    //     let max_y = (self.f_num_y as f32 - 1.0) * h - r;

    //     for i in 0..self.num_particles {
    //         let mut x = self.particles[i].x;
    //         let mut y = self.particles[i].y;

    //         let dx = x - obstacle_x;
    //         let dy = y - obstacle_y;
    //         let d2 = dx * dx + dy * dy;

    //         if d2 < min_dist2 {

    //             let d = d2.sqrt().max(1e-8); // unikaj dzielenia przez 0
    //             let penetration = min_dist - d;
    //             x += (dx / d) * penetration;
    //             y += (dy / d) * penetration;

    //             self.particles[i].vx = obstacle_vel_x;
    //             self.particles[i].vy = obstacle_vel_y;
    //         }

    //         if x < min_x {
    //             x = min_x;
    //             self.particles[i].vx = 0.0;
    //         }
    //         if x > max_x {
    //             x = max_x;
    //             self.particles[i].vx = 0.0;
    //         }
    //         if y < min_y {
    //             y = min_y;
    //             self.particles[i].vy = 0.0;
    //         }
    //         if y > max_y {
    //             y = max_y;
    //             self.particles[i].vy = 0.0;
    //         }

    //         self.particles[i].x = x;
    //         self.particles[i].y = y;
    //     }
    // }

    pub fn transfer_velocities(&mut self, to_grid: bool, flip_ratio: f32) {
        let n = self.f_num_y;
        let h = self.h;
        let h1 = self.f_inv_spacing;
        let h2 = 0.5 * h;

        if to_grid {
            for cell in self.grid.iter_mut() {
                cell.prev_u = cell.u;
                cell.prev_v = cell.v;
                cell.u = 0.0;
                cell.v = 0.0;
                cell.du = 0.0;
                cell.dv = 0.0;
                cell.cell_type = if cell.s == 0.0 {
                    CellTypes::Solid
                } else {
                    CellTypes::Gas
                };
            }

            for i in 0..self.num_particles {
                let p = &self.particles[i];
                let xi = ((p.x * h1).floor() as i32).clamp(0, self.f_num_x as i32 - 1) as usize;
                let yi = ((p.y * h1).floor() as i32).clamp(0, self.f_num_y as i32 - 1) as usize;
                let cell_nr = xi * n + yi;
                if self.grid[cell_nr].cell_type == CellTypes::Gas {
                    self.grid[cell_nr].cell_type = CellTypes::Liquid;
                }
            }
        }

        for component in 0..2 {
            let dx = if component == 0 { 0.0 } else { h2 };
            let dy = if component == 0 { h2 } else { 0.0 };

            for i in 0..self.num_particles {
                let p = &self.particles[i];
                let max_bound_x = f32::max(h, (self.f_num_x as f32 - 1.0) * h);
                let max_bound_y = f32::max(h, (self.f_num_y as f32 - 1.0) * h);
                let x = p.x.clamp(h, max_bound_x);
                let y = p.y.clamp(h, max_bound_y);

                let x0 = (((x - dx) * h1).floor() as usize).min(self.f_num_x.saturating_sub(2));
                let tx = ((x - dx) - x0 as f32 * h) * h1;
                let x1 = (x0 + 1).min(self.f_num_x.saturating_sub(2));

                let y0 = (((y - dy) * h1).floor() as usize).min(self.f_num_y.saturating_sub(2));
                let ty = ((y - dy) - y0 as f32 * h) * h1;
                let y1 = (y0 + 1).min(self.f_num_y.saturating_sub(2));

                let sx = 1.0 - tx;
                let sy = 1.0 - ty;
                let d0 = sx * sy;
                let d1 = tx * sy;
                let d2 = tx * ty;
                let d3 = sx * ty;

                let nr0 = x0 * n + y0;
                let nr1 = x1 * n + y0;
                let nr2 = x1 * n + y1;
                let nr3 = x0 * n + y1;

                if to_grid {
                    let pv = if component == 0 { p.vx } else { p.vy };
                    if component == 0 {
                        self.grid[nr0].u += pv * d0;
                        self.grid[nr0].du += d0;
                        self.grid[nr1].u += pv * d1;
                        self.grid[nr1].du += d1;
                        self.grid[nr2].u += pv * d2;
                        self.grid[nr2].du += d2;
                        self.grid[nr3].u += pv * d3;
                        self.grid[nr3].du += d3;
                    } else {
                        self.grid[nr0].v += pv * d0;
                        self.grid[nr0].dv += d0;
                        self.grid[nr1].v += pv * d1;
                        self.grid[nr1].dv += d1;
                        self.grid[nr2].v += pv * d2;
                        self.grid[nr2].dv += d2;
                        self.grid[nr3].v += pv * d3;
                        self.grid[nr3].dv += d3;
                    }
                } else {
                    let offset = if component == 0 { n } else { 1 };

                    let valid0 = if self.grid[nr0].cell_type != CellTypes::Gas
                        || (nr0 >= offset && self.grid[nr0 - offset].cell_type != CellTypes::Gas)
                    {
                        1.0
                    } else {
                        0.0
                    };
                    let valid1 = if self.grid[nr1].cell_type != CellTypes::Gas
                        || (nr1 >= offset && self.grid[nr1 - offset].cell_type != CellTypes::Gas)
                    {
                        1.0
                    } else {
                        0.0
                    };
                    let valid2 = if self.grid[nr2].cell_type != CellTypes::Gas
                        || (nr2 >= offset && self.grid[nr2 - offset].cell_type != CellTypes::Gas)
                    {
                        1.0
                    } else {
                        0.0
                    };
                    let valid3 = if self.grid[nr3].cell_type != CellTypes::Gas
                        || (nr3 >= offset && self.grid[nr3 - offset].cell_type != CellTypes::Gas)
                    {
                        1.0
                    } else {
                        0.0
                    };
                    // let offset = if component == 0 { n } else { 1 };

                    // let valid0 = if self.grid[nr0].cell_type != CellTypes::Gas || self.grid[nr0.saturating_sub(offset)].cell_type != CellTypes::Gas { 1.0 } else { 0.0 };
                    // let valid1 = if self.grid[nr1].cell_type != CellTypes::Gas || self.grid[nr1.saturating_sub(offset)].cell_type != CellTypes::Gas { 1.0 } else { 0.0 };
                    // let valid2 = if self.grid[nr2].cell_type != CellTypes::Gas || self.grid[nr2.saturating_sub(offset)].cell_type != CellTypes::Gas { 1.0 } else { 0.0 };
                    // let valid3 = if self.grid[nr3].cell_type != CellTypes::Gas || self.grid[nr3.saturating_sub(offset)].cell_type != CellTypes::Gas { 1.0 } else { 0.0 };

                    let v = if component == 0 { p.vx } else { p.vy };
                    let d = valid0 * d0 + valid1 * d1 + valid2 * d2 + valid3 * d3;

                    if d > 0.0 {
                        let (f0, f1, f2, f3) = if component == 0 {
                            (
                                self.grid[nr0].u,
                                self.grid[nr1].u,
                                self.grid[nr2].u,
                                self.grid[nr3].u,
                            )
                        } else {
                            (
                                self.grid[nr0].v,
                                self.grid[nr1].v,
                                self.grid[nr2].v,
                                self.grid[nr3].v,
                            )
                        };

                        let (pf0, pf1, pf2, pf3) = if component == 0 {
                            (
                                self.grid[nr0].prev_u,
                                self.grid[nr1].prev_u,
                                self.grid[nr2].prev_u,
                                self.grid[nr3].prev_u,
                            )
                        } else {
                            (
                                self.grid[nr0].prev_v,
                                self.grid[nr1].prev_v,
                                self.grid[nr2].prev_v,
                                self.grid[nr3].prev_v,
                            )
                        };

                        let pic_v = (valid0 * d0 * f0
                            + valid1 * d1 * f1
                            + valid2 * d2 * f2
                            + valid3 * d3 * f3)
                            / d;
                        let corr = (valid0 * d0 * (f0 - pf0)
                            + valid1 * d1 * (f1 - pf1)
                            + valid2 * d2 * (f2 - pf2)
                            + valid3 * d3 * (f3 - pf3))
                            / d;
                        let flip_v = v + corr;

                        if component == 0 {
                            self.particles[i].vx = (1.0 - flip_ratio) * pic_v + flip_ratio * flip_v;
                        } else {
                            self.particles[i].vy = (1.0 - flip_ratio) * pic_v + flip_ratio * flip_v;
                        }
                    }
                }
            }

            if to_grid {
                for i in 0..self.f_num_cells {
                    if component == 0 {
                        if self.grid[i].du > 0.0 {
                            self.grid[i].u /= self.grid[i].du;
                        }
                    } else {
                        if self.grid[i].dv > 0.0 {
                            self.grid[i].v /= self.grid[i].dv;
                        }
                    }
                }

                for i in 0..self.f_num_x {
                    for j in 0..self.f_num_y {
                        let solid = self.grid[i * n + j].cell_type == CellTypes::Solid;
                        if component == 0 {
                            if solid
                                || (i > 0
                                    && self.grid[(i - 1) * n + j].cell_type == CellTypes::Solid)
                            {
                                self.grid[i * n + j].u = self.grid[i * n + j].prev_u;
                            }
                        } else {
                            if solid
                                || (j > 0
                                    && self.grid[i * n + j.saturating_sub(1)].cell_type
                                        == CellTypes::Solid)
                            {
                                self.grid[i * n + j].v = self.grid[i * n + j].prev_v;
                            }
                        }
                    }
                }
            }
        }
    }

    pub fn solve_incompressibility(
        &mut self,
        num_iters: usize,
        dt: f32,
        over_relaxation: f32,
        compensate_drift: bool,
    ) {
        for cell in self.grid.iter_mut() {
            cell.prev_u = cell.u;
            cell.prev_v = cell.v;
            cell.p = 0.0; // przy okazji zeruj ciśnienie, bo JS też to robi
        }

        let n = self.f_num_y;
        let cp = self.config.density * self.h / dt;

        for _ in 0..num_iters {
            // self.read_grid.copy_from_slice(&self.read_grid);

            for i in 1..self.f_num_x - 1 {
                for j in 1..self.f_num_y - 1 {
                    let center = i * n + j;
                    if self.grid[center].cell_type != CellTypes::Liquid {
                        continue;
                    }

                    let left = (i - 1) * n + j;
                    let right = (i + 1) * n + j;
                    let bottom = i * n + j - 1;
                    let top = i * n + j + 1;

                    let sx0 = self.grid[left].s;
                    let sx1 = self.grid[right].s;
                    let sy0 = self.grid[bottom].s;
                    let sy1 = self.grid[top].s;
                    let s = sx0 + sx1 + sy0 + sy1;

                    if s < 0.1 {
                        continue;
                    }

                    let mut div = self.grid[right].u - self.grid[center].u + self.grid[top].v
                        - self.grid[center].v;

                    if self.particle_rest_density > 0.0 && compensate_drift {
                        let k = 1.0;
                        let compression =
                            self.grid[center].particle_density - self.particle_rest_density;
                        if compression > 0.0 {
                            div -= k * compression;
                        }
                    }

                    let mut p = -div / s;
                    p *= over_relaxation;

                    self.grid[center].p += cp * p;
                    self.grid[center].u -= sx0 * p;
                    self.grid[right].u += sx1 * p;
                    self.grid[center].v -= sy0 * p;
                    self.grid[top].v += sy1 * p;

                    // self.read_grid[center].p += cp * p;
                    // self.read_grid[center].u -= sx0 * p;
                    // self.read_grid[right].u  += sx1 * p;
                    // self.read_grid[center].v -= sy0 * p;
                    // self.read_grid[top].v    += sy1 * p;
                }
            }
            // std::mem::swap(&mut self.read_grid, &mut self.read_grid); borrow checker fix unfix
        }
    }

    pub fn update_cell_colors(&mut self, mono_mode: bool) {
        for i in 0..self.f_num_cells {
            if self.grid[i].cell_type == CellTypes::Solid {
                self.grid[i].color = (0.5, 0.5, 0.5);
            } else if self.grid[i].cell_type == CellTypes::Liquid {
                if mono_mode {
                    // Stały kolor płynu, np. niebieski
                    self.grid[i].color = (1.0, 0.4, 1.0);
                } else {
                    let mut d = self.grid[i].particle_density;
                    if self.particle_rest_density > 0.0 {
                        d /= self.particle_rest_density;
                    }
                    let mut val = d.clamp(0.0, 1.99);
                    val /= 2.0;
                    let m = 0.25;
                    let num = (val / m).floor() as i32;
                    let s = (val - num as f32 * m) / m;
                    self.grid[i].color = match num {
                        0 => (0.0, s, 1.0),
                        1 => (0.0, 1.0, 1.0 - s),
                        2 => (s, 1.0, 0.0),
                        3 => (1.0, 1.0 - s, 0.0),
                        _ => (0.0, 0.0, 0.0),
                    };
                }
            } else {
                self.grid[i].color = (0.0, 0.0, 0.0); // Gas – czarne
            }
        }
    }

    pub fn simulate(&mut self, runtime: &RuntimeConfig) {
        self.integrate_particles(runtime.dt, runtime.gravity);

        if runtime.separate_particles {
            self.push_particles_apart(runtime.num_particle_iters);
        }
        self.handle_particle_collisions(
            runtime.obstacle_x,
            runtime.obstacle_y,
            runtime.obstacle_radius,
            runtime.obstacle_vel_x,
            runtime.obstacle_vel_y,
        );

        self.transfer_velocities(true, runtime.flip_ratio);
        self.update_particle_density();
        self.solve_incompressibility(
            runtime.num_pressure_iters,
            runtime.dt,
            runtime.over_relaxation,
            runtime.compensate_drift,
        );
        self.transfer_velocities(false, runtime.flip_ratio);

        self.update_cell_colors(runtime.mono_mode);
    }
}
