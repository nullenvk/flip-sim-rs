#![no_std]
#![no_main]

#[global_allocator]
static ALLOCATOR: emballoc::Allocator<58000> = emballoc::Allocator::new();

#[macro_use]
extern crate alloc;
pub mod simulation;
pub mod config;

use embassy_time::Timer;
use simulation::*;
use config::*;
use embassy_executor::Spawner;
use embassy_stm32::{Config, Peri, adc::AdcChannel, gpio::{AnyPin, Flex, Output, Pin}, i2c::{self, Master}, mode::Blocking, rcc::{Pll, PllRDiv::DIV2, PllSource}, time::Hertz};
use {defmt_rtt as _, panic_probe as _};
use embassy_stm32::i2c::I2c;
use num_traits::Float;
use defmt::info;

#[embassy_executor::main]
async fn main(_spawner: Spawner) {

    let mut syscfg = Config::default();
    syscfg.rcc.hsi = true;
    syscfg.rcc.pll = Some(Pll { source: PllSource::HSI, mul: embassy_stm32::rcc::PllMul::MUL10, prediv: embassy_stm32::rcc::PllPreDiv::DIV1, divr: Some(DIV2), divq: None, divp: None });
    syscfg.rcc.sys = embassy_stm32::rcc::Sysclk::PLL1_R;

    let p = embassy_stm32::init(syscfg);



    let sim_config = CONFIG.clone();
    let mut runtime_config = INITIAL_RUNTIME_CONFIG.clone();

    let mut sim = Simulation::new(&sim_config);
    // ---------- NOWY KSZTAŁT: KOŁO ----------
    let cx = sim.f_num_x as f32 * sim.h * 0.5; // środek domeny X
    let cy = sim.f_num_y as f32 * sim.h * 0.5; // środek domeny Y
    let radius = (sim.f_num_x.min(sim.f_num_y) as f32 * sim.h) * 0.45; // 45% krótszego boku

    // Ustaw komórki: wewnątrz koła -> s=1.0, na zewnątrz -> s=0.0 (Solid)
    for x in 0..sim.f_num_x {
        for y in 0..sim.f_num_y {
            let cell_center_x = (x as f32 + 0.5) * sim.h;
            let cell_center_y = (y as f32 + 0.5) * sim.h;
            let dx = cell_center_x - cx;
            let dy = cell_center_y - cy;
            let in_circle = dx * dx + dy * dy <= radius * radius;

            let cell_nr = x * sim.f_num_y + y;
            sim.grid[cell_nr].s = if in_circle { 1.0 } else { 0.0 };
            sim.grid[cell_nr].cell_type = if in_circle {
                cell::CellTypes::Gas
            } else {
                cell::CellTypes::Solid
            };
        }
    }

    // ---------- NOWE CZĄSTKI W KOLE ----------
    // Wyczyść stare cząstki
    sim.num_particles = 0;
    let r = CONFIG.particle_radius;
    let dx = 2.0 * r;
    let dy = (3.0_f32).sqrt() / 2.0 * dx;

    // Ile cząstek zmieści się w prostokącie opisującym koło (przybliżenie)
    let num_x = ((2.0 * radius - 2.0 * r) / dx).floor() as usize;
    let num_y = ((2.0 * radius - 2.0 * r) / dy).floor() as usize;
    let start_x = cx - radius + r;
    let start_y = cy - radius + r;

    let mut p_idx = 0;
    'spawn: for j in 0..num_y {
        for i in 0..num_x {
            if p_idx >= CONFIG.max_particles {
                break 'spawn;
            }
            let px = start_x + dx * i as f32 + if j % 2 == 0 { 0.0 } else { r };
            let py = start_y + dy * j as f32;

            // sprawdź, czy cząstka jest wewnątrz koła
            if (px - cx) * (px - cx) + (py - cy) * (py - cy) <= (radius - r) * (radius - r) {
                let jitter = if p_idx % 2 == 0 { 1e-4 } else { -1e-4 };
                sim.particles[p_idx].x = px + jitter;
                sim.particles[p_idx].y = py;
                p_idx += 1;
            }
        }
    }
    sim.num_particles = p_idx;
    
    let mut pins:[Flex;9] = [Flex::new(p.PA0),Flex::new(p.PA1),Flex::new(p.PA3),Flex::new(p.PA4),Flex::new(p.PA5),Flex::new(p.PA6),Flex::new(p.PA7),Flex::new(p.PA8),Flex::new(p.PA11)];
    let lut: [[(u8, u8);8];9] = [[(8, 7), (6, 8), (5, 6), (4, 5), (3, 4), (2, 3), (1, 2), (0, 1)], [(7, 8), (5, 7), (6, 5), (3, 6), (4, 3), (1, 4), (2, 1), (0, 2)], [(5, 8), (7, 5), (3, 7), (6, 3), (1, 6), (4, 1), (0, 4), (2, 0)], [(8, 5), (3, 8), (7, 3), (1, 7), (6, 1), (0, 6), (4, 0), (2, 4)], [(3, 5), (8, 3), (1, 8), (7, 1), (0, 7), (6, 0), (2, 6), (4, 2)], [(5, 3), (1, 5), (8, 1), (0, 8), (7, 0), (2, 7), (6, 2), (4, 6)], [(1, 3), (5, 1), (0, 5), (8, 0), (2, 8), (7, 2), (4, 7), (6, 4)], [(3, 1), (0, 3), (5, 0), (2, 5), (8, 2), (4, 8), (7, 4), (6, 7)], [(1, 0), (3, 0), (3, 2), (5, 2), (5, 4), (8, 4), (8, 6), (7, 6)]];
    
    loop{
        for r in 0..lut.len(){
            for c in 0..lut[0].len(){
                let (i,j) = lut[r][c];
                let i = i as usize;
                let j = j as usize;
                pins[i].set_as_output(embassy_stm32::gpio::Speed::High);
                pins[j].set_as_output(embassy_stm32::gpio::Speed::High);
                pins[i].set_high();
                pins[j].set_low();
                Timer::after_micros(500000).await;
                pins[j].set_as_analog();
                pins[i].set_as_analog();
            }
        }
        // for i in 0..pins.len(){
        //     for j in 0..pins.len(){
        //         if i != j{
        //             pins[i].set_as_output(embassy_stm32::gpio::Speed::High);
        //             pins[j].set_as_output(embassy_stm32::gpio::Speed::High);
        //             pins[i].set_high();
        //             pins[j].set_low();
        //             Timer::after_micros(500000).await;
        //             pins[j].set_as_analog();
        //             pins[i].set_as_analog();
        //         }
        //     }
        // }   
        // Timer::after_micros(5000000*10).await;
        // sim.simulate(&runtime_config);
    }
}
