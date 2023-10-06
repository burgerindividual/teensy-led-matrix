// use core::mem::swap;
//
// use rand::rngs::SmallRng;
// use rand::{RngCore, SeedableRng};
// use teensy4_bsp::hal::trng::{RetryCount, SampleMode, Trng};
// use teensy4_bsp::ral::trng::TRNG;
//
// use crate::collections::InlineVec;
// use crate::color::Color;
// use crate::driver::ScreenDriver;
// use crate::framebuffer::{BackBuffer, ColorLines, Framebuffer, FrontBuffer};
// use crate::program::{Program};
// use crate::take_mut::take_mut;
//
// pub struct Rain {
//     rng: SmallRng,
//     raindrop_lines: [RaindropLine; Framebuffer::WIDTH],
//     temp_line: RaindropLine,
//     line_shift: usize,
//     frame_current_stage: u16,
// }
//
// pub type RaindropLine = InlineVec<{ Framebuffer::HEIGHT }, Raindrop>;
//
// #[repr(u8)]
// pub enum RaindropState {
//     Falling = 0,
//     Splashing = 1,
// }
//
// pub struct Raindrop {
//     y: usize,
//     state: RaindropState,
// }
//
// impl Raindrop {
//     pub fn new(y: usize) -> Self {
//         Self {
//             y,
//             state: RaindropState::Falling,
//         }
//     }
// }
//
// impl Rain {
//     pub const END_SPAWN_DROP_RANGE: u16 = Framebuffer::HEIGHT as u16;
//
//     pub const END_MAYBE_SPLASH_RANGE: u16 =
//         Self::END_SPAWN_DROP_RANGE + (Framebuffer::WIDTH - Self::GROUND_LEVEL) as u16 - 1;
//
//     pub const END_FORCE_SPLASH_RANGE: u16 = Self::END_MAYBE_SPLASH_RANGE + 1;
//
//     pub const END_RASTERIZE_LINE_RANGE: u16 =
//         Self::END_FORCE_SPLASH_RANGE + Framebuffer::WIDTH as u16;
//
//     pub const RAINDROP_FREQUENCY: u32 = u32::MAX / 10;
//     pub const GROUND_LEVEL: usize = Framebuffer::WIDTH - 3;
//     pub const SPLASH_FREQUENCY: u32 = u32::MAX / (Framebuffer::WIDTH - Self::GROUND_LEVEL) as u32;
//
//     pub const RAINDROP_COLOR: Color = Color::from_rgb(200, 200, 200);
//     pub const GROUND_COLOR: Color = Color::from_rgb(36, 40, 43);
//
//     pub const BASE_FRAME: BackBuffer = {
//         let mut buffer = BackBuffer {
//             bit_lines: [[0; Framebuffer::WIDTH]; Framebuffer::HEIGHT * ColorLines::COUNT],
//         };
//
//         // fill the ground area
//         let mut y = 0_usize;
//         while y < Framebuffer::HEIGHT {
//             let mut x = Self::GROUND_LEVEL;
//             while x < Framebuffer::WIDTH {
//                 buffer.bit_lines[y * ColorLines::COUNT + ColorLines::Red as usize][x] =
//                     Self::GROUND_COLOR.r;
//                 buffer.bit_lines[y * ColorLines::COUNT + ColorLines::Green as usize][x] =
//                     Self::GROUND_COLOR.g;
//                 buffer.bit_lines[y * ColorLines::COUNT + ColorLines::Blue as usize][x] =
//                     Self::GROUND_COLOR.b;
//
//                 x += 1;
//             }
//
//             y += 1;
//         }
//
//         buffer
//     };
//
//     pub(crate) fn new(raw_trng: &mut TRNG) -> Self {
//         let mut prng_seed = [0_u8; 16];
//
//         take_mut(raw_trng, |raw_trng| {
//             let mut trng = Trng::new(raw_trng, SampleMode::VonNeumann, RetryCount::default());
//
//             // use the TRNG to seed the PRNG
//             prng_seed.copy_from_slice(
//                 [
//                     trng.next_u32().unwrap().to_ne_bytes(),
//                     trng.next_u32().unwrap().to_ne_bytes(),
//                     trng.next_u32().unwrap().to_ne_bytes(),
//                     trng.next_u32().unwrap().to_ne_bytes(),
//                 ]
//                 .flatten(),
//             );
//
//             // return the TRNG in a disabled state
//             trng.release_disabled()
//         });
//
//         let prng = SmallRng::from_seed(prng_seed);
//
//         Self {
//             rng: prng,
//             raindrop_lines: Default::default(),
//             temp_line: Default::default(),
//             line_shift: 0,
//             frame_current_stage: 0,
//         }
//     }
// }
//
// impl Program for Rain {
//     fn init(&mut self, driver: &mut ScreenDriver) {
//         driver.framebuffer.back_buffer = Self::BASE_FRAME;
//     }
//
//     #[inline(always)]
//     fn render(&mut self, driver: &mut ScreenDriver) {
//         unsafe {
//             match self.frame_current_stage {
//                 0..Self::END_SPAWN_DROP_RANGE => {
//                     if self.rng.next_u32() <= Self::RAINDROP_FREQUENCY {
//                         self.raindrop_lines[self.line_shift]
//                             .push(Raindrop::new(self.frame_current_stage as usize));
//                         back_buffer.set_led_unchecked(
//                             0,
//                             self.frame_current_stage as usize,
//                             Self::RAINDROP_COLOR,
//                         );
//                     }
//                     self.frame_current_stage += 1;
//                 }
//                 Self::END_SPAWN_DROP_RANGE..Self::END_FORCE_SPLASH_RANGE => {
//                     let current_line = (self.frame_current_stage as usize
//                         - Self::END_SPAWN_DROP_RANGE as usize
//                         + Self::GROUND_LEVEL
//                         + self.line_shift)
//                         % Framebuffer::WIDTH;
//
//                     if !self.raindrop_lines[current_line].is_empty() {
//                         if self.rng.next_u32() > Self::SPLASH_FREQUENCY {}
//                     } else {
//                         swap(&mut self.raindrop_lines[current_line], &mut self.temp_line);
//                     }
//                 }
//                 Self::END_MAYBE_SPLASH_RANGE..Self::END_FORCE_SPLASH_RANGE => {
//                     let last_line = (self.line_shift - 1) % Framebuffer::WIDTH;
//
//                     for raindrop in self.raindrop_lines[last_line].get_slice_mut() {
//                         raindrop.state = RaindropState::Splashing;
//                     }
//                 }
//                 Self::END_FORCE_SPLASH_RANGE..Self::END_RASTERIZE_LINE_RANGE => {}
//                 _ => {
//                     // we're done with the frame, so just skip
//                 }
//             }
//         }
//
//         self.frame_current_stage = 0;
//
//         self.line_shift += 1;
//         self.line_shift %= Framebuffer::WIDTH;
//
//         self.raindrop_lines[self.line_shift].clear();
//
//         driver.framebuffer.back_buffer = Self::BASE_FRAME;
//     }
// }
