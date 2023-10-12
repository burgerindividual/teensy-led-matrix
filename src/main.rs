#![feature(abi_unadjusted)]
#![feature(link_llvm_intrinsics)]
#![feature(array_chunks)]
#![feature(slice_flatten)]
#![feature(exclusive_range_pattern)]
#![feature(maybe_uninit_slice)]
#![no_std]
#![no_main]

extern crate alloc;

mod color;
mod framebuffer;
mod intrinsics;
mod pins;
mod take_mut;

mod collections;
mod driver;
mod peripherals;
mod program;
mod unwrap;

use alloc::boxed::Box;

use cortex_m::interrupt;
use embedded_alloc::Heap;
use teensy4_bsp::board::prepare_clocks_and_power;
use teensy4_bsp::hal::iomuxc::into_pads;
use teensy4_bsp::pins::t40::*;
#[allow(unused_imports)]
use teensy4_panic as _;

use crate::driver::ScreenDriver;
use crate::intrinsics::init_heap;
use crate::program::*;

#[global_allocator]
static mut HEAP: Heap = Heap::empty();

pub const PROGRAM_CONSTRUCTORS: [fn() -> Box<dyn Program>; 1] = [HueCycle::new];

#[teensy4_bsp::rt::entry]
unsafe fn main() -> ! {
    // completely disable all interrupts, allows for unsafe peripheral access to be safe
    interrupt::disable();

    init_heap(&HEAP);

    prepare_clocks_and_power(
        &mut peripherals::ccm(),
        &mut peripherals::ccm_analog(),
        &mut peripherals::dcdc(),
    );

    peripherals::dcb().enable_trace();
    peripherals::dwt().enable_cycle_counter();

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
