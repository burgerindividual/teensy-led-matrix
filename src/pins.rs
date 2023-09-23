use teensy4_bsp::pins::imxrt_iomuxc::gpio::Pin;
use teensy4_bsp::pins::imxrt_iomuxc::*;
use teensy4_bsp::pins::t40::*;

use crate::framebuffer::{COLOR_COUNT, HEIGHT};

pub const SHIFT_COUNT: u8 = (HEIGHT * COLOR_COUNT) as u8;

pub const GPIO6_PIN_MASK: u32 = get_pin_mask(&GPIO6_PINS);
pub const GPIO7_PIN_MASK: u32 = get_pin_mask(&GPIO7_PINS);
pub const GPIO9_PIN_MASK: u32 = (1 << P2::OFFSET) | (1 << P3::OFFSET);

pub const GPIO6_PINS: [LedOutputPin; 1] = [
    LedOutputPin::new(0, 0, P0::OFFSET),
    // LedOutputPin::new(1, 1, P1::OFFSET),
    // LedOutputPin::new(14, 10, P14::OFFSET),
    // LedOutputPin::new(15, 11, P15::OFFSET),
    // LedOutputPin::new(16, 12, P16::OFFSET),
    // LedOutputPin::new(17, 13, P17::OFFSET),
    // LedOutputPin::new(18, 14, P18::OFFSET),
    // LedOutputPin::new(19, 15, P19::OFFSET),
    // LedOutputPin::new(20, 16, P20::OFFSET),
    // LedOutputPin::new(21, 17, P21::OFFSET),
    // LedOutputPin::new(22, 18, P22::OFFSET),
    // LedOutputPin::new(23, 19, P23::OFFSET),
];

pub const GPIO7_PINS: [LedOutputPin; 0] = [
    // LedOutputPin::new(6, 2, P6::OFFSET),
    // LedOutputPin::new(7, 3, P7::OFFSET),
    // LedOutputPin::new(8, 4, P8::OFFSET),
    // LedOutputPin::new(9, 5, P9::OFFSET),
    // LedOutputPin::new(10, 6, P10::OFFSET),
    // LedOutputPin::new(11, 7, P11::OFFSET),
    // LedOutputPin::new(12, 8, P12::OFFSET),
    // LedOutputPin::new(13, 9, P13::OFFSET),
];

#[derive(Copy, Clone)]
pub struct LedOutputPin {
    pub pin_index: usize,
    pub led_x: usize,
    pub offset: u32,
}

impl LedOutputPin {
    pub const fn new(pin_index: usize, led_x: usize, offset: u32) -> Self {
        Self {
            pin_index,
            led_x,
            offset,
        }
    }
}

pub const fn get_pin_mask(pins: &[LedOutputPin]) -> u32 {
    let mut mask = 0_u32;

    let mut i = 0;
    while i < pins.len() {
        let pin = &pins[i];
        mask |= 1 << pin.offset;
        i += 1;
    }

    mask
}

pub fn led_output_pin_setup<P: Iomuxc>(pin: &mut P) {
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

pub fn clock_pin_setup<P: Iomuxc>(pin: &mut P) {
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
