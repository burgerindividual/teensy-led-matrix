use core::mem::transmute;

use cortex_m::register::apsr;

extern "unadjusted" {
    #[link_name = "llvm.arm.uadd8"]
    fn arm_uadd8(a: i32, b: i32) -> i32;
}

pub const BATCH_SIZE: usize = 4;

#[inline(always)]
pub fn pwm_pulse_batched(
    current_values: &mut [u8; BATCH_SIZE],
    target_values: &[u8; BATCH_SIZE],
    bit_offsets: &[usize; BATCH_SIZE],
    out_buffer: &mut u32,
) {
    unsafe {
        *current_values = transmute(arm_uadd8(
            transmute(*current_values),
            transmute(target_values),
        ));
        // the overflow bits are in the APSR register bits 19-16
        let apsr = apsr::read().bits();
        *out_buffer |= ((apsr >> 16) & 0b1) << bit_offsets[0];
        *out_buffer |= ((apsr >> 17) & 0b1) << bit_offsets[1];
        *out_buffer |= ((apsr >> 18) & 0b1) << bit_offsets[2];
        *out_buffer |= ((apsr >> 19) & 0b1) << bit_offsets[3];
    }
}
