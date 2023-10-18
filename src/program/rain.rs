use alloc::boxed::Box;

use rand::rngs::SmallRng;
use rand::{RngCore, SeedableRng};
use teensy4_bsp::hal::trng::{RetryCount, SampleMode, Trng};

use crate::collections::InlineVec;
use crate::color::Color;
use crate::driver::{FrameRate, ScreenDriver};
use crate::framebuffer::{BackBuffer, ColorLines, Framebuffer};
use crate::peripherals;
use crate::program::Program;

pub struct Rain {
    rng: SmallRng,
    raindrop_lines: [RaindropLine; Framebuffer::WIDTH],
    line_shift: usize,
}

pub type RaindropLine = InlineVec<{ Framebuffer::HEIGHT }, Raindrop>;

pub enum RaindropState {
    Falling,
    Splashing { splash_x: usize, frame: u8 },
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
    pub const RAINDROP_FREQUENCY: u32 = u32::MAX / 10;
    pub const GROUND_LEVEL: usize = Framebuffer::WIDTH - 3; // inclusive
    pub const SPLASH_FREQUENCY: u32 = u32::MAX / (Framebuffer::WIDTH - Self::GROUND_LEVEL) as u32;

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

    pub fn new() -> Box<dyn Program> {
        let mut prng_seed = [0_u8; 16];

        let mut trng = Trng::new(
            peripherals::trng(),
            SampleMode::VonNeumann,
            RetryCount::default(),
        );

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

        // disable TRNG
        trng.release_disabled();

        let prng = SmallRng::from_seed(prng_seed);

        Box::new(Self {
            rng: prng,
            raindrop_lines: Default::default(),
            line_shift: 0,
        })
    }
}

impl Program for Rain {
    fn init(&mut self, driver: &mut ScreenDriver) {
        driver.set_target_frame_rate(FrameRate::Fps64);
    }

    #[inline(always)]
    fn render(&mut self, driver: &mut ScreenDriver) {
        driver.framebuffer.back_buffer = Self::BASE_FRAME;
        driver.drive_mid_render();

        // spawn drops
        for y in 0..Framebuffer::HEIGHT {
            if self.rng.next_u32() <= Self::RAINDROP_FREQUENCY {
                self.raindrop_lines[self.line_shift].push(Raindrop::new(y as usize));
            }

            driver.drive_mid_render();
        }

        // splash maybe
        let mut x = self.line_shift;
        for _ in Self::GROUND_LEVEL..Framebuffer::WIDTH {
            if self.rng.next_u32() > Self::SPLASH_FREQUENCY {}
            driver.drive_mid_render();
        }

        // force splash
        let last_line_idx = match self.line_shift.checked_sub(1) {
            Some(val) => val,
            None => Framebuffer::WIDTH - 1,
        };

        for raindrop in self.raindrop_lines[last_line_idx].get_slice_mut() {
            raindrop.state = RaindropState::Splashing {
                splash_x: Framebuffer::WIDTH - 1,
                frame: 0,
            };
        }

        driver.drive_mid_render();

        // rasterize
        let mut falling_x = self.line_shift;

        for line in &mut self.raindrop_lines {
            for drop in line.get_slice_mut() {
                match &mut drop.state {
                    RaindropState::Falling => {
                        driver.framebuffer.back_buffer.set_led(
                            falling_x,
                            drop.y,
                            Self::RAINDROP_COLOR,
                        );
                    }
                    RaindropState::Splashing { splash_x, frame } => {
                        // these may be out of bounds, so only attempt to set the pixels
                        match frame {
                            0 => {
                                driver.framebuffer.back_buffer.try_set_led(
                                    *splash_x - 1,
                                    drop.y - 1,
                                    Self::RAINDROP_COLOR,
                                );
                                driver.framebuffer.back_buffer.try_set_led(
                                    *splash_x - 1,
                                    drop.y + 1,
                                    Self::RAINDROP_COLOR,
                                );
                                *frame += 1;
                            }
                            1 => {
                                driver.framebuffer.back_buffer.try_set_led(
                                    *splash_x,
                                    drop.y - 1,
                                    Self::RAINDROP_COLOR,
                                );
                                driver.framebuffer.back_buffer.try_set_led(
                                    *splash_x,
                                    drop.y + 1,
                                    Self::RAINDROP_COLOR,
                                );
                                *frame += 1;
                            }
                            _ => {
                                // finished splash animation
                            }
                        }
                    }
                }

                driver.drive_mid_render();
            }

            falling_x += 1;
            if falling_x >= Framebuffer::WIDTH {
                falling_x = 0;
            }
        }

        self.line_shift += 1;
        if self.line_shift >= Framebuffer::WIDTH {
            self.line_shift = 0;
        }

        self.raindrop_lines[self.line_shift].clear();
    }
}
