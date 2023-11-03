#![feature(abi_unadjusted)]
#![feature(link_llvm_intrinsics)]
#![feature(array_chunks)]
#![feature(slice_flatten)]
#![feature(exclusive_range_pattern)]
#![feature(maybe_uninit_slice)]
#![no_std]
#![no_main]

extern crate alloc;

mod button;
mod collections;
mod color;
mod framebuffer;
mod intrinsics;
mod led_driver;
mod peripherals;
mod pins;
mod program;

use core::arch::asm;

use cortex_m::peripheral::syst::SystClkSource;

use cortex_m::register::basepri;
use embedded_alloc::Heap;
use teensy4_bsp::board::{prepare_clocks_and_power};
use teensy4_bsp::hal::iomuxc::into_pads;


use teensy4_bsp::pins::t40::*;
use teensy4_bsp::ral::{self, modify_reg};
#[allow(unused_imports)]
use teensy4_panic as _;

use crate::button::Button;
use crate::intrinsics::{init_heap};
use crate::led_driver::ScreenDriver;

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

    let mut button = Button::new(&mut erased_pins[5]);
    let mut led_driver = ScreenDriver::new(&mut erased_pins);

    let mut program_index = 0;
    let mut current_program = PROGRAM_CONSTRUCTORS[program_index](&mut led_driver);

    loop {
        current_program.render(&mut led_driver);
        led_driver.drive_post_render();

        button.check_if_pressed(|| {
            program_index += 1;
            if program_index >= PROGRAM_CONSTRUCTORS.len() {
                program_index = 0;
            }

            current_program = PROGRAM_CONSTRUCTORS[program_index](&mut led_driver);
        });
    }
}
