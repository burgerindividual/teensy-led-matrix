use core::arch::arm::__sel;
use core::mem::transmute;

extern "unadjusted" {
    #[link_name = "llvm.arm.uadd8"]
    fn arm_uadd8(a: i32, b: i32) -> i32;
}

#[inline(always)]
pub unsafe fn __uadd8(a: (u8, u8, u8, u8), b: (u8, u8, u8, u8)) -> (u8, u8, u8, u8) {
    transmute(arm_uadd8(transmute(a), transmute(b)))
}

// PinGroup with offsets in byte and byte offset in u32 reg
#[no_mangle]
pub extern "C" fn pwm_pulse_dsp(
    current_values: &mut (u8, u8, u8, u8),
    target_values: (u8, u8, u8, u8),
) -> u32 {
    unsafe {
        let sums = __uadd8(*current_values, target_values);
        // the lanes are selected based off if the previous function's lanes overflowed
        let selected = __sel();
        *current_values = transmute::<_, (u16, u16)>(sums & 0b0111111111111111_0111111111111111);
        sums & 0b1000000000000000_1000000000000000
    }
}
