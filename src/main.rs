#![no_std]
#![no_main]

//extern crate sam3x8e;
//use sam3x8e::RTT;
#[allow(dead_code)]
mod sam3x8e;
mod schedule;

extern crate panic_halt; // you can put a breakpoint on `rust_begin_unwind` to catch panics

use cortex_m_rt::entry;
use sam3x8e::pio::IoLine;
use sam3x8e::pmc::PeripheralClockAreas;
use sam3x8e::rtt::ModeAreas;

fn delay_ms(rtt: &sam3x8e::rtt::Rtt, ms: u32) {
    // We're not considering overflow here, 32 bits can keep about 49 days in ms
    let now = unsafe { rtt.value.read() };
    let until = now + ms;

    while unsafe { rtt.value.read() } < until {}
}

#[entry]
fn main() -> ! {
    const LED_PIN: IoLine = IoLine::L27;
    const DELAY: u32 = 1000;

    let piob = sam3x8e::pio::pio_b(); //parallel input output controller b
    let pmc = sam3x8e::pmc::pmc(); //power management controller
    let rtt = sam3x8e::rtt::rtt(); //real-time timer

    unsafe {
        // Enable PIOB
        pmc.peripheral_clock_0
            .write_area(PeripheralClockAreas::PeripheralClock12, 1);

        // Configure RTT resolution to approx. 1ms
        rtt.mode.write_area(ModeAreas::Prescaler, 0x20);

        // Configure PIOB pin 27 (LED)
        piob.pio.set.write_area(LED_PIN, 1);
        piob.output.set.write_area(LED_PIN, 1);
        piob.pull_up_disable.set.write_area(LED_PIN, 1);

        // On/off blinking
        loop {
            piob.output_data.set.write_area(LED_PIN, 1);
            delay_ms(&rtt, DELAY);
            piob.output_data.reset.write_area(LED_PIN, 1);
            delay_ms(&rtt, DELAY);
        }
    }
}
