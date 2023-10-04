use crate::color::Color;
use crate::framebuffer::Framebuffer::{HEIGHT, WIDTH};
use crate::framebuffer::{BackBuffer, FrontBuffer};
use crate::program::{FrameRate, Program};

pub struct HueCycle {
    current_stage: u16,
}

impl HueCycle {
    pub const START_LED_RANGE: u16 = 0;
    pub const END: u16 = (Framebuffer::WIDTH * Framebuffer::HEIGHT) as u16;
}

impl HueCycle {
    pub fn new() -> Self {
        Self { current_stage: 0 }
    }
}

impl Program for HueCycle {
    #[inline(always)]
    fn frame_rate() -> FrameRate {
        FrameRate::Fps512
    }

    fn init(&mut self, back_buffer: &mut BackBuffer) {
        unsafe {
            back_buffer.set_led_unchecked(0, 3, Color::from_rgb(0, 255, 0));
            back_buffer.set_led_unchecked(0, 4, Color::from_rgb(127, 255, 0));
            back_buffer.set_led_unchecked(0, 5, Color::from_rgb(255, 255, 0));
            back_buffer.set_led_unchecked(0, 6, Color::from_rgb(255, 127, 0));
            back_buffer.set_led_unchecked(0, 7, Color::from_rgb(255, 0, 0));
        }
    }

    #[inline(always)]
    fn process_chunk(&mut self, front_buffer: &FrontBuffer, back_buffer: &mut BackBuffer) {
        if self.current_stage < Self::END {
            let y = self.current_stage as usize / Framebuffer::WIDTH;
            let x = self.current_stage as usize % Framebuffer::WIDTH;

            let mut color = unsafe { front_buffer.get_led_unchecked(x, y) };

            color.g = color
                .g
                .saturating_add((color.r == 0xFF && color.b == 0x00) as u8);
            color.g = color
                .g
                .saturating_sub((color.b == 0xFF && color.r == 0x00) as u8);

            color.b = color
                .b
                .saturating_add((color.g == 0xFF && color.r == 0x00) as u8);
            color.b = color
                .b
                .saturating_sub((color.r == 0xFF && color.g == 0x00) as u8);

            color.r = color
                .r
                .saturating_add((color.b == 0xFF && color.g == 0x00) as u8);
            color.r = color
                .r
                .saturating_sub((color.g == 0xFF && color.b == 0x00) as u8);

            unsafe {
                back_buffer.set_led_unchecked(x, y, color);
            }

            self.current_stage += 1;
        }
    }

    #[inline(always)]
    fn frame_finished(&mut self, _back_buffer: &mut BackBuffer) {
        if self.current_stage < Self::END {
            panic!();
        }

        self.current_stage = Self::START_LED_RANGE;
    }
}
