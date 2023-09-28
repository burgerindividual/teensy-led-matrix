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

use core::hint::spin_loop;

use teensy4_bsp::hal::iomuxc::into_pads;
use teensy4_bsp::pins::imxrt_iomuxc::gpio::Pin;
use teensy4_bsp::pins::t40::*;
use teensy4_bsp::ral::snvs::HPCR;
use teensy4_bsp::ral::{modify_reg, read_reg, write_reg};
use teensy4_bsp::{board, ral};
#[allow(unused_imports)]
use teensy4_panic as _;

use crate::framebuffer::*;
use crate::intrinsics::{pwm_pulse_batched, yield_ns, BATCH_SIZE};
use crate::pins::*;

/// Gets rid of the delays used to fit in spec with the SN74HC595 datasheet.
pub const FAST_MODE: bool = false;
/// Effectively sets the FPS by masking which bits of the RTC 32khz clock should be tested.
pub const RTC_MASK: u32 = (-1_i32 << 6) as u32;

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

    let mut framebuffer = LedFramebuffer::default();
    unsafe {
        framebuffer.set_led_unchecked(0, 3, 0, 255, 0);
        framebuffer.set_led_unchecked(0, 4, 127, 255, 0);
        framebuffer.set_led_unchecked(0, 5, 255, 255, 0);
        framebuffer.set_led_unchecked(0, 6, 255, 127, 0);
        framebuffer.set_led_unchecked(0, 7, 255, 0, 0);
    }

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

        let mut clock_pulse = 1 << P2::OFFSET;
        clock_pulse |= if current_shift_bit == 0 {
            1 << P3::OFFSET
        } else {
            0
        };

        if !FAST_MODE {
            yield_ns::<110>();
        }

        write_reg!(ral::gpio, teensy_peripherals.GPIO9, DR_SET, clock_pulse);

        if !FAST_MODE {
            yield_ns::<125>();
        }

        write_reg!(ral::gpio, teensy_peripherals.GPIO9, DR_CLEAR, clock_pulse);

        if !FAST_MODE {
            yield_ns::<10>();
        }

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
