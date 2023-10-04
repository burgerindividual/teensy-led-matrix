use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};
use teensy4_bsp::hal::trng::{RetryCount, SampleMode, Trng};
use teensy4_bsp::ral::trng::TRNG;

use crate::collections::InlineVec;
use crate::color::Color;
use crate::framebuffer::{BackBuffer, ColorLines, Framebuffer, FrontBuffer};
use crate::program::{FrameRate, Program};
use crate::take_mut::take_mut;

pub struct Rain {
    rng: SmallRng,
    raindrop_lines: [RaindropLine; Framebuffer::WIDTH],
    temp_line: RaindropLine,
    line_shift: usize,
    frame_current_stage: u16,
}

pub type RaindropLine = InlineVec<{ Framebuffer::HEIGHT }, Raindrop>;

#[repr(u8)]
pub enum RaindropState {
    Falling = 0,
    Splashing = 1,
    Finished = 2,
}

pub struct Raindrop {
    y: usize,
    state: RaindropState,
}

impl Raindrop {
    pub fn new(y: usize) -> Self {
        Self {
            y,
            state: RaindropState::Falling,
        }
    }
}

impl Rain {
    pub const END_SPAWN_DROP_RANGE: u16 = Framebuffer::HEIGHT as u16;
    pub const END_SPLASH_RANGE: u16 =
        Self::END_SPAWN_DROP_RANGE + (Framebuffer::WIDTH - Self::GROUND_LEVEL) as u16;
    pub const END_RASTERIZE_LINE_RANGE: u16 = Self::END_SPLASH_RANGE + Framebuffer::WIDTH as u16;

    pub const RAINDROP_FREQUENCY: u32 = u32::MAX / 10;
    pub const GROUND_LEVEL: usize = Framebuffer::WIDTH - 3;

    pub const RAINDROP_COLOR: Color = Color::from_rgb(200, 200, 200);
    pub const GROUND_COLOR: Color = Color::from_rgb(36, 40, 43);

    pub const BASE_FRAME: BackBuffer = {
        let mut buffer = BackBuffer {
            bit_lines: [[0; Framebuffer::WIDTH]; Framebuffer::HEIGHT * ColorLines::COUNT],
        };

        // fill the ground area
        let mut y = 0_usize;
        while y < Framebuffer::HEIGHT {
            let mut x = Self::GROUND_LEVEL;
            while x < Framebuffer::WIDTH {
                buffer.bit_lines[y * ColorLines::COUNT + ColorLines::Red as usize][x] =
                    Self::GROUND_COLOR.r;
                buffer.bit_lines[y * ColorLines::COUNT + ColorLines::Green as usize][x] =
                    Self::GROUND_COLOR.g;
                buffer.bit_lines[y * ColorLines::COUNT + ColorLines::Blue as usize][x] =
                    Self::GROUND_COLOR.b;

                x += 1;
            }

            y += 1;
        }

        buffer
    };

    pub(crate) fn new(raw_trng: &mut TRNG) -> Self {
        let mut prng_seed = [0_u8; 16];

        take_mut(raw_trng, |raw_trng| {
            let mut trng = Trng::new(raw_trng, SampleMode::VonNeumann, RetryCount::default());

            // use the TRNG to seed the PRNG
            prng_seed.copy_from_slice(
                [
                    trng.next_u32().unwrap().to_ne_bytes(),
                    trng.next_u32().unwrap().to_ne_bytes(),
                    trng.next_u32().unwrap().to_ne_bytes(),
                    trng.next_u32().unwrap().to_ne_bytes(),
                ]
                .flatten(),
            );

            // return the TRNG in a disabled state
            trng.release_disabled()
        });

        let prng = SmallRng::from_seed(prng_seed);

        Self {
            rng: prng,
            raindrop_lines: Default::default(),
            temp_line: Default::default(),
            line_shift: 0,
            frame_current_stage: 0,
        }
    }
}

impl Program for Rain {
    #[inline(always)]
    fn frame_rate() -> FrameRate {
        FrameRate::Fps64
    }

    fn init(&mut self, back_buffer: &mut BackBuffer) {
        *back_buffer = Self::BASE_FRAME;
    }

    #[inline(always)]
    fn process_chunk(&mut self, _front_buffer: &FrontBuffer, back_buffer: &mut BackBuffer) {
        unsafe {
            match self.frame_current_stage {
                0..Self::END_SPAWN_DROP_RANGE => {
                    if self.rng.next_u32() <= Self::RAINDROP_FREQUENCY {
                        self.raindrop_lines[self.line_shift]
                            .push(Raindrop::new(self.frame_current_stage as usize));
                        back_buffer.set_led_unchecked(
                            0,
                            self.frame_current_stage as usize,
                            Self::RAINDROP_COLOR,
                        );
                    }
                    self.frame_current_stage += 1;
                }
                Self::END_SPAWN_DROP_RANGE..Self::END_SPLASH_RANGE => {
                    let current_line = (self.frame_current_stage - Self::END_SPAWN_DROP_RANGE
                        + Self::GROUND_LEVEL as u16)
                        % Framebuffer::WIDTH as u16;
                }
                Self::END_SPLASH_RANGE..Self::END_RASTERIZE_LINE_RANGE => {}
                _ => {
                    // we're done with the frame, so just skip
                }
            }
        }
    }

    #[inline(always)]
    fn frame_finished(&mut self, back_buffer: &mut BackBuffer) {
        if !self.raindrop_lines.is_empty()
            || self.frame_current_stage < Self::END_RASTERIZE_LINE_RANGE
        {
            panic!();
        }

        self.frame_current_stage = 0;

        self.line_shift += 1;
        self.line_shift %= Framebuffer::WIDTH;

        self.raindrop_lines[self.line_shift].clear();

        *back_buffer = Self::BASE_FRAME;
    }
}
