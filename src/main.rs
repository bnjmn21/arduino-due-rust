#![no_std]
#![no_main]

mod config;
mod schedule;
extern crate alloc;
extern crate panic_halt;
extern crate sam3x8e; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use config::HEAP_SIZE;
use core::mem::MaybeUninit;
use cortex_m_rt::entry;
use embedded_alloc::Heap;
use sam3x8e::PIOB;
use schedule::Scheduler;

const BLINK_TIME: u32 = 1000;

#[global_allocator]
static HEAP: Heap = Heap::empty();

unsafe fn init_alloc() {
    static mut HEAP_MEM: [MaybeUninit<u8>; HEAP_SIZE] = [MaybeUninit::uninit(); HEAP_SIZE];
    HEAP.init(HEAP_MEM.as_ptr() as usize, HEAP_SIZE);
}

static mut PIO_B: MaybeUninit<PIOB> = MaybeUninit::uninit();

#[entry]
fn main() -> ! {
    unsafe {
        init_alloc();
    }

    let p = sam3x8e::Peripherals::take().unwrap();
    let piob = p.PIOB;
    let pmc = p.PMC;
    let rtt = p.RTT;

    pmc.pmc_pcer0.write_with_zero(|w| w.pid12().set_bit());

    piob.per.write_with_zero(|w| w.p27().set_bit());
    piob.oer.write_with_zero(|w| w.p27().set_bit());
    piob.pudr.write_with_zero(|w| w.p27().set_bit());

    unsafe {
        PIO_B = MaybeUninit::new(piob);
    }

    let mut scheduler = Scheduler::new(rtt);
    scheduler.push(
        |s| unsafe {
            PIO_B
                .assume_init_ref()
                .sodr
                .write_with_zero(|w| w.p27().set_bit());
            s.yield_for(BLINK_TIME);
            PIO_B
                .assume_init_ref()
                .codr
                .write_with_zero(|w| w.p27().set_bit());
            s.repeat_in(BLINK_TIME);
        },
        0,
    );
    scheduler.main_loop();
}
