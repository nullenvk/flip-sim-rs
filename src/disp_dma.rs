use core::mem::forget;

use embassy_stm32::dma::{Channel, Priority, TransferOptions};
use embassy_stm32::time::Hertz;
use embassy_stm32::{Peripherals, bind_interrupts, dma, pac, peripherals};

use embassy_stm32::timer::low_level::{RoundTo, Timer as HwTimer};
use embassy_stm32::peripherals::{DMA1_CH2, DMA1_CH5, TIM2};
use embassy_stm32::timer::{Ch1, Channel as TimCh, Dma as CcDma, UpDma};

static LUT: [[(u8, u8);8];9] = [[(8, 7), (6, 8), (5, 6), (4, 5), (3, 4), (2, 3), (1, 2), (0, 1)], [(7, 8), (5, 7), (6, 5), (3, 6), (4, 3), (1, 4), (2, 1), (0, 2)], [(5, 8), (7, 5), (3, 7), (6, 3), (1, 6), (4, 1), (0, 4), (2, 0)], [(8, 5), (3, 8), (7, 3), (1, 7), (6, 1), (0, 6), (4, 0), (2, 4)], [(3, 5), (8, 3), (1, 8), (7, 1), (0, 7), (6, 0), (2, 6), (4, 2)], [(5, 3), (1, 5), (8, 1), (0, 8), (7, 0), (2, 7), (6, 2), (4, 6)], [(1, 3), (5, 1), (0, 5), (8, 0), (2, 8), (7, 2), (4, 7), (6, 4)], [(3, 1), (0, 3), (5, 0), (2, 5), (8, 2), (4, 8), (7, 4), (6, 7)], [(1, 0), (3, 0), (3, 2), (5, 2), (5, 4), (8, 4), (8, 6), (7, 6)]];
static PINS_VIRT_TO_REAL:[u8;9] = [0u8, 1u8, 3u8, 4u8, 5u8, 6u8, 7u8, 8u8, 11u8];

static mut PINS_MODE: [u32; 72] = [0u32; 72];
static mut PINS_ODR: [u32; 72] = [0u32; 72];
bind_interrupts!(struct Irqs {
    DMA1_CHANNEL2 => dma::InterruptHandler<peripherals::DMA1_CH2>;
    DMA1_CHANNEL5 => dma::InterruptHandler<peripherals::DMA1_CH5>;
});

pub fn set_led(r: usize, c: usize, s: bool) {
    unsafe { PINS_ODR[r * 8 + c] = u32::from(s) << PINS_VIRT_TO_REAL[LUT[r][c].0 as usize]; }
}

pub fn setup_permanent_dma_display(p: &Peripherals) {
    for i in 0..72 {
        // The LED is lit, so just go through the LUT and get the values.
        let (_a, _b) = LUT[i / 8][i % 8];
        let a = PINS_VIRT_TO_REAL[_a as usize];
        let b = PINS_VIRT_TO_REAL[_b as usize];
        unsafe {
            PINS_MODE[i] = (1 << (a << 1)) | ( 1 << (b << 1));
        }
    }

    let pins_odr_bor = unsafe { &*& raw const PINS_ODR } ;
    let pins_mode_bor = unsafe { &*& raw const PINS_MODE };
 
    // TIM2: period = one step. CC1 at CNT == ARR fires one tick before the update.
    let tim2 = HwTimer::new(unsafe { p.TIM2.clone_unchecked() });
    tim2.set_frequency(Hertz::khz(1000), RoundTo::Slower);
    tim2.set_compare_value(TimCh::Ch1, tim2.get_max_compare_value());
    tim2.set_cc_dma_enable_state(TimCh::Ch1, true);
    tim2.enable_update_dma(true);
 
    let odr_req = <DMA1_CH5 as CcDma<TIM2, Ch1>>::request(&p.DMA1_CH5);
    let moder_req = <DMA1_CH2 as UpDma<TIM2>>::request(&p.DMA1_CH2);
    let mut odr_ch = Channel::new(unsafe { p.DMA1_CH5.clone_unchecked() }, Irqs);
    let mut moder_ch = Channel::new(unsafe { p.DMA1_CH2.clone_unchecked() }, Irqs);
 
    let mut opts = TransferOptions::default();
    opts.circular = true;
    opts.complete_transfer_ir = false;
    opts.half_transfer_ir = false;
 
    let mut odr_opts = opts;
    odr_opts.priority = Priority::VeryHigh;
    let mut moder_opts = opts;
    moder_opts.priority = Priority::High;
 
    let gpioa = pac::GPIOA;
    let odr_lo = gpioa.odr().as_ptr() as *mut u32;
    let moder_lo = gpioa.moder().as_ptr() as *mut u32;
 
    let odr_xfer =  unsafe { odr_ch.write(odr_req, pins_odr_bor, odr_lo, odr_opts) };
    let moder_xfer = unsafe { moder_ch.write(moder_req, pins_mode_bor, moder_lo, moder_opts) };
 
    tim2.start();

    forget(odr_xfer);
    forget(moder_xfer);
    forget(tim2);
}
