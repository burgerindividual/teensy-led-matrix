use core::hint::spin_loop;

use cortex_m::peripheral::DWT;
use teensy4_bsp::hal::ccm::clock_gate;
use teensy4_bsp::hal::iomuxc::gpio::Pin;
use teensy4_bsp::pins::t40::{ErasedPins, P2, P3};
use teensy4_bsp::ral;
use teensy4_bsp::ral::ccm::CCM;
use teensy4_bsp::ral::gpio::{GPIO6, GPIO9};
use teensy4_bsp::ral::iomuxc_gpr::IOMUXC_GPR;
use teensy4_bsp::ral::snvs::SNVS;
use teensy4_bsp::ral::{modify_reg, read_reg, write_reg};

use crate::framebuffer::{ColorLines, Framebuffer};
use crate::intrinsics::{ns_to_cycles, pwm_pulse_batched, yield_cycles, BATCH_SIZE};
use crate::pins::*;

#[repr(u32)]
#[rustfmt::skip]
#[derive(Copy, Clone)]
// The u32 representation of FrameRate
pub enum FrameRate {
    Fps32768 = 0b11111111111111111111111111111111_u32,
    Fps16384 = 0b11111111111111111111111111111110_u32,
    Fps8192  = 0b11111111111111111111111111111100_u32,
    Fps4096  = 0b11111111111111111111111111111000_u32,
    Fps2048  = 0b11111111111111111111111111110000_u32,
    Fps1024  = 0b11111111111111111111111111100000_u32,
    Fps512   = 0b11111111111111111111111111000000_u32,
    Fps256   = 0b11111111111111111111111110000000_u32,
    Fps128   = 0b11111111111111111111111100000000_u32,
    Fps64    = 0b11111111111111111111111000000000_u32,
    Fps32    = 0b11111111111111111111110000000000_u32,
    Fps16    = 0b11111111111111111111100000000000_u32,
    Fps8     = 0b11111111111111111111000000000000_u32,
    Fps4     = 0b11111111111111111110000000000000_u32,
    Fps2     = 0b11111111111111111100000000000000_u32,
    Fps1     = 0b11111111111111111000000000000000_u32,
    Spf2     = 0b11111111111111110000000000000000_u32,
    Spf4     = 0b11111111111111100000000000000000_u32,
    Spf8     = 0b11111111111111000000000000000000_u32,
    Spf16    = 0b11111111111110000000000000000000_u32,
    Spf32    = 0b11111111111100000000000000000000_u32,
    Spf64    = 0b11111111111000000000000000000000_u32,
    Spf128   = 0b11111111110000000000000000000000_u32,
    Spf256   = 0b11111111100000000000000000000000_u32,
    Spf512   = 0b11111111000000000000000000000000_u32,
    Spf1024  = 0b11111110000000000000000000000000_u32,
    Spf2048  = 0b11111100000000000000000000000000_u32,
    Spf4096  = 0b11111000000000000000000000000000_u32,
    Spf8192  = 0b11110000000000000000000000000000_u32,
    Spf16384 = 0b11100000000000000000000000000000_u32,
    Spf32768 = 0b11000000000000000000000000000000_u32,
    Spf65536 = 0b10000000000000000000000000000000_u32,
}

impl FrameRate {
    pub const fn rtc_mask(&self) -> u32 {
        *self as u32
    }
}

#[derive(Copy, Clone)]
enum DriverState {
    ClockOn,
    ClockOffDataOut,
}

impl DriverState {
    pub const fn pre_delay_cycles(&self) -> u32 {
        match self {
            DriverState::ClockOn => ns_to_cycles::<22>(), // 22 at 4.5v, 110 at 2v
            DriverState::ClockOffDataOut => ns_to_cycles::<25>(), // 25 at 4.5v, 125 at 2v
        }
    }
}

pub struct ScreenDriver {
    rtc_mask: u32,

    pub framebuffer: Framebuffer,
    pub current_shift_bit: u8,
    state: DriverState,
    delay_start_cycles: u32,
    clock_pulse_bits: u32,
    last_rtc_val: u32,

    gpio6: GPIO6,
    gpio9: GPIO9,
    snvs: SNVS,
}

impl ScreenDriver {
    pub const SHIFT_COUNT: u8 = (Framebuffer::HEIGHT * ColorLines::COUNT) as u8;

    pub fn new(
        gpio6: GPIO6,
        gpio9: GPIO9,
        snvs: SNVS,
        iomuxc_gpr: &mut IOMUXC_GPR,
        ccm: &mut CCM,
        erased_pins: &mut ErasedPins,
    ) -> Self {
        // configure LED output pins
        for idx in LED_OUTPUT_PIN_INDICES {
            led_output_pin_setup(&mut erased_pins[idx]);
        }

        // configure clock pins
        clock_pin_setup(&mut erased_pins[2]);
        clock_pin_setup(&mut erased_pins[3]);

        // activate high-speed GPIO with our used pins
        write_reg!(ral::gpio, iomuxc_gpr, GPR26, GPIO6_PIN_MASK);
        write_reg!(ral::gpio, iomuxc_gpr, GPR29, GPIO9_PIN_MASK);

        // set directions for GPIO pins
        modify_reg!(ral::gpio, gpio6, GDIR, |gdir| gdir | GPIO6_PIN_MASK);
        modify_reg!(ral::gpio, gpio9, GDIR, |gdir| gdir | GPIO9_PIN_MASK);

        // enable SNVS HP clock gate because the RTC is on it
        clock_gate::snvs_hp().set(ccm, clock_gate::ON);

        // enable RTC and wait for it to get set
        modify_reg!(ral::snvs, snvs, HPCR, RTC_EN: RTC_EN_1);
        while read_reg!(ral::snvs, snvs, HPCR, RTC_EN != RTC_EN_1) {
            spin_loop();
        }

        Self {
            rtc_mask: FrameRate::Fps64.rtc_mask(),
            framebuffer: Framebuffer::default(),
            current_shift_bit: 0,
            state: DriverState::ClockOn,
            delay_start_cycles: DWT::cycle_count(),
            clock_pulse_bits: 0,
            last_rtc_val: 0,
            gpio6,
            gpio9,
            snvs,
        }
    }

    pub fn set_target_frame_rate(&mut self, frame_rate: FrameRate) {
        self.rtc_mask = frame_rate.rtc_mask();
    }

    #[inline(always)]
    pub fn drive_mid_render(&mut self) {
        if DWT::cycle_count().wrapping_sub(self.delay_start_cycles) >= self.state.pre_delay_cycles()
        {
            match self.state {
                DriverState::ClockOn => {
                    self.drive_clock_on();
                    self.state = DriverState::ClockOffDataOut;
                }
                DriverState::ClockOffDataOut => {
                    self.drive_clock_off_data_out::<false>();
                    self.state = DriverState::ClockOn;
                }
            }

            self.delay_start_cycles = DWT::cycle_count();
        }
    }

    #[inline(always)]
    pub fn drive_post_render(&mut self) {
        let mut frame_flipped = false;

        while !frame_flipped {
            match self.state {
                DriverState::ClockOn => {
                    yield_cycles::<{ DriverState::ClockOn.pre_delay_cycles() }>();
                    self.drive_clock_on();
                    self.state = DriverState::ClockOffDataOut;
                }
                DriverState::ClockOffDataOut => {
                    yield_cycles::<{ DriverState::ClockOffDataOut.pre_delay_cycles() }>();
                    frame_flipped = self.drive_clock_off_data_out::<true>();
                    self.state = DriverState::ClockOn;
                }
            }
        }
    }

    #[inline(always)]
    fn drive_clock_on(&mut self) {
        self.clock_pulse_bits = 0b1 << P2::OFFSET;
        self.clock_pulse_bits |= if self.current_shift_bit == 0 {
            0b1 << P3::OFFSET
        } else {
            0
        };

        write_reg!(ral::gpio, self.gpio9, DR_SET, self.clock_pulse_bits);
    }

    #[inline(always)]
    fn drive_clock_off_data_out<const ALLOW_FRAME_FLIP: bool>(&mut self) -> bool {
        write_reg!(ral::gpio, self.gpio9, DR_CLEAR, self.clock_pulse_bits);

        // between the clock pulse and the serial output changing, 3 cycles of delay is expected.
        // in any scenario, this is already satisfied by the code setting up the next serial output,
        // so it should be fine to exclude an excess yield.

        let mut frame_flipped = false;

        self.current_shift_bit += 1;
        if self.current_shift_bit == Self::SHIFT_COUNT {
            self.current_shift_bit = 0;

            if ALLOW_FRAME_FLIP {
                // the mask chooses which bits are tested against, which can effectively set the
                // framerate
                let current_rtc_val = read_reg!(ral::snvs, self.snvs, HPRTCLR) & self.rtc_mask;

                // Frame advance is done here to effectively cause a vertical sync, as we
                // will only be updating the FB after all scanlines are written.
                if self.last_rtc_val != current_rtc_val {
                    self.last_rtc_val = current_rtc_val;
                    self.framebuffer.flip();
                    frame_flipped = true;
                }
            }
        }

        let target_values = unsafe {
            self.framebuffer
                .front_buffer
                .bit_target_lines
                .get_unchecked_mut(self.current_shift_bit as usize)
        };
        let current_values = unsafe {
            self.framebuffer
                .front_buffer
                .bit_current_lines
                .get_unchecked_mut(self.current_shift_bit as usize)
        };

        let mut gpio6_out_buffer = 0_u32;

        for ((current_value_batch, target_value_batch), pin_offset_batch) in current_values
            .array_chunks_mut::<BATCH_SIZE>()
            .zip(target_values.array_chunks::<BATCH_SIZE>())
            .zip(GPIO6_BATCHED_PIN_OFFSETS.iter())
        {
            pwm_pulse_batched(
                current_value_batch,
                target_value_batch,
                pin_offset_batch,
                &mut gpio6_out_buffer,
            );
        }

        write_reg!(ral::gpio, self.gpio6, DR, gpio6_out_buffer);

        frame_flipped
    }
}
