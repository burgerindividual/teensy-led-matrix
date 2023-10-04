use crate::color::Color;

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

    #[inline(always)]
    pub fn flip(&mut self) {
        self.front_buffer.bit_target_lines = self.back_buffer.bit_lines;
    }
}

impl FrontBuffer {
    pub const RED_MAX: u8 = (255.0 * (430.0 / 470.0)) as u8;
    pub const GREEN_MAX: u8 = u8::MAX;
    pub const BLUE_MAX: u8 = u8::MAX;

    #[inline(always)]
    /// # Safety:
    /// This method is safe as long as led_x and led_y are within the bounds of the led grid
    pub unsafe fn get_led_unchecked(&self, led_x: usize, led_y: usize) -> Color {
        let led_start_column = led_y * ColorLines::COUNT;
        let r = *(self
            .bit_target_lines
            .get_unchecked(led_start_column + (ColorLines::Red as usize))
            .get_unchecked(led_x));
        let g = *(self
            .bit_target_lines
            .get_unchecked(led_start_column + (ColorLines::Green as usize))
            .get_unchecked(led_x));
        let b = *(self
            .bit_target_lines
            .get_unchecked(led_start_column + (ColorLines::Blue as usize))
            .get_unchecked(led_x));

        Color::from_rgb(r, g, b)
    }
}

impl BackBuffer {
    #[inline(always)]
    /// # Safety:
    /// This method is safe as long as led_x and led_y are within the bounds of the led grid
    pub unsafe fn set_led_unchecked(&mut self, led_x: usize, led_y: usize, color: Color) {
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
