use core::mem::variant_count;

pub const WIDTH: usize = 1;
pub const HEIGHT: usize = 5;
pub const COLOR_COUNT: usize = variant_count::<Colors>();

#[repr(u8)]
pub enum Colors {
    Red = 0,
    Green = 1,
    Blue = 2,
}

#[derive(Default)]
pub struct LedFramebuffer {
    pub(crate) bit_target_lines: [[f32; WIDTH]; HEIGHT * COLOR_COUNT],
    pub(crate) bit_current_lines: [[f32; WIDTH]; HEIGHT * COLOR_COUNT],
}

impl LedFramebuffer {
    pub const RED_MULTIPLIER: f32 = 430.0 / 470.0;
    pub const GREEN_MULTIPLIER: f32 = 1.0;
    pub const BLUE_MULTIPLIER: f32 = 1.0;

    #[inline(always)]
    /// # Safety this method is safe as long as led_x and led_y are within the bounds of the led grid
    pub unsafe fn set_led_unchecked(&mut self, led_x: usize, led_y: usize, r: f32, g: f32, b: f32) {
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
