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
use embassy_stm32::{Config, i2c::{self, Master}, mode::Blocking, rcc::{Pll, PllRDiv::DIV2, PllSource}, time::Hertz};
use {defmt_rtt as _, panic_probe as _};
use embassy_stm32::i2c::I2c;
use num_traits::Float;
use defmt::info;

const SET_COL_ADDR: u8 =  0x15;
const SET_SCROLL_DEACTIVATE: u8 =  0x2E;
const SET_ROW_ADDR: u8 =  0x75;
const SET_CONTRAST: u8 =  0x81;
const SET_SEG_REMAP: u8 =  0xA0;
const SET_DISP_START_LINE: u8 =  0xA1;
const SET_DISP_OFFSET: u8 =  0xA2;
const SET_DISP_MODE: u8 =  0xA4;
const SET_MUX_RATIO: u8 =  0xA8;
const SET_FN_SELECT_A: u8 =  0xAB;
const SET_DISP: u8 =  0xAE;
const SET_PHASE_LEN: u8 =  0xB1;
const SET_DISP_CLK_DIV: u8 =  0xB3;
const SET_SECOND_PRECHARGE: u8 =  0xB6;
const SET_GRAYSCALE_TABLE: u8 =  0xB8;
const SET_GRAYSCALE_LINEAR: u8 =  0xB9;
const SET_PRECHARGE: u8 =  0xBC;
const SET_VCOM_DESEL: u8 =  0xBE;
const SET_FN_SELECT_B: u8 =  0xD5;
const SET_COMMAND_LOCK: u8 =  0xFD;

const OLED: u8 = 0x78u8 >> 1;
type I2cRef<'a, 'b> = &'a mut I2c<'b, Blocking, Master>;

const CO_CMD: u8 =    0b0000_0000;
const CO_DATA: u8 =   0b0100_0000;
const CO_CONT: u8 =   0b0000_0000;
const CO_SINGLE: u8 = 0b1000_0000;

fn send_init(i2c: I2cRef) {
    i2c.blocking_write(OLED, &[
        CO_CMD | CO_CONT,

        SET_COMMAND_LOCK, 0x12, // Unlock
        SET_DISP, // Display off
        SET_DISP_START_LINE, 0, //0x20,
        SET_DISP_OFFSET, 0, // Set vertical offset by COM from 0~127
        SET_SEG_REMAP, 0b01010001,
        SET_MUX_RATIO, 127,
        SET_FN_SELECT_A, 0x00, // Enable internal VDD regulator
        SET_PHASE_LEN, 0x51, // Phase 1: 1 DCLK, Phase 2: 5 DCLKs
        SET_DISP_CLK_DIV, 0x01, // Divide ratio: 1, Oscillator Frequency: 0
        SET_PRECHARGE, 0x08, // Set pre-charge voltage level: VCOMH
        SET_VCOM_DESEL, 0x07, // Set VCOMH COM deselect voltage level: 0.86*Vcc
        SET_SECOND_PRECHARGE, 0x01, // Second Pre-charge period: 1 DCLK
        SET_FN_SELECT_B, 0x62, // Enable enternal VSL, Enable second precharge
        // Display
        SET_GRAYSCALE_LINEAR, // Use linear greyscale lookup table
        SET_CONTRAST, 0x7f, // Medium brightness
        SET_DISP_MODE, // Normal, inverted
        SET_SCROLL_DEACTIVATE,
        SET_DISP | 1,
    ]).unwrap();
}

fn set_ranges(i2c: I2cRef, start_x: u8, start_y: u8, end_x: u8, end_y: u8) {
    i2c.blocking_write(OLED, &[
        CO_CMD | CO_CONT,
        SET_COL_ADDR, start_x / 2, end_x / 2 - 1,
        SET_ROW_ADDR, start_y, end_y - 1,
    ]).unwrap();
}

fn clear_screen(i2c: I2cRef) {
    let mut data_packet = [0u8; 17];
    data_packet[0] = CO_DATA | CO_CONT;
    set_ranges(i2c, 0, 0, 128, 128);
    for _ in 0..128 {
        for _ in 0..((96 / 16) / 2) {
            i2c.blocking_write(OLED, &data_packet).unwrap();
        }
    }
}

fn send_data_to_screen(data: &[u8], i2c: I2cRef) {
    let mut buffer = [0u8; 17];
    buffer[0] = CO_DATA | CO_CONT;
    let ld = data.len();
    for x in (0..ld).step_by(16) {
        buffer[1..].fill(0);
        buffer[1..].copy_from_slice(&data[x..(x + 16).min(ld)]);
        i2c.blocking_write(OLED, &buffer).unwrap();
    }
}

fn send_sim_data_to_screen(sim: &Simulation, i2c: I2cRef) {
    set_ranges(i2c, 0, 0, sim.config.width as u8, sim.config.height as u8);
    assert_eq!((sim.config.width) % 32, 0);
    let mut buffer = [0u8; 17];
    buffer[0] = CO_DATA | CO_CONT;

    for row in 0..sim.f_num_y {
        for col_root in (0..sim.f_num_x - 1).step_by(2) {
            buffer[(1 +((col_root as u8 % 32) >> 1))as usize] = (sim.get_cell(col_root, row).color << 4)|
                                                                sim.get_cell(col_root + 1, row).color;
            if (col_root % 32) == 30 {
                i2c.blocking_write(OLED, &buffer).unwrap();
                // info!("{:?}",buffer)
            }
        }
    }
}

#[embassy_executor::main]
async fn main(_spawner: Spawner) {

    let mut syscfg = Config::default();
    syscfg.rcc.hsi = true;
    syscfg.rcc.pll = Some(Pll { source: PllSource::HSI, mul: embassy_stm32::rcc::PllMul::MUL10, prediv: embassy_stm32::rcc::PllPreDiv::DIV1, divr: Some(DIV2), divq: None, divp: None });
    syscfg.rcc.sys = embassy_stm32::rcc::Sysclk::PLL1_R;

    let p = embassy_stm32::init(syscfg);
    let mut conf = i2c::Config::default();
    conf.frequency = Hertz::khz(400);
    let mut i2c = I2c::new_blocking(p.I2C1, p.PB6, p.PB7, conf);
    send_init(&mut i2c);
    clear_screen(&mut i2c);

    let screendata = include_bytes!("raw");
    set_ranges(&mut i2c, 0, 0, 96, 96);
    send_data_to_screen(screendata, &mut i2c);

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
    clear_screen(&mut i2c);
    loop{
        send_sim_data_to_screen(&sim, &mut i2c);
        sim.simulate(&runtime_config);
        // TODO: Timer::after_micros(100).await;

        
    }
}
