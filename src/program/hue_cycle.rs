use alloc::boxed::Box;

use crate::color::Color;
use crate::framebuffer::Framebuffer;
use crate::led_driver::{FrameRate, ScreenDriver};
use crate::program::Program;

pub struct HueCycle {
    scratch_buffer: [[Color; Framebuffer::WIDTH]; Framebuffer::HEIGHT],
}

impl HueCycle {
    pub fn new(driver: &mut ScreenDriver) -> Box<dyn Program> {
        let mut program = Box::new(Self {
            scratch_buffer: Default::default(),
        });

        for y in 0..Framebuffer::HEIGHT {
            for x in 0..Framebuffer::WIDTH {
                *unsafe {
                    program
                        .scratch_buffer
                        .get_mut(y)
                        .unwrap_unchecked()
                        .get_mut(x)
                        .unwrap_unchecked()
                } = Color::from_rgb(((x + y) * 10) as u8, 255, 0);
            }
        }
        driver.set_target_frame_rate(FrameRate::Fps512);
        // necessary to make sure the front buffer is initialized
        driver.framebuffer.flip();

        program
    }
}

impl Program for HueCycle {
    fn render(&mut self, driver: &mut ScreenDriver) {
        for y in 0..Framebuffer::HEIGHT {
            for x in 0..Framebuffer::WIDTH {
                let color = unsafe {
                    self.scratch_buffer
                        .get_mut(y)
                        .unwrap_unchecked()
                        .get_mut(x)
                        .unwrap_unchecked()
                };

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

                driver.framebuffer.back_buffer.set_led(x, y, *color);

                driver.drive_mid_render();
            }
        }
    }
}
