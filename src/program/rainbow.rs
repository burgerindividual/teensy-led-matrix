use crate::framebuffer::{BackBuffer, FrontBuffer};
use crate::program::Program;

pub struct Rainbow<'f> {
    front_buffer: &'f FrontBuffer,
    back_buffer: &'f mut BackBuffer,
}

impl<'f> Program for Rainbow<'f> {
    fn frame_rate() -> u16 {
        512
    }

    fn init(&mut self) {
        unsafe {
            self.back_buffer.set_led_unchecked(0, 3, 0, 255, 0);
            self.back_buffer.set_led_unchecked(0, 4, 127, 255, 0);
            self.back_buffer.set_led_unchecked(0, 5, 255, 255, 0);
            self.back_buffer.set_led_unchecked(0, 6, 255, 127, 0);
            self.back_buffer.set_led_unchecked(0, 7, 255, 0, 0);
        }
    }

    fn process_chunk(&mut self) {
        todo!()
    }

    fn finish_frame(&mut self) {
        todo!()
    }
}
