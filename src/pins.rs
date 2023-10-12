use teensy4_bsp::pins::imxrt_iomuxc::gpio::Pin;
use teensy4_bsp::pins::imxrt_iomuxc::*;
use teensy4_bsp::pins::t40::*;

use crate::intrinsics::BATCH_SIZE;

pub const GPIO6_PIN_MASK: u32 = create_pin_mask(&GPIO6_BATCHED_PIN_OFFSETS);
pub const GPIO9_PIN_MASK: u32 = (1 << P2::OFFSET) | (1 << P3::OFFSET);

pub const LED_OUTPUT_PIN_INDICES: [usize; 12] = [1, 0, 17, 16, 19, 18, 14, 15, 22, 23, 20, 21];
pub const GPIO6_BATCHED_PIN_OFFSETS: [[usize; BATCH_SIZE]; 3] = [
    [
        P1::OFFSET as usize,
        P0::OFFSET as usize,
        P17::OFFSET as usize,
        P16::OFFSET as usize,
    ],
    [
        P19::OFFSET as usize,
        P18::OFFSET as usize,
        P14::OFFSET as usize,
        P15::OFFSET as usize,
    ],
    [
        P22::OFFSET as usize,
        P23::OFFSET as usize,
        P20::OFFSET as usize,
        P21::OFFSET as usize,
    ],
];

pub const fn create_pin_mask(pins: &[[usize; BATCH_SIZE]]) -> u32 {
    let flattened_offsets = pins.flatten();
    let mut mask = 0_u32;

    let mut i = 0;
    while i < flattened_offsets.len() {
        mask |= 1 << flattened_offsets[i];
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
    // clear_sion(pin);
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

pub fn button_pin_setup<P: Iomuxc>(pin: &mut P) {
    // configure to be GPIO, which is done by setting ALT to 5
    alternate(pin, 5);
    // we *do* want to read this pin, so force enable input
    set_sion(pin);
    configure(
        pin,
        Config::zero()
            .set_speed(Speed::Low)
            .set_drive_strength(DriveStrength::Disabled)
            .set_pull_keeper(Some(PullKeeper::Keeper))
            .set_hysteresis(Hysteresis::Disabled)
            .set_slew_rate(SlewRate::Slow)
            .set_open_drain(OpenDrain::Disabled),
    );
}
