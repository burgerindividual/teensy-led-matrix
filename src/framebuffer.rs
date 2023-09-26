use core::mem::variant_count;

pub const WIDTH: usize = 12;
pub const HEIGHT: usize = 8;
pub const COLOR_COUNT: usize = variant_count::<Colors>();

#[repr(u8)]
pub enum Colors {
    Red = 0,
    Green = 1,
    Blue = 2,
}

#[derive(Default)]
#[repr(align(4))] // align to batch size
pub struct LedFramebuffer {
    pub(crate) bit_target_lines: [[u8; WIDTH]; HEIGHT * COLOR_COUNT],
    pub(crate) bit_current_lines: [[u8; WIDTH]; HEIGHT * COLOR_COUNT],
}

impl LedFramebuffer {
    pub const RED_MAX: u8 = (255.0 * (430.0 / 470.0)) as u8;
    pub const GREEN_MAX: u8 = u8::MAX;
    pub const BLUE_MAX: u8 = u8::MAX;

    #[inline(always)]
    /// # Safety this method is safe as long as led_x and led_y are within the bounds of the led grid
    pub unsafe fn set_led_unchecked(&mut self, led_x: usize, led_y: usize, r: u8, g: u8, b: u8) {
        let led_start_column = led_y * COLOR_COUNT;
        *(self
            .bit_target_lines
            .get_unchecked_mut(led_start_column + (Colors::Red as usize))
            .get_unchecked_mut(led_x)) = r;
        *(self
            .bit_target_lines
            .get_unchecked_mut(led_start_column + (Colors::Green as usize))
            .get_unchecked_mut(led_x)) = g;
        *(self
            .bit_target_lines
            .get_unchecked_mut(led_start_column + (Colors::Blue as usize))
            .get_unchecked_mut(led_x)) = b;

        // we have to reset the current lines, because if we don't, the pwm function can break
        *(self
            .bit_current_lines
            .get_unchecked_mut(led_start_column + (Colors::Red as usize))
            .get_unchecked_mut(led_x)) = r;
        *(self
            .bit_current_lines
            .get_unchecked_mut(led_start_column + (Colors::Green as usize))
            .get_unchecked_mut(led_x)) = g;
        *(self
            .bit_current_lines
            .get_unchecked_mut(led_start_column + (Colors::Blue as usize))
            .get_unchecked_mut(led_x)) = b;
    }
}
