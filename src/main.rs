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

pub const WIDTH: usize = 20;
pub const HEIGHT: usize = 5;
pub const COLOR_COUNT: usize = variant_count::<Colors>();
pub const SHIFT_COUNT: u8 = (HEIGHT * COLOR_COUNT) as u8;

pub const GPIO6_PIN_MASK: u32 = get_pin_mask(&GPIO6_PINS);
pub const GPIO7_PIN_MASK: u32 = get_pin_mask(&GPIO7_PINS);
pub const GPIO9_PIN_MASK: u32 = (1 << P2::OFFSET) | (1 << P3::OFFSET);

/// Probably very bad, gets rid of the memory barriers in exchange for a single yield instruction.
pub const FAST_MODE: bool = true;

pub const fn get_pin_mask(pins: &[OutputPin]) -> u32 {
    let mut mask = 0_u32;

    let mut i = 0;
    while i < pins.len() {
        let pin = &pins[i];
        mask |= 1 << pin.offset;
        i += 1;
    }

    mask
}

pub const GPIO6_PINS: [OutputPin; 12] = [
    OutputPin::new(0, 0, P0::OFFSET),
    OutputPin::new(1, 1, P1::OFFSET),
    OutputPin::new(14, 10, P14::OFFSET),
    OutputPin::new(15, 11, P15::OFFSET),
    OutputPin::new(16, 12, P16::OFFSET),
    OutputPin::new(17, 13, P17::OFFSET),
    OutputPin::new(18, 14, P18::OFFSET),
    OutputPin::new(19, 15, P19::OFFSET),
    OutputPin::new(20, 16, P20::OFFSET),
    OutputPin::new(21, 17, P21::OFFSET),
    OutputPin::new(22, 18, P22::OFFSET),
    OutputPin::new(23, 19, P23::OFFSET),
];

pub const GPIO7_PINS: [OutputPin; 8] = [
    OutputPin::new(6, 2, P6::OFFSET),
    OutputPin::new(7, 3, P7::OFFSET),
    OutputPin::new(8, 4, P8::OFFSET),
    OutputPin::new(9, 5, P9::OFFSET),
    OutputPin::new(10, 6, P10::OFFSET),
    OutputPin::new(11, 7, P11::OFFSET),
    OutputPin::new(12, 8, P12::OFFSET),
    OutputPin::new(13, 9, P13::OFFSET),
];

#[derive(Copy, Clone)]
pub struct OutputPin {
    pin_index: usize,
    led_x: usize,
    offset: u32,
}

impl OutputPin {
    pub const fn new(pin_index: usize, led_x: usize, offset: u32) -> Self {
        Self {
            pin_index,
            led_x,
            offset,
        }
    }
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
    pub const RED_MULTIPLIER: f32 = 0.3;
    pub const GREEN_MULTIPLIER: f32 = 0.3;
    pub const BLUE_MULTIPLIER: f32 = 0.3;

    /// # Safety this method is safe as long as led_x and led_y are within the bounds of the led grid
    pub unsafe fn set_led_unchecked(&mut self, led_x: usize, led_y: usize, r: f32, g: f32, b: f32) {
        // TODO: do color adjustments in here

        let led_start_column = led_y * COLOR_COUNT;
        *(self
            .bit_target_lines
            .get_unchecked_mut(led_start_column + (Colors::Red as usize))
            .get_unchecked_mut(led_x)) = r * Self::RED_MULTIPLIER;
        *(self
            .bit_target_lines
            .get_unchecked_mut(led_start_column + (Colors::Green as usize))
            .get_unchecked_mut(led_x)) = g * Self::GREEN_MULTIPLIER;
        *(self
            .bit_target_lines
            .get_unchecked_mut(led_start_column + (Colors::Blue as usize))
            .get_unchecked_mut(led_x)) = b * Self::BLUE_MULTIPLIER;

        // we have to reset the current lines, because if we don't, the pwm function can break
        *(self
            .bit_current_lines
            .get_unchecked_mut(led_start_column + (Colors::Red as usize))
            .get_unchecked_mut(led_x)) = 0.0;
        *(self
            .bit_current_lines
            .get_unchecked_mut(led_start_column + (Colors::Green as usize))
            .get_unchecked_mut(led_x)) = 0.0;
        *(self
            .bit_current_lines
            .get_unchecked_mut(led_start_column + (Colors::Blue as usize))
            .get_unchecked_mut(led_x)) = 0.0;
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

    let mut erased_pins = pins.erase();
    for pin in GPIO6_PINS.iter().chain(GPIO7_PINS.iter()) {
        pin_setup(&mut erased_pins[pin.pin_index]);
    }

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
        for x in 0..WIDTH {
            let multiplier = (x + 1) as f32;
            framebuffer.set_led_unchecked(
                x,
                0,
                0.05 * multiplier,
                0.05 * multiplier,
                0.05 * multiplier,
            );
            framebuffer.set_led_unchecked(
                x,
                1,
                0.05 * multiplier,
                0.05 * multiplier,
                0.05 * multiplier,
            );
            framebuffer.set_led_unchecked(
                x,
                2,
                0.05 * multiplier,
                0.05 * multiplier,
                0.05 * multiplier,
            );
            framebuffer.set_led_unchecked(
                x,
                3,
                0.05 * multiplier,
                0.05 * multiplier,
                0.05 * multiplier,
            );
            framebuffer.set_led_unchecked(
                x,
                4,
                0.05 * multiplier,
                0.05 * multiplier,
                0.05 * multiplier,
            );
        }
        framebuffer.set_led_unchecked(0, 0, 0.01, 0.01, 0.01);
        framebuffer.set_led_unchecked(0, 1, 0.01, 0.01, 0.01);
        framebuffer.set_led_unchecked(0, 2, 0.01, 0.01, 0.01);
        framebuffer.set_led_unchecked(0, 3, 0.01, 0.01, 0.01);
        framebuffer.set_led_unchecked(0, 4, 0.01, 0.01, 0.01);
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
