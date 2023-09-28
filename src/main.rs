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
mod program;

use core::hint::spin_loop;

use cortex_m::peripheral::DWT;
use cortex_m::Peripherals;
use teensy4_bsp::hal::iomuxc::into_pads;
use teensy4_bsp::pins::imxrt_iomuxc::gpio::Pin;
use teensy4_bsp::pins::t40::*;
use teensy4_bsp::ral::snvs::HPCR;
use teensy4_bsp::ral::{modify_reg, read_reg, write_reg};
use teensy4_bsp::{board, ral};
#[allow(unused_imports)]
use teensy4_panic as _;

use crate::framebuffer::*;
use crate::intrinsics::{pwm_pulse_batched, wait_cycles, BATCH_SIZE};
use crate::pins::*;

pub const DELAY_1_CYCLES: u32 = (22_u32 * 6).div_ceil(10);
pub const DELAY_2_CYCLES: u32 = (25_u32 * 6).div_ceil(10);
/// Effectively sets the FPS by masking which bits of the RTC 32khz clock should be tested.
pub const RTC_MASK: u32 = (-1_i32 << 6) as u32;

#[teensy4_bsp::rt::entry]
fn main() -> ! {
    let teensy_peripherals = board::instances();
    let mut cortex_peripherals = Peripherals::take().unwrap();
    cortex_peripherals.DWT.enable_cycle_counter();

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
    for idx in LED_OUTPUT_PIN_INDICES {
        led_output_pin_setup(&mut erased_pins[idx]);
    }

    // configure clock pins
    clock_pin_setup(&mut erased_pins[2]);
    clock_pin_setup(&mut erased_pins[3]);

    // set directions for gpio pins
    modify_reg!(ral::gpio, teensy_peripherals.GPIO6, GDIR, |gdir| gdir
        | GPIO6_PIN_MASK);
    modify_reg!(ral::gpio, teensy_peripherals.GPIO9, GDIR, |gdir| gdir
        | GPIO9_PIN_MASK);

    // enable RTC and wait for it to get set
    modify_reg!(ral::gpio, teensy_peripherals.SNVS, HPCR, |hpcr| hpcr
        | HPCR::RTC_EN::RW::RTC_EN_1);
    while (read_reg!(ral::gpio, teensy_peripherals.SNVS, HPCR) & HPCR::RTC_EN::mask)
        != HPCR::RTC_EN::RW::RTC_EN_1
    {
        spin_loop();
    }

    let mut framebuffer = Framebuffer::default();

    let mut current_shift_bit = 0_u8;
    let mut last_rtc_val = 0;

    loop {
        let target_values = unsafe {
            framebuffer
                .bit_target_lines
                .get_unchecked_mut(current_shift_bit as usize)
        };
        let current_values = unsafe {
            framebuffer
                .bit_current_lines
                .get_unchecked_mut(current_shift_bit as usize)
        };

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

        let start_cycle_count = DWT::cycle_count();
        // TODO: process frame here
        let mut clock_pulse = 1 << P2::OFFSET;
        clock_pulse |= if current_shift_bit == 0 {
            1 << P3::OFFSET
        } else {
            0
        };

        wait_cycles::<DELAY_1_CYCLES>(start_cycle_count);

        write_reg!(ral::gpio, teensy_peripherals.GPIO9, DR_SET, clock_pulse);

        let start_cycle_count = DWT::cycle_count();
        // TODO: process frame here
        wait_cycles::<DELAY_2_CYCLES>(start_cycle_count);

        write_reg!(ral::gpio, teensy_peripherals.GPIO9, DR_CLEAR, clock_pulse);

        // between the clock pulse and the serial output changing, 3 cycles of delay is expected.
        // in any scenario, this is already satisfied by the code setting up the next serial output,
        // so it should be fine to exclude an excess yield.

        current_shift_bit += 1;
        if current_shift_bit == SHIFT_COUNT {
            current_shift_bit = 0;

            let current_rtc_val = read_reg!(ral::gpio, teensy_peripherals.SNVS, HPRTCLR) & RTC_MASK;

            // Frame advance is done here to effectively cause a vertical sync, as we
            // will only be updating the FB after all scanlines are written.
            if last_rtc_val != current_rtc_val {
                last_rtc_val = current_rtc_val;
                frame_advance(&mut framebuffer);
            }
        }
    }
}

#[inline(always)]
pub fn frame_advance(framebuffer: &mut LedFramebuffer) {
    unsafe {
        for x in 0..WIDTH {
            for y in 0..HEIGHT {
                let (mut r, mut g, mut b) = framebuffer.get_led_unchecked(x, y);

                g = g.saturating_add((r == 0xFF && b == 0x00) as u8);
                g = g.saturating_sub((b == 0xFF && r == 0x00) as u8);

                b = b.saturating_add((g == 0xFF && r == 0x00) as u8);
                b = b.saturating_sub((r == 0xFF && g == 0x00) as u8);

                r = r.saturating_add((b == 0xFF && g == 0x00) as u8);
                r = r.saturating_sub((g == 0xFF && b == 0x00) as u8);

                framebuffer.set_led_unchecked(x, y, r, g, b);
            }
        }
    }
}
