use crate::framebuffer::{BackBuffer, FrontBuffer};

pub(crate) mod rainbow;

pub trait Program {
    /// Has to be a power of 2 with a maximum of 32768
    fn frame_rate() -> u16;
    fn init(&mut self, back_buffer: &mut BackBuffer);
    fn process_chunk(&mut self, front_buffer: &FrontBuffer, back_buffer: &mut BackBuffer);
    fn frame_finished(&mut self);
}
