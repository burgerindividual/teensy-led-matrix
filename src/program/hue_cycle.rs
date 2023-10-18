use alloc::boxed::Box;

use crate::color::Color;
use crate::driver::{FrameRate, ScreenDriver};
use crate::framebuffer::Framebuffer;
use crate::program::Program;

pub struct HueCycle {}

impl HueCycle {
    pub fn new() -> Box<dyn Program> {
        Box::new(Self {})
    }
}

impl Program for HueCycle {
    fn init(&mut self, driver: &mut ScreenDriver) {
        for y in 0..Framebuffer::HEIGHT {
            for x in 0..Framebuffer::WIDTH {
                driver.framebuffer.back_buffer.set_led(
                    x,
                    y,
                    Color::from_rgb(((x + y) * 10) as u8, 255, 0),
                );
            }
        }
        driver.set_target_frame_rate(FrameRate::Fps512);
        // necessary to make sure the front buffer is initialized
        driver.framebuffer.flip();
    }

    #[inline(always)]
    fn render(&mut self, driver: &mut ScreenDriver) {
        for y in 0..Framebuffer::HEIGHT {
            for x in 0..Framebuffer::WIDTH {
                let mut color = unsafe { driver.framebuffer.front_buffer.get_led(x, y) };

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

                driver.framebuffer.back_buffer.set_led(x, y, color);

                driver.drive_mid_render();
            }
        }
    }
}
