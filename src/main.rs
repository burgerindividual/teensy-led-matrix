#![feature(abi_unadjusted)]
#![feature(link_llvm_intrinsics)]
#![feature(array_chunks)]
#![feature(slice_flatten)]
#![feature(exclusive_range_pattern)]
#![feature(maybe_uninit_slice)]
#![no_std]
#![no_main]

mod color;
mod framebuffer;
mod intrinsics;
mod pins;
mod take_mut;

mod collections;
mod driver;
mod program;

use cortex_m::Peripherals;
use teensy4_bsp::board;
use teensy4_bsp::hal::iomuxc::into_pads;
use teensy4_bsp::pins::t40::*;
#[allow(unused_imports)]
use teensy4_panic as _;

use crate::driver::ScreenDriver;
use crate::program::*;

pub type CurrentProgram = HueCycle;

#[teensy4_bsp::rt::entry]
fn main() -> ! {
    let mut teensy_peripherals = board::instances();
    let mut cortex_peripherals = Peripherals::take().unwrap();

    cortex_peripherals.DCB.enable_trace();
    cortex_peripherals.DWT.enable_cycle_counter();

    let iomuxc = into_pads(teensy_peripherals.IOMUXC);
    let pins = from_pads(iomuxc);
    let mut erased_pins = pins.erase();

    let mut driver = ScreenDriver::new(
        teensy_peripherals.GPIO6,
        teensy_peripherals.GPIO9,
        teensy_peripherals.SNVS,
        &mut teensy_peripherals.IOMUXC_GPR,
        &mut erased_pins,
    );

    // let mut program = CurrentProgram::new(&mut teensy_peripherals.TRNG);
    let mut program = CurrentProgram {};
    program.init(&mut driver);

    loop {
        program.render(&mut driver);
        driver.drive_post_render();
    }
}
