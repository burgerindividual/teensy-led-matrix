#![feature(variant_count)]
#![feature(abi_unadjusted)]
#![feature(link_llvm_intrinsics)]
#![feature(array_chunks)]
#![feature(slice_flatten)]
#![no_std]
#![no_main]

mod framebuffer;
mod intrinsics;
mod pins;

// this is used to add the default panic handler, not sure why it goes marked as unused
use core::hint::{black_box, spin_loop};
use core::sync::atomic::{fence, Ordering};

use teensy4_bsp::hal::iomuxc::into_pads;
use teensy4_bsp::pins::imxrt_iomuxc::gpio::Pin;
use teensy4_bsp::pins::t40::*;
use teensy4_bsp::ral::snvs::HPCR;
use teensy4_bsp::ral::{modify_reg, read_reg, write_reg};
use teensy4_bsp::{board, ral};
#[allow(unused_imports)]
use teensy4_panic as _;

use crate::framebuffer::*;
use crate::intrinsics::{pwm_pulse_batched, BATCH_SIZE};
use crate::pins::*;

/// Probably very bad, gets rid of the memory barriers in exchange for a single yield instruction.
pub const FAST_MODE: bool = true;
/// Effectively sets the FPS by masking which bits of the RTC 32khz clock should be tested.
pub const RTC_MASK: u32 = 0xFFFFFD00;

#[teensy4_bsp::rt::entry]
fn main() -> ! {
    let teensy_peripherals = board::instances();
    // let cortex_peripherals = Peripherals::take().unwrap();

    // activate GPIO6, GPIO7, and GPIO9 with our used pins
    write_reg!(
        ral::gpio,
        teensy_peripherals.IOMUXC_GPR,
        GPR26,
        GPIO6_PIN_MASK
    );
    write_reg!(
        ral::gpio,
        teensy_peripherals.IOMUXC_GPR,
        GPR29,
        GPIO9_PIN_MASK
    );

    let iomuxc = into_pads(teensy_peripherals.IOMUXC);
    let pins = from_pads(iomuxc);

    let mut erased_pins = pins.erase();

    // configure LED output pins
    for &pin in GPIO6_BATCHED_PIN_OFFSETS.flatten() {
        led_output_pin_setup(&mut erased_pins[pin]);
    }

    // configure clock pins
    clock_pin_setup(&mut erased_pins[2]);
    clock_pin_setup(&mut erased_pins[3]);

    // set directions for gpio pins
    modify_reg!(ral::gpio, teensy_peripherals.GPIO6, GDIR, |gdir| gdir
        | GPIO6_PIN_MASK);
    modify_reg!(ral::gpio, teensy_peripherals.GPIO9, GDIR, |gdir| gdir
        | GPIO9_PIN_MASK);

    let mut framebuffer = black_box(LedFramebuffer::default());

    // enable RTC and wait for it to get set
    modify_reg!(ral::gpio, teensy_peripherals.SNVS, HPCR, |hpcr| hpcr
        | HPCR::RTC_EN::RW::RTC_EN_1);
    while (read_reg!(ral::gpio, teensy_peripherals.SNVS, HPCR) & HPCR::RTC_EN::mask)
        != HPCR::RTC_EN::RW::RTC_EN_1
    {
        spin_loop();
    }

    let mut current_shift_bit = 0_u8;
    let mut last_rtc_val = 0;

    loop {
        unsafe {
            let target_values = framebuffer
                .bit_target_lines
                .get_unchecked_mut(current_shift_bit as usize);
            let current_values = framebuffer
                .bit_current_lines
                .get_unchecked_mut(current_shift_bit as usize);

            let mut gpio6_out_buffer = 0_u32;

            for ((current_value_batch, target_value_batch), pin_offset_batch) in current_values
                .array_chunks_mut::<BATCH_SIZE>()
                .zip(target_values.array_chunks::<BATCH_SIZE>())
                .zip(GPIO6_BATCHED_PIN_OFFSETS.iter())
            {
                pwm_pulse_batched(
                    current_value_batch,
                    target_value_batch,
                    pin_offset_batch,
                    &mut gpio6_out_buffer,
                );
            }

            write_reg!(ral::gpio, teensy_peripherals.GPIO6, DR, gpio6_out_buffer);

            let mut clock_pulse = 1 << P2::OFFSET;
            clock_pulse |= if current_shift_bit == 0 {
                1 << P3::OFFSET
            } else {
                0
            };

            // 110ns delay?
            if !FAST_MODE {
                fence(Ordering::Release);
            }

            write_reg!(ral::gpio, teensy_peripherals.GPIO9, DR, clock_pulse);

            // 125ns delay?
            if !FAST_MODE {
                fence(Ordering::Release);
            } else {
                spin_loop();
            }

            write_reg!(ral::gpio, teensy_peripherals.GPIO9, DR, 0);

            // 10ns delay?

            current_shift_bit += 1;
            if current_shift_bit == SHIFT_COUNT {
                current_shift_bit = 0;

                let current_rtc_val =
                    read_reg!(ral::gpio, teensy_peripherals.SNVS, HPTALR) & RTC_MASK;

                // Frame advance is done here to effectively cause a vertical sync, as we
                // will only be updating the FB after all scanlines are written.
                if last_rtc_val != current_rtc_val {
                    last_rtc_val = current_rtc_val;
                    frame_advance(&mut framebuffer);
                }
            }
        }
    }
}

#[inline(always)]
pub fn frame_advance(framebuffer: &mut LedFramebuffer) {
    unsafe {
        framebuffer.set_led_unchecked(0, 0, 255, 255, 255);
    }
}
