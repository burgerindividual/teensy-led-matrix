mod rainbow;

pub trait Program {
    /// Has to be a power of 2 with a maximum of 32768
    fn frame_rate() -> u16;
    fn init(&mut self);
    fn process_chunk(&mut self);
    fn finish_frame(&mut self);
}
