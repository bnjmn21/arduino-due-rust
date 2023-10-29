use super::{
    from_mut_ptr, register_type, MaskedRegisterWriter, RegisterField, RegisterReader, RegisterType,
    RegisterWriter, Ro, Rw,
};

register_type!(
    Mode;
    prescaler: u16 => 0,
    alarm_interrupt: bool => 16,
    increment_interrupt: bool => 17,
    restart: bool => 18
);

register_type!(
    Status;
    alarm: bool => 0,
    timer_increment: bool => 1
);

#[repr(C)]
struct RegisterBlock {
    pub mode: Rw<Mode>,
    pub alarm: Rw<u32>,
    pub value: Ro<u32>,
    pub status: Ro<Status>,
}

pub struct TimerStatus {
    pub alarm: bool,
    pub timer_increment: bool,
}

pub struct RealTimeTimer {
    register_block: &'static mut RegisterBlock,
}

impl RealTimeTimer {
    pub fn new() -> RealTimeTimer {
        RealTimeTimer {
            register_block: unsafe { from_mut_ptr!(0x400E1A30, RegisterBlock) },
        }
    }

    pub const APPROX_1MS_PRESCALER: u32 = 0x20;

    pub fn set_prescaler(&mut self, value: u16) {
        unsafe {
            self.register_block.mode.set_fields(|w| w.prescaler(value));
        }
    }

    pub fn enable_alarm(&mut self) {
        unsafe {
            self.register_block
                .mode
                .set_fields(|w| w.alarm_interrupt(true));
        }
    }

    pub fn disable_alarm(&mut self) {
        unsafe {
            self.register_block
                .mode
                .set_fields(|w| w.alarm_interrupt(false));
        }
    }

    pub fn enable_increment_interrupt(&mut self) {
        unsafe {
            self.register_block
                .mode
                .set_fields(|w| w.increment_interrupt(true));
        }
    }

    pub fn disable_increment_interrupt(&mut self) {
        unsafe {
            self.register_block
                .mode
                .set_fields(|w| w.increment_interrupt(false));
        }
    }

    pub fn restart(&mut self) {
        unsafe {
            self.register_block.mode.set_fields(|w| w.restart(true));
        }
    }

    pub fn set_alarm(&mut self, value: u32) {
        unsafe {
            self.register_block.alarm.write(value);
        }
    }

    pub fn get_time(&self) -> u32 {
        unsafe { self.register_block.value.read().value }
    }

    pub fn get_status(&self) -> TimerStatus {
        let timer_status = unsafe { self.register_block.status.read() };
        TimerStatus {
            alarm: timer_status.alarm(),
            timer_increment: timer_status.timer_increment(),
        }
    }
}
