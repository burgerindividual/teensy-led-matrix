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

use embedded_alloc::Heap;
use teensy4_bsp::board::prepare_clocks_and_power;
use teensy4_bsp::hal::iomuxc::into_pads;
use teensy4_bsp::pins::t40::*;
#[allow(unused_imports)]
use teensy4_panic as _;

use crate::driver::ScreenDriver;
use crate::intrinsics::init_heap;
use crate::peripherals::Peripherals;
use crate::program::*;
use crate::unwrap::unwrap;

#[global_allocator]
static mut HEAP: Heap = Heap::empty();

pub const PROGRAM_CONSTRUCTORS: [fn(&mut Peripherals) -> Box<dyn Program>; 1] = [HueCycle::new];

#[teensy4_bsp::rt::entry]
unsafe fn main() -> ! {
    init_heap(&mut HEAP);

    let mut peripherals = Peripherals::instance();

    prepare_clocks_and_power(
        unwrap(peripherals.CCM.as_mut()),
        unwrap(peripherals.CCM_ANALOG.as_mut()),
        unwrap(peripherals.DCDC.as_mut()),
    );

    unwrap(peripherals.DCB.as_mut()).enable_trace();
    unwrap(peripherals.DWT.as_mut()).enable_cycle_counter();

    let iomuxc = into_pads(unwrap(peripherals.IOMUXC.take()));
    let pins = from_pads(iomuxc);
    let mut erased_pins = pins.erase();

    let mut driver = ScreenDriver::new(
        unwrap(peripherals.GPIO6.take()),
        unwrap(peripherals.GPIO9.take()),
        unwrap(peripherals.SNVS.take()),
        unwrap(peripherals.IOMUXC_GPR.as_mut()),
        unwrap(peripherals.CCM.as_mut()),
        &mut erased_pins,
    );

    let mut current_program = PROGRAM_CONSTRUCTORS[0](&mut peripherals);
    current_program.init(&mut driver);

    loop {
        current_program.render(&mut driver);
        driver.drive_post_render();
    }
}
