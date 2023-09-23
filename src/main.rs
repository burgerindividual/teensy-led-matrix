#![feature(variant_count)]
#![feature(stdsimd)]
#![no_std]
#![no_main]

mod framebuffer;
mod pins;

use rtic::app;
// this is used to add the default panic handler, not sure why it goes marked as unused
#[allow(unused_imports)]
use teensy4_panic as _;

#[app(device = teensy4_bsp, peripherals = true)]
mod app {

    use core::hint::spin_loop;
    use core::sync::atomic::{fence, Ordering};

    use teensy4_bsp::hal::iomuxc::into_pads;
    use teensy4_bsp::pins::imxrt_iomuxc::gpio::Pin;
    use teensy4_bsp::pins::t40::*;
    use teensy4_bsp::ral::gpio::*;
    use teensy4_bsp::ral::snvs::{HPCR, HPSR, SNVS};
    use teensy4_bsp::ral::{modify_reg, read_reg, write_reg};
    use teensy4_bsp::{board, ral};

    use crate::framebuffer::*;
    use crate::pins::*;

    /// Probably very bad, gets rid of the memory barriers in exchange for a single yield instruction.
    pub const FAST_MODE: bool = true;

    #[shared]
    struct Shared {
        framebuffer: LedFramebuffer,
    }

    #[local]
    struct Local {
        snvs: SNVS,
        gpio6: GPIO6,
        gpio7: GPIO7,
        gpio9: GPIO9,
    }

    #[init]
    fn init(_: init::Context) -> (Shared, Local) {
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
            GPR27,
            GPIO7_PIN_MASK
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
        for pin in GPIO6_PINS.iter().chain(GPIO7_PINS.iter()) {
            led_output_pin_setup(&mut erased_pins[pin.pin_index]);
        }

        // configure clock pins
        clock_pin_setup(&mut erased_pins[2]);
        clock_pin_setup(&mut erased_pins[3]);

        // set directions for gpio pins
        modify_reg!(ral::gpio, teensy_peripherals.GPIO6, GDIR, |gdir| gdir
            | GPIO6_PIN_MASK);
        modify_reg!(ral::gpio, teensy_peripherals.GPIO7, GDIR, |gdir| gdir
            | GPIO7_PIN_MASK);
        modify_reg!(ral::gpio, teensy_peripherals.GPIO9, GDIR, |gdir| gdir
            | GPIO9_PIN_MASK);

        let framebuffer = LedFramebuffer::default();

        // enable RTC and wait for it to get set
        modify_reg!(ral::gpio, teensy_peripherals.SNVS, HPCR, |hpcr| hpcr
            | HPCR::RTC_EN::RW::RTC_EN_1);
        while (read_reg!(ral::gpio, teensy_peripherals.SNVS, HPCR) & HPCR::RTC_EN::mask)
            != HPCR::RTC_EN::RW::RTC_EN_1
        {
            spin_loop();
        }

        // sets the periodic interrupt frequency to 1/64ths of a second
        modify_reg!(ral::gpio, teensy_peripherals.SNVS, HPCR, |hpcr| hpcr
            | (HPCR::PI_FREQ::RW::PI_FREQ_9 << HPCR::PI_FREQ::offset));

        // enable RTC periodic interrupts and wait for it to get set
        modify_reg!(ral::gpio, teensy_peripherals.SNVS, HPCR, |hpcr| hpcr
            | (HPCR::PI_EN::RW::PI_EN_1 << HPCR::PI_EN::offset));
        while ((read_reg!(ral::gpio, teensy_peripherals.SNVS, HPCR) & HPCR::PI_EN::mask)
            >> HPCR::PI_EN::offset)
            != HPCR::PI_EN::RW::PI_EN_1
        {
            spin_loop();
        }

        (
            Shared { framebuffer },
            Local {
                snvs: teensy_peripherals.SNVS,
                gpio6: teensy_peripherals.GPIO6,
                gpio7: teensy_peripherals.GPIO7,
                gpio9: teensy_peripherals.GPIO9,
            },
        )
    }

    #[idle(local = [gpio6, gpio7, gpio9], shared = [framebuffer])]
    fn idle(mut context: idle::Context) -> ! {
        let mut current_shift_bit = 0_u8;

        loop {
            context.shared.framebuffer.lock(|framebuffer| unsafe {
                let target_values = framebuffer
                    .bit_target_lines
                    .get_unchecked_mut(current_shift_bit as usize);
                let current_values = framebuffer
                    .bit_current_lines
                    .get_unchecked_mut(current_shift_bit as usize);

                let mut gpio6_out_buffer = 0_u32;
                let mut gpio7_out_buffer = 0_u32;

                for pin in GPIO6_PINS {
                    let current_value = current_values.get_unchecked_mut(pin.led_x);
                    let target_value = *target_values.get_unchecked(pin.led_x);
                    let pulse = pwm_pulse(current_value, target_value);

                    gpio6_out_buffer |= pulse << pin.offset;
                }

                for pin in GPIO7_PINS {
                    let current_value = current_values.get_unchecked_mut(pin.led_x);
                    let target_value = *target_values.get_unchecked(pin.led_x);
                    let pulse = pwm_pulse(current_value, target_value);

                    gpio7_out_buffer |= pulse << pin.offset;
                }

                write_reg!(ral::gpio, context.local.gpio6, DR, gpio6_out_buffer);
                write_reg!(ral::gpio, context.local.gpio7, DR, gpio7_out_buffer);

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

                write_reg!(ral::gpio, context.local.gpio9, DR, clock_pulse);

                // 125ns delay?
                if !FAST_MODE {
                    fence(Ordering::Release);
                } else {
                    spin_loop();
                }

                write_reg!(ral::gpio, context.local.gpio9, DR, 0);

                // 10ns delay?

                current_shift_bit += 1;
                if current_shift_bit == SHIFT_COUNT {
                    current_shift_bit = 0;
                }
            });
        }
    }

    #[inline(always)]
    pub fn pwm_pulse(current_value: &mut f32, target_value: f32) -> u32 {
        *current_value += target_value;
        // this should only ever return 0 or 1, so it should be safe
        let truncated = unsafe { current_value.to_int_unchecked::<i32>() };
        *current_value -= truncated as f32;
        truncated as u32
    }

    #[task(binds = SNVS_HP_WRAPPER, priority = 1, local = [snvs], shared = [framebuffer])]
    fn frame_advance(mut context: frame_advance::Context) {
        modify_reg!(ral::gpio, context.local.snvs, HPSR, |hpsr| hpsr
            | (HPSR::PI::RW::PI_1 << HPSR::PI::offset));
        context.shared.framebuffer.lock(|framebuffer| unsafe {
            framebuffer.set_led_unchecked(0, 0, 1.0, 1.0, 1.0);
        });
    }
}
