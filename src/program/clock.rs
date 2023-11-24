use alloc::boxed::Box;

use teensy4_bsp::hal::snvs;
use teensy4_bsp::hal::snvs::srtc::Srtc;
use teensy4_bsp::hal::snvs::*;

use super::Program;
use crate::color::{self, AdjustedColor, Color, BLACK};
use crate::led_driver::{FrameRate, ScreenDriver};
use crate::peripherals;

#[rustfmt::skip]
const NUMBER_STENCILS: [[[u8; 3]; 5]; 10] = [
    [
        [ 1, 1, 1, ],
        [ 1, 0, 1, ],
        [ 1, 0, 1, ],
        [ 1, 0, 1, ],
        [ 1, 1, 1, ],
    ],
    [
        [ 0, 1, 0, ],
        [ 1, 1, 0, ],
        [ 0, 1, 0, ],
        [ 0, 1, 0, ],
        [ 1, 1, 1, ],
    ],
    [
        [ 1, 1, 1, ],
        [ 0, 0, 1, ],
        [ 1, 1, 1, ],
        [ 1, 0, 0, ],
        [ 1, 1, 1, ],
    ],
    [
        [ 1, 1, 1, ],
        [ 0, 0, 1, ],
        [ 0, 1, 1, ],
        [ 0, 0, 1, ],
        [ 1, 1, 1, ],
    ],
    [
        [ 1, 0, 1, ],
        [ 1, 0, 1, ],
        [ 1, 1, 1, ],
        [ 0, 0, 1, ],
        [ 0, 0, 1, ],
    ],
    [
        [ 1, 1, 1, ],
        [ 1, 0, 0, ],
        [ 1, 1, 1, ],
        [ 0, 0, 1, ],
        [ 1, 1, 1, ],
    ],
    [
        [ 1, 1, 1, ],
        [ 1, 0, 0, ],
        [ 1, 1, 1, ],
        [ 1, 0, 1, ],
        [ 1, 1, 1, ],
    ],
    [
        [ 1, 1, 1, ],
        [ 0, 0, 1, ],
        [ 0, 0, 1, ],
        [ 0, 0, 1, ],
        [ 0, 0, 1, ],
    ],
    [
        [ 1, 1, 1, ],
        [ 1, 0, 1, ],
        [ 1, 1, 1, ],
        [ 1, 0, 1, ],
        [ 1, 1, 1, ],
    ],
    [
        [ 1, 1, 1, ],
        [ 1, 0, 1, ],
        [ 1, 1, 1, ],
        [ 0, 0, 1, ],
        [ 0, 0, 1, ],
    ],
];

#[rustfmt::skip]
const LOWER_A_STENCIL: [[u8; 3]; 3] = [
    [ 0, 1, 1, ],
    [ 1, 0, 1, ],
    [ 0, 1, 1, ],
];

#[rustfmt::skip]
const LOWER_P_STENCIL: [[u8; 3]; 3] = [
    [ 1, 1, 1, ],
    [ 1, 1, 1, ],
    [ 1, 0, 0, ],
];

#[rustfmt::skip]
const LOWER_M_STENCIL: [[u8; 4]; 3] = [
    [ 0, 1, 1, 0, ],
    [ 1, 1, 1, 1, ],
    [ 1, 0, 0, 1, ],
];

const TEXT_COLOR: AdjustedColor = Color::from_rgb(0xAA, 0xAA, 0xAA).adjust_for_led();

const NUMBER_GLYPHS: [[[AdjustedColor; 3]; 5]; 10] =
    process_glyph_array(NUMBER_STENCILS, TEXT_COLOR);
const LOWER_A_GLYPH: [[AdjustedColor; 3]; 3] = process_glyph(LOWER_A_STENCIL, TEXT_COLOR);
const LOWER_P_GLYPH: [[AdjustedColor; 3]; 3] = process_glyph(LOWER_P_STENCIL, TEXT_COLOR);
const LOWER_M_GLYPH: [[AdjustedColor; 4]; 3] = process_glyph(LOWER_M_STENCIL, TEXT_COLOR);

const fn process_glyph<const WIDTH: usize, const HEIGHT: usize>(
    stencil: [[u8; WIDTH]; HEIGHT],
    color: AdjustedColor,
) -> [[AdjustedColor; WIDTH]; HEIGHT] {
    let mut processed_colors = [[BLACK.adjust_for_led(); WIDTH]; HEIGHT];

    let mut y = 0;
    while y < HEIGHT {
        let mut x = 0;
        while x < WIDTH {
            if stencil[y][x] != 0 {
                processed_colors[y][x] = color;
            }
            x += 1;
        }
        y += 1;
    }

    processed_colors
}

const fn process_glyph_array<const WIDTH: usize, const HEIGHT: usize, const COUNT: usize>(
    stencils: [[[u8; WIDTH]; HEIGHT]; COUNT],
    color: AdjustedColor,
) -> [[[AdjustedColor; WIDTH]; HEIGHT]; COUNT] {
    let mut processed_glyphs = [[[BLACK.adjust_for_led(); WIDTH]; HEIGHT]; COUNT];

    let mut i = 0;
    while i < COUNT {
        processed_glyphs[i] = process_glyph(stencils[i], color);
        i += 1;
    }

    processed_glyphs
}

pub struct Clock {
    srtc: Srtc,
}

impl Clock {
    pub fn new(driver: &mut ScreenDriver) -> Box<dyn Program> {
        let LowPower {
            mut core,
            srtc: raw_srtc,
            ..
        } = snvs::new(peripherals::snvs()).low_power;

        let srtc = raw_srtc.enable(&mut core);

        driver.set_target_frame_rate(FrameRate::Fps1);

        Box::new(Clock { srtc })
    }
}

impl Program for Clock {
    fn render(&mut self, driver: &mut ScreenDriver) {
        let total_seconds = self.srtc.get();

        let total_minutes = total_seconds / 60;
        let minutes = total_minutes % 60;

        let total_hours = total_minutes / 60;
        let hours_24 = total_hours % 24;
        let mut hours_12 = hours_24 % 12;
        if hours_12 == 0 {
            hours_12 = 12;
        }

        let pm = hours_24 < 12;

        driver.drive_mid_render();
    }
}
