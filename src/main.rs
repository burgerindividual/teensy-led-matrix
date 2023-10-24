#![feature(abi_unadjusted)]
#![feature(link_llvm_intrinsics)]
#![feature(array_chunks)]
#![feature(slice_flatten)]
#![feature(exclusive_range_pattern)]
#![feature(maybe_uninit_slice)]
#![no_std]
#![no_main]

extern crate alloc;

mod collections;
mod color;
mod driver;
mod framebuffer;
mod intrinsics;
mod peripherals;
mod pins;
mod program;

use core::arch::asm;

use cortex_m::delay::Delay;
use cortex_m::interrupt;
use cortex_m::peripheral::scb::Exception;
use cortex_m::peripheral::syst::SystClkSource;
use cortex_m::peripheral::SCB;
use cortex_m::register::{basepri, faultmask};
use embedded_alloc::Heap;
use teensy4_bsp::board::{prepare_clocks_and_power, ARM_FREQUENCY};
use teensy4_bsp::hal::iomuxc::into_pads;
use teensy4_bsp::pins::t40::*;
use teensy4_bsp::ral::{self, modify_reg, write_reg};
use teensy4_bsp::rt::exception;
#[allow(unused_imports)]
use teensy4_panic as _;

use crate::driver::ScreenDriver;
use crate::intrinsics::{init_heap, yield_cycles};
use crate::program::*;

#[global_allocator]
static mut HEAP: Heap = Heap::empty();

#[teensy4_bsp::rt::entry]
fn main() -> ! {
    // interrupt::disable();
    unsafe {
        asm!("CPSID f");
        basepri::write(0);
    }

    unsafe {
        init_heap(&HEAP);
    }

    prepare_clocks_and_power(
        &mut peripherals::ccm(),
        &mut peripherals::ccm_analog(),
        &mut peripherals::dcdc(),
    );

    peripherals::dcb().enable_trace();
    peripherals::dwt().enable_cycle_counter();

    modify_reg!(ral::iomuxc_gpr, peripherals::iomuxc_gpr(), GPR1, CM7_FORCE_HCLK_EN: CM7_FORCE_HCLK_EN_1);
    unsafe {
        let mut scb = peripherals::scb();
        scb.enable_icache();
        scb.enable_dcache(&mut peripherals::cpuid());
        // enable SEVONPEND (for systick), disable DEEPSLEEP
        scb.scr.write(0b10000);
    }

    let mut systick = peripherals::syst();
    systick.set_clock_source(SystClkSource::Core);
    systick.clear_current();
    systick.enable_interrupt();

    let iomuxc = into_pads(peripherals::iomuxc());
    let pins = from_pads(iomuxc);
    let mut erased_pins = pins.erase();

    let mut driver = ScreenDriver::new(&mut erased_pins);

    let mut current_program = PROGRAM_CONSTRUCTORS[0]();
    current_program.init(&mut driver);

    loop {
        current_program.render(&mut driver);
        driver.drive_post_render();
    }
}
