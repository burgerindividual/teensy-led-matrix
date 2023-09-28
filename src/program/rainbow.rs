use crate::framebuffer::{BackBuffer, FrontBuffer, HEIGHT, WIDTH};
use crate::program::Program;

#[derive(Default)]
pub struct Rainbow {
    current_stage: u16,
}

impl Rainbow {
    pub const START_LED_RANGE: u16 = 0;
    pub const END: u16 = (WIDTH * HEIGHT) as u16;
}

impl Program for Rainbow {
    #[inline(always)]
    fn frame_rate() -> u16 {
        512
    }

    fn init(&mut self, back_buffer: &mut BackBuffer) {
        unsafe {
            back_buffer.set_led_unchecked(0, 3, 0, 255, 0);
            back_buffer.set_led_unchecked(0, 4, 127, 255, 0);
            back_buffer.set_led_unchecked(0, 5, 255, 255, 0);
            back_buffer.set_led_unchecked(0, 6, 255, 127, 0);
            back_buffer.set_led_unchecked(0, 7, 255, 0, 0);
        }
    }

    #[inline(always)]
    fn process_chunk(&mut self, front_buffer: &FrontBuffer, back_buffer: &mut BackBuffer) {
        if self.current_stage < Self::END {
            let y = self.current_stage as usize / WIDTH;
            let x = self.current_stage as usize % WIDTH;

            let (mut r, mut g, mut b) = unsafe { front_buffer.get_led_unchecked(x, y) };

            g = g.saturating_add((r == 0xFF && b == 0x00) as u8);
            g = g.saturating_sub((b == 0xFF && r == 0x00) as u8);

            b = b.saturating_add((g == 0xFF && r == 0x00) as u8);
            b = b.saturating_sub((r == 0xFF && g == 0x00) as u8);

            r = r.saturating_add((b == 0xFF && g == 0x00) as u8);
            r = r.saturating_sub((g == 0xFF && b == 0x00) as u8);

            unsafe {
                back_buffer.set_led_unchecked(x, y, r, g, b);
            }

            self.current_stage += 1;
        }
    }

    #[inline(always)]
    fn frame_finished(&mut self) {
        self.current_stage = Self::START_LED_RANGE;
    }
}
