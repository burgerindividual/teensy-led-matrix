use core::hint::spin_loop;
use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;

use cortex_m::register::apsr;
use embedded_alloc::Heap;
use teensy4_bsp::board::ARM_FREQUENCY;
use teensy4_bsp::rt::{heap_end, heap_start};

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

/// good option for shorter, less precise timing requirements (this will also use less power)
#[inline(always)]
pub fn yield_cycles<const CYCLES: u32>() {
    // 148 seems to be the maximum that LLVM wants to unroll this, so do batches of 148 first
    for _ in 0..(CYCLES / 148) {
        for _ in 0..148 {
            spin_loop();
        }
    }
    // do remaining
    for _ in 0..(CYCLES % 148) {
        spin_loop();
    }
}

pub const fn ns_to_cycles<const NS: u32>() -> u32 {
    ((NS as u64) * (ARM_FREQUENCY as u64)).div_ceil(1_000_000_000_u64) as u32
}

pub fn init_heap(heap: &Heap) {
    // // the runtime already has a dedicated spot in OCRAM for us to use as the heap
    // let heap_start = heap_start() as usize;
    // let heap_size = heap_start - heap_end() as usize;
    unsafe {
        const HEAP_SIZE: usize = 16 * 1024;
        static mut HEAP_AREA: MaybeUninit<[u8; HEAP_SIZE]> = MaybeUninit::uninit();
        heap.init(addr_of_mut!(HEAP_AREA) as usize, HEAP_SIZE);
    }
}
