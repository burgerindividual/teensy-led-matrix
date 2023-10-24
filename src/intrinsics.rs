use core::arch::asm;
use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;
use core::sync::atomic::compiler_fence;

use cortex_m::asm::wfe;
use cortex_m::register::apsr;
use embedded_alloc::Heap;
use teensy4_bsp::board::ARM_FREQUENCY;

use crate::peripherals;

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

pub const fn ns_to_cycles<const NS: u64>() -> u64 {
    (NS * (ARM_FREQUENCY as u64)).div_ceil(1_000_000_000_u64)
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

#[inline(always)]
pub fn yield_cycles<const CYCLES: u64>() {
    const START_LOOP: u64 = 0xFFFFFF + SETUP_TIME_SINGLE + 2;
    // estimated using LLVM MCA
    const SETUP_TIME_SINGLE: u64 = 20;
    const SETUP_TIME_LOOP: u64 = 38;

    // create predefined blocks of yeilds to prevent reordering.
    // when feasible, use a timer and halt the processor until the timer causes an event.
    match CYCLES {
        1 => unsafe {
            asm!("yield");
        },
        2 => unsafe {
            asm!("yield", "yield");
        },
        3 => unsafe {
            asm!("yield", "yield", "yield");
        },
        4 => unsafe {
            asm!("yield", "yield", "yield", "yield");
        },
        5 => unsafe {
            asm!("yield", "yield", "yield", "yield", "yield");
        },
        6 => unsafe {
            asm!("yield", "yield", "yield", "yield", "yield", "yield");
        },
        7 => unsafe {
            asm!("yield", "yield", "yield", "yield", "yield", "yield", "yield");
        },
        8 => unsafe {
            asm!("yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield");
        },
        9 => unsafe {
            asm!("yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield");
        },
        10 => unsafe {
            asm!(
                "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield",
                "yield"
            );
        },
        11 => unsafe {
            asm!(
                "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield",
                "yield", "yield"
            );
        },
        12 => unsafe {
            asm!(
                "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield",
                "yield", "yield", "yield"
            );
        },
        13 => unsafe {
            asm!(
                "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield",
                "yield", "yield", "yield", "yield"
            );
        },
        14 => unsafe {
            asm!(
                "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield",
                "yield", "yield", "yield", "yield", "yield"
            );
        },
        15 => unsafe {
            asm!(
                "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield",
                "yield", "yield", "yield", "yield", "yield", "yield"
            );
        },
        16 => unsafe {
            asm!(
                "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield",
                "yield", "yield", "yield", "yield", "yield", "yield", "yield"
            );
        },
        17 => unsafe {
            asm!(
                "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield",
                "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield"
            );
        },
        18 => unsafe {
            asm!(
                "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield",
                "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield"
            );
        },
        19 => unsafe {
            asm!(
                "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield",
                "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield",
                "yield"
            );
        },
        20 => unsafe {
            asm!(
                "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield",
                "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield", "yield",
                "yield", "yield"
            );
        },
        21..START_LOOP => {
            let wait_cycles = (CYCLES - SETUP_TIME_SINGLE - 1) as u32;
            systick_yield(wait_cycles);
        }
        _ => {
            let wait_cycles = CYCLES - SETUP_TIME_LOOP - 1;
            let loop_count = wait_cycles >> 24;
            let remainder = wait_cycles as u32 & 0xFFFFFF;

            for _ in 0..loop_count {
                systick_yield(0xFFFFFF);
            }

            if remainder > 0 {
                systick_yield(remainder);
            }
        }
    }
}

#[inline(always)]
fn systick_yield(cycles: u32) {
    let mut systick = peripherals::syst();
    systick.set_reload(cycles); // minus one here?
    systick.clear_current();
    systick.enable_counter();
    // these must not be reordered
    unsafe {
        asm!("sev", "wfe", "wfe");
    }
    systick.disable_counter();
    unsafe {
        let scb = peripherals::scb();
        // clear the pending systick interrupt
        scb.icsr.modify(|icsr| icsr | (0b1 << 25));
        // clear the pending systick exception
        // scb.shcsr.modify(|shcsr| shcsr & !(0b1 << 11));
    }
}
