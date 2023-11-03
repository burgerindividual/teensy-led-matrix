use core::ptr;

use teensy4_bsp::hal::gpio::{Port, Trigger};
use teensy4_bsp::pins::imxrt_iomuxc::gpio::Pin;
use teensy4_bsp::pins::imxrt_iomuxc::*;
use teensy4_bsp::pins::t40::*;
use teensy4_bsp::ral;
use teensy4_bsp::ral::modify_reg;

use crate::intrinsics::BATCH_SIZE;
use crate::peripherals;

pub const LED_OUTPUT_PIN_INDICES: [u32; 12] = [1, 0, 17, 16, 19, 18, 14, 15, 22, 23, 20, 21];
pub const GPIO6_BATCHED_PIN_OFFSETS: [[u32; BATCH_SIZE]; 3] = [
    [P1::OFFSET, P0::OFFSET, P17::OFFSET, P16::OFFSET],
    [P19::OFFSET, P18::OFFSET, P14::OFFSET, P15::OFFSET],
    [P22::OFFSET, P23::OFFSET, P20::OFFSET, P21::OFFSET],
];

pub fn led_output_pin_setup<P: Iomuxc>(pin: &mut P, bit_offset: u32) {
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
    let pin_bit = 1 << bit_offset;
    // activate high-speed GPIO on pin
    modify_reg!(
        ral::iomuxc_gpr,
        peripherals::iomuxc_gpr(),
        GPR26,
        |gpr26| gpr26 | pin_bit
    );
    // set direction for pin
    modify_reg!(ral::gpio, peripherals::gpio6(), GDIR, |gdir| gdir | pin_bit);
}

pub fn clock_pin_setup<P: Iomuxc>(pin: &mut P, bit_offset: u32) {
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
    let pin_bit = 1 << bit_offset;
    // activate high-speed GPIO on pin
    modify_reg!(
        ral::iomuxc_gpr,
        peripherals::iomuxc_gpr(),
        GPR29,
        |gpr29| gpr29 | pin_bit
    );
    // set output for pin
    modify_reg!(ral::gpio, peripherals::gpio9(), GDIR, |gdir| gdir | pin_bit);
}

pub fn button_pin_setup<P: Iomuxc>(pin: &mut P, bit_offset: u32) {
    // configure to be GPIO, which is done by setting ALT to 5
    alternate(pin, 5);
    configure(
        pin,
        Config::zero()
            .set_speed(Speed::Low)
            .set_drive_strength(DriveStrength::Disabled)
            .set_pull_keeper(Some(PullKeeper::Pulldown100k))
            .set_hysteresis(Hysteresis::Disabled)
            .set_slew_rate(SlewRate::Slow)
            .set_open_drain(OpenDrain::Disabled),
    );
    let pin_bit = 1 << bit_offset;
    // activate high-speed GPIO on pin
    modify_reg!(
        ral::iomuxc_gpr,
        peripherals::iomuxc_gpr(),
        GPR29,
        |gpr29| gpr29 | pin_bit
    );
    // set input for pin
    modify_reg!(ral::gpio, peripherals::gpio9(), GDIR, |gdir| gdir
        & !pin_bit);
}
