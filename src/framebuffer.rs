use crate::color::{AdjustedColor, Color};

#[repr(u8)]
pub enum ColorLines {
    Red = 0,
    Green = 1,
    Blue = 2,
}

impl ColorLines {
    pub const VALUES: [ColorLines; 3] = [ColorLines::Red, ColorLines::Green, ColorLines::Blue];
    pub const COUNT: usize = Self::VALUES.len();
}

#[derive(Default)]
pub struct Framebuffer {
    pub(crate) front_buffer: FrontBuffer,
    pub(crate) back_buffer: BackBuffer,
}

#[derive(Default)]
#[repr(align(4))] // align to batch size
pub struct FrontBuffer {
    pub(crate) bit_target_lines:
        [[u8; Framebuffer::WIDTH]; Framebuffer::HEIGHT * ColorLines::COUNT],
    pub(crate) bit_current_lines:
        [[u8; Framebuffer::WIDTH]; Framebuffer::HEIGHT * ColorLines::COUNT],
}

#[derive(Default)]
#[repr(align(4))] // align to batch size
pub struct BackBuffer {
    pub(crate) bit_lines: [[u8; Framebuffer::WIDTH]; Framebuffer::HEIGHT * ColorLines::COUNT],
}

impl Framebuffer {
    pub const WIDTH: usize = 12;
    pub const HEIGHT: usize = 8;

    pub fn flip(&mut self) {
        self.front_buffer.bit_target_lines = self.back_buffer.bit_lines;
    }
}

impl BackBuffer {
    pub fn try_set_led(&mut self, led_x: usize, led_y: usize, color: Color) {
        self.try_set_led_adjusted(led_x, led_y, color.adjust_for_led())
    }

    pub fn try_set_led_adjusted(&mut self, led_x: usize, led_y: usize, color: AdjustedColor) {
        if led_x < Framebuffer::WIDTH && led_y < Framebuffer::HEIGHT {
            self.set_led_adjusted(led_x, led_y, color);
        }
    }

    pub fn set_led(&mut self, led_x: usize, led_y: usize, color: Color) {
        self.set_led_adjusted(led_x, led_y, color.adjust_for_led());
    }

    pub fn set_led_adjusted(&mut self, led_x: usize, led_y: usize, color: AdjustedColor) {
        debug_assert!(led_x < Framebuffer::WIDTH);
        debug_assert!(led_y < Framebuffer::HEIGHT);

        unsafe {
            let led_start_column = led_y * ColorLines::COUNT;
            *(self
                .bit_lines
                .get_unchecked_mut(led_start_column + (ColorLines::Red as usize))
                .get_unchecked_mut(led_x)) = color.r;
            *(self
                .bit_lines
                .get_unchecked_mut(led_start_column + (ColorLines::Green as usize))
                .get_unchecked_mut(led_x)) = color.g;
            *(self
                .bit_lines
                .get_unchecked_mut(led_start_column + (ColorLines::Blue as usize))
                .get_unchecked_mut(led_x)) = color.b;
        }
    }
}
