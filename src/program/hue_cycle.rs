use crate::color::Color;
use crate::driver::{FrameRate, ScreenDriver};
use crate::framebuffer::Framebuffer;
use crate::program::Program;

pub struct HueCycle {}

impl Program for HueCycle {
    fn init(&mut self, driver: &mut ScreenDriver) {
        unsafe {
            driver
                .framebuffer
                .back_buffer
                .set_led_unchecked(0, 3, Color::from_rgb(0, 255, 0));
            driver
                .framebuffer
                .back_buffer
                .set_led_unchecked(0, 4, Color::from_rgb(127, 255, 0));
            driver
                .framebuffer
                .back_buffer
                .set_led_unchecked(0, 5, Color::from_rgb(255, 255, 0));
            driver
                .framebuffer
                .back_buffer
                .set_led_unchecked(0, 6, Color::from_rgb(255, 127, 0));
            driver
                .framebuffer
                .back_buffer
                .set_led_unchecked(0, 7, Color::from_rgb(255, 0, 0));
        }
        driver.set_target_frame_rate(FrameRate::Fps512);
        // necessary to make sure the front buffer is initialized
        driver.framebuffer.flip();
    }

    #[inline(always)]
    fn render(&mut self, driver: &mut ScreenDriver) {
        for y in 0..Framebuffer::HEIGHT {
            for x in 0..Framebuffer::WIDTH {
                let mut color = unsafe { driver.framebuffer.front_buffer.get_led_unchecked(x, y) };

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
                    driver
                        .framebuffer
                        .back_buffer
                        .set_led_unchecked(x, y, color);
                }

                driver.drive_mid_render();
            }
        }
    }
}
