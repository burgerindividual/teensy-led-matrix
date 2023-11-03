use cortex_m::peripheral::DWT;
use teensy4_bsp::board::ARM_FREQUENCY;
use teensy4_bsp::pins::imxrt_iomuxc::gpio::Pin;
use teensy4_bsp::pins::imxrt_iomuxc::ErasedPad;
use teensy4_bsp::pins::tmm::P5;
use teensy4_bsp::ral::{self, read_reg};

use crate::peripherals;
use crate::pins::button_pin_setup;

pub struct Button {
    last_button_input_time: u32,
    last_set_value: bool,
    debounce_value: bool,
}

impl Button {
    pub const BUTTON_DEBOUNCE_DELAY: u32 = ARM_FREQUENCY / 50;

    pub fn new(pin_5: &mut ErasedPad) -> Self {
        button_pin_setup(pin_5, P5::OFFSET);

        Self {
            last_button_input_time: 0,
            last_set_value: false,
            debounce_value: false,
        }
    }

    pub fn check_if_pressed<F: FnOnce()>(&mut self, true_eval: F) {
        let current_cycles = DWT::cycle_count();
        if current_cycles.wrapping_sub(self.last_button_input_time) >= Self::BUTTON_DEBOUNCE_DELAY {
            let button_read_value =
                (read_reg!(ral::gpio, peripherals::gpio9(), PSR) & (1 << P5::OFFSET)) != 0;
            let button_pushed = button_read_value && self.debounce_value;

            if button_pushed && !self.last_set_value {
                true_eval();
            }

            self.last_set_value = button_pushed;
            self.debounce_value = button_read_value;
            self.last_button_input_time = current_cycles;
        }
    }
}
