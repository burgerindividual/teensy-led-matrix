use core::arch::asm;
use core::mem::MaybeUninit;
use core::ptr::addr_of_mut;

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

#[inline(always)]
pub fn yield_cycles<const CYCLES: u32>() {
    const SETUP_TIME: u32 = 4;
    const SYSTICK_MAX_CYCLES: u32 = (0b1 << 24) - 1 + SETUP_TIME;

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
        16..SYSTICK_MAX_CYCLES => {
            let mut systick = peripherals::syst();
            systick.set_reload(CYCLES - SETUP_TIME); // minus one here?
            systick.clear_current();
            systick.enable_counter();

            wfe();

            systick.disable_counter();
        }
        _ => unreachable!(),
    }
}
