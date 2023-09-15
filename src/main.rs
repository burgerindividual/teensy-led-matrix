#![feature(core_intrinsics)]
#![feature(variant_count)]
#![feature(stdsimd)]
#![no_std]
#![no_main]

use core::hint::spin_loop;
use core::intrinsics::*;
use core::sync::atomic::{fence, Ordering};

use teensy4_bsp::hal::iomuxc::{
    clear_sion, configure, into_pads, DriveStrength, Hysteresis, SlewRate, Speed,
};
use teensy4_bsp::pins::imxrt_iomuxc::gpio::Pin;
use teensy4_bsp::pins::imxrt_iomuxc::{alternate, Config, Iomuxc, OpenDrain};
use teensy4_bsp::pins::t40::*;
use teensy4_bsp::ral::{modify_reg, write_reg};
use teensy4_bsp::{board, ral};
// this is used to add the default panic handler, not sure why it goes marked as unused
#[allow(unused_imports)]
use teensy4_panic as _;

pub const WIDTH: usize = 1;
pub const HEIGHT: usize = 5;
pub const COLOR_COUNT: usize = variant_count::<Colors>();
pub const SHIFT_COUNT: u8 = (HEIGHT * COLOR_COUNT) as u8;

pub const GPIO6_PIN_MASK: u32 = get_pin_mask(OutputGpio::Gpio6);
pub const GPIO7_PIN_MASK: u32 = get_pin_mask(OutputGpio::Gpio7);
pub const GPIO9_PIN_MASK: u32 = (1 << P2::OFFSET) | (1 << P3::OFFSET);

/// Probably very bad, gets rid of the memory barriers in exchange for a single yield instruction.
pub const FAST_MODE: bool = true;

pub const fn get_pin_mask(gpio: OutputGpio) -> u32 {
    let mut mask = 0_u32;

    let mut i = 0;
    while i < LED_OUTPUT_PINS.len() {
        let pin = &LED_OUTPUT_PINS[i];
        if pin.gpio as u8 == gpio as u8 {
            mask |= 1 << pin.offset;
        }
        i += 1;
    }

    mask
}

pub const LED_OUTPUT_PINS: [OutputPin; WIDTH] = [OutputPin {
    offset: P0::OFFSET,
    gpio: OutputGpio::Gpio6,
}];

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum OutputGpio {
    Gpio6 = 0,
    Gpio7 = 1,
}

pub struct OutputPin {
    offset: u32,
    gpio: OutputGpio,
}

#[repr(u8)]
pub enum Colors {
    Red = 0,
    Green = 1,
    Blue = 2,
}

#[derive(Default)]
pub struct LedFramebuffer {
    bit_target_lines: [[f32; WIDTH]; HEIGHT * COLOR_COUNT],
    bit_current_lines: [[f32; WIDTH]; HEIGHT * COLOR_COUNT],
}

impl LedFramebuffer {
    pub unsafe fn set_led_unchecked(&mut self, led_x: usize, led_y: usize, r: f32, g: f32, b: f32) {
        // TODO: do color adjustments in here

        let led_start_column = led_y * COLOR_COUNT;
        *(self
            .bit_target_lines
            .get_unchecked_mut(led_start_column + (Colors::Blue as usize))
            .get_unchecked_mut(led_x)) = b;
        *(self
            .bit_target_lines
            .get_unchecked_mut(led_start_column + (Colors::Green as usize))
            .get_unchecked_mut(led_x)) = g;
        *(self
            .bit_target_lines
            .get_unchecked_mut(led_start_column + (Colors::Red as usize))
            .get_unchecked_mut(led_x)) = r;
    }
}

#[teensy4_bsp::rt::entry]
fn main() -> ! {
    // These are peripheral instances. Let the board configure these for us.
    // This function can only be called once!
    let instances = board::instances();
    // let peripherals = Peripherals::take().unwrap();

    // activate GPIO6, GPIO7, and GPIO9 with our used pins
    write_reg!(ral::gpio, instances.IOMUXC_GPR, GPR26, GPIO6_PIN_MASK);
    write_reg!(ral::gpio, instances.IOMUXC_GPR, GPR27, GPIO7_PIN_MASK);
    write_reg!(ral::gpio, instances.IOMUXC_GPR, GPR29, GPIO9_PIN_MASK);

    let iomuxc = into_pads(instances.IOMUXC);
    let pins = from_pads(iomuxc);

    let [mut p0, _unused, mut p2, mut p3, ..] = pins.erase();
    pin_setup(&mut p0);
    pin_setup(&mut p2);
    pin_setup(&mut p3);

    // set directions for gpio pins
    modify_reg!(ral::gpio, instances.GPIO6, GDIR, |gdir| gdir
        | GPIO6_PIN_MASK);
    modify_reg!(ral::gpio, instances.GPIO7, GDIR, |gdir| gdir
        | GPIO7_PIN_MASK);
    modify_reg!(ral::gpio, instances.GPIO9, GDIR, |gdir| gdir
        | GPIO9_PIN_MASK);

    // let mut counter = 0_u32;
    let mut current_bit = 0_u8;
    let mut framebuffer = LedFramebuffer::default();
    unsafe {
        // framebuffer.set_led_unchecked(0, 0, 1.0, 0.0, 0.0);
        // framebuffer.set_led_unchecked(0, 1, 1.0, 1.0, 0.0);
        // framebuffer.set_led_unchecked(0, 2, 0.0, 1.0, 0.0);
        // framebuffer.set_led_unchecked(0, 3, 0.0, 0.0, 1.0);
        // framebuffer.set_led_unchecked(0, 4, 1.0, 0.0, 1.0);
        framebuffer.set_led_unchecked(0, 0, 0.00001, 0.0, 0.0);
        framebuffer.set_led_unchecked(0, 1, 0.00001, 0.00001, 0.0);
        framebuffer.set_led_unchecked(0, 2, 0.0, 0.00001, 0.0);
        framebuffer.set_led_unchecked(0, 3, 0.0, 0.0, 0.00001);
        framebuffer.set_led_unchecked(0, 4, 0.00001, 0.0, 0.00001);
    }

    loop {
        unsafe {
            let target_values = framebuffer
                .bit_target_lines
                .get_unchecked_mut(current_bit as usize);
            let current_values = framebuffer
                .bit_current_lines
                .get_unchecked_mut(current_bit as usize);

            let mut gpio6_out_buffer = 0_u32;
            let mut gpio7_out_buffer = 0_u32;

            for (i, pin) in LED_OUTPUT_PINS.iter().enumerate() {
                let output_buffer = match pin.gpio {
                    OutputGpio::Gpio6 => &mut gpio6_out_buffer,
                    OutputGpio::Gpio7 => &mut gpio7_out_buffer,
                };

                let current_value = current_values.get_unchecked_mut(i);
                let target_value = *target_values.get_unchecked(i);
                let pulse = pwm_pulse(current_value, target_value);

                *output_buffer |= pulse << pin.offset;
            }

            write_reg!(ral::gpio, instances.GPIO6, DR, gpio6_out_buffer);
            write_reg!(ral::gpio, instances.GPIO7, DR, gpio7_out_buffer);

            let mut clock_pulse = 1 << P2::OFFSET;
            clock_pulse |= if current_bit == 0 { 1 << P3::OFFSET } else { 0 };

            // 110ns delay?
            if !FAST_MODE {
                fence(Ordering::Release);
            }

            write_reg!(ral::gpio, instances.GPIO9, DR, clock_pulse);

            // 125ns delay?
            if !FAST_MODE {
                fence(Ordering::Release);
            } else {
                spin_loop();
            }

            write_reg!(ral::gpio, instances.GPIO9, DR, 0);

            // 10ns delay?

            current_bit += 1;
            if current_bit == SHIFT_COUNT {
                current_bit = 0;
            }
        }
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

#[inline(always)]
pub fn pin_setup<P: Iomuxc>(pin: &mut P) {
    // configure to be GPIO, which is done by setting ALT to 5
    alternate(pin, 5);
    // we don't want to read it, so disable input
    clear_sion(pin);
    configure(
        pin,
        Config::zero()
            .set_speed(Speed::Max)
            .set_drive_strength(DriveStrength::R0)
            .set_pull_keeper(None)
            .set_hysteresis(Hysteresis::Disabled)
            .set_slew_rate(SlewRate::Fast)
            .set_open_drain(OpenDrain::Disabled),
    );
}
