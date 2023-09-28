use core::hint::spin_loop;

use cortex_m::register::apsr;

extern "unadjusted" {
    #[link_name = "llvm.arm.uadd8"]
    fn arm_uadd8(a: u32, b: u32) -> u32;
}

pub const BATCH_SIZE: usize = 4;

#[inline(always)]
pub fn pwm_pulse_batched(
    current_values: &mut [u8; BATCH_SIZE],
    target_values: &[u8; BATCH_SIZE],
    bit_offsets: &[usize; BATCH_SIZE],
    out_buffer: &mut u32,
) {
    *current_values = unsafe {
        arm_uadd8(
            u32::from_ne_bytes(*current_values),
            u32::from_ne_bytes(*target_values),
        )
        .to_ne_bytes()
    };
    // the overflow bits are in the APSR register bits 19-16
    let apsr = apsr::read().bits();
    *out_buffer |= ((apsr >> 16) & 0b1) << bit_offsets[0];
    *out_buffer |= ((apsr >> 17) & 0b1) << bit_offsets[1];
    *out_buffer |= ((apsr >> 18) & 0b1) << bit_offsets[2];
    *out_buffer |= ((apsr >> 19) & 0b1) << bit_offsets[3];
}

#[inline(always)]
pub fn yield_cycles<const CYCLES: u64>() {
    for _ in 0..CYCLES {
        spin_loop();
    }
}

#[inline(always)]
pub fn yield_ns<const NS: u64>() {
    for _ in 0..(NS * 6 / 10) {
        spin_loop();
    }
}
