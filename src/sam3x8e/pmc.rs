use core::convert::TryInto;
use core::marker::PhantomData;

use super::special_ints::{U11, U24, U4, U6};
use super::{
    from_mut_ptr, register_field_enum, register_type, reref, Byte16, Byte4, Byte8,
    MaskedRegisterWriter, RegisterField, RegisterReader, RegisterType, RegisterWriter, Reserved,
    Ro, Rw, Sr, Srs, Wo,
};

register_type!(
    SystemClock;
    usb_otg_clock: bool => 5,
    programmable_clock_0: bool => 8,
    programmable_clock_1: bool => 9,
    programmable_clock_2: bool => 10
);

register_type!(
    PeripheralClock0;
    pid2: bool => 2,
    pid3: bool => 3,
    pid4: bool => 4,
    pid5: bool => 5,
    pid6: bool => 6,
    pid7: bool => 7,
    pid8: bool => 8,
    pid9: bool => 9,
    pid10: bool => 10,
    pid11: bool => 11,
    pid12: bool => 12,
    pid13: bool => 13,
    pid14: bool => 14,
    pid15: bool => 15,
    pid16: bool => 16,
    pid17: bool => 17,
    pid18: bool => 18,
    pid19: bool => 19,
    pid20: bool => 20,
    pid21: bool => 21,
    pid22: bool => 22,
    pid23: bool => 23,
    pid24: bool => 24,
    pid25: bool => 25,
    pid26: bool => 26,
    pid27: bool => 27,
    pid28: bool => 28,
    pid29: bool => 29,
    pid30: bool => 30,
    pid31: bool => 31
);

register_type!(
    PeripheralClock1;
    pid32: bool => 0,
    pid33: bool => 1,
    pid34: bool => 2,
    pid35: bool => 3,
    pid36: bool => 4,
    pid37: bool => 5,
    pid38: bool => 6,
    pid39: bool => 7,
    pid40: bool => 8,
    pid41: bool => 9,
    pid42: bool => 10,
    pid43: bool => 11,
    pid44: bool => 12
);

register_type!(
    UtmiClock;
    utmi_pll: bool => 16,
    utmi_pll_start_up_time: U4 => 20
);

register_field_enum!(
    OnChipRcFrequencyMHz, 0b111;
    Four = 0,
    Eight = 1,
    Twelve = 2
);

register_field_enum!(
    Oscillator, 0b1;
    OnChipRc = 0,
    CrystalOscillator = 1
);

register_type!(
    MainOscillator;
    crystal_oscillator: bool => 0,
    crystal_oscillator_bypass: bool => 1,
    on_chip_rc: bool => 3,
    on_chip_rc_frequency: OnChipRcFrequencyMHz => 4,
    crystal_oscillator_startup_time: u8 => 8,
    key: u8 => 16,
    oscillator_selection: Oscillator => 24,
    clock_failure_detector: bool => 15
);

register_type!(
    ClockFrequency;
    clock_frequency: u16 => 0,
    clock_ready: bool => 16
);

#[repr(u8)]
#[derive(PartialEq)]
pub enum Divider {
    Zero = 0,
    Bypass = 1,
    N(u8) = 2,
}

impl RegisterField for Divider {
    const MASK: u32 = 0b1111_1111;
    fn from_u32(value: u32) -> Self {
        if value == 0 {
            Divider::Zero
        } else if value == 1 {
            Divider::Bypass
        } else {
            unsafe { Divider::N(value.try_into().unwrap_unchecked()) }
        }
    }
    fn into_u32(self) -> u32 {
        match self {
            Divider::Zero => 0,
            Divider::Bypass => 1,
            Divider::N(value) => value as u32,
        }
    }
}

register_type!(
    Plla;
    divider: Divider => 0,
    plla_counter: U6 => 8,
    plla_multiplier: U11 => 16,
    allow_write: bool => 29
);

register_field_enum!(
    MasterClockSource, 0b11;
    Slow = 0,
    Main = 1,
    Plla = 2,
    Upll = 3
);

register_field_enum!(
    ProcessorClockPrescaler, 0b111;
    Normal = 0,
    Div2 = 1,
    Div4 = 2,
    Div8 = 3,
    Div16 = 4,
    Div32 = 5,
    Div64 = 6,
    Div3 = 7
);

register_type!(
    MasterClock;
    master_clock_source: MasterClockSource => 0,
    processor_clock_prescaler: ProcessorClockPrescaler => 4,
    divide_plla_by_2: bool => 12,
    divide_upll_by_2: bool => 13
);

register_field_enum!(
    UsbInputClock, 0b1;
    Plla = 0,
    Pllb = 1
);

register_type!(
    UsbClock;
    usb_input_clock: UsbInputClock => 0,
    usb_divider: U4 => 8
);

register_field_enum!(
    ProgrammableClockSource, 0b111;
    Slow = 0,
    Main = 1,
    Plla = 2,
    Upll = 3,
    Master = 4
);

register_field_enum!(
    ProgrammableClockPrescaler, 0b111;
    Normal = 0,
    Div2 = 1,
    Div4 = 2,
    Div8 = 3,
    Div16 = 4,
    Div32 = 5,
    Div64 = 6
);

register_type!(
    ProgrammableClock;
    source: ProgrammableClockSource => 0,
    prescaler: ProgrammableClockPrescaler => 4
);

register_type!(
    Interrupt;
    crystal_oscillator_status: bool => 0,
    plla_lock: bool => 1,
    master_clock_ready: bool => 3,
    utmi_pll_lock: bool => 6,
    programmable_clock_0_ready: bool => 8,
    programmable_clock_1_ready: bool => 9,
    programmable_clock_2_ready: bool => 10,
    main_oscillator_selection_status: bool => 16,
    on_chip_rc_status: bool => 17,
    clock_failure_detector_event: bool => 18
);

register_field_enum!(
    SlowClockOscillator, 0b1;
    InternalRc = 0,
    External32kHz = 1
);

register_type!(
    Status;
    crystal_oscillator_status: bool => 0,
    plla_lock: bool => 1,
    master_clock_ready: bool => 3,
    utmi_pll_lock: bool => 6,
    slow_clock_oscillator: SlowClockOscillator => 7,
    programmable_clock_0_ready: bool => 8,
    programmable_clock_1_ready: bool => 9,
    programmable_clock_2_ready: bool => 10,
    main_oscillator_selection_status: bool => 16,
    on_chip_rc_status: bool => 17,
    clock_failure_detector_event: bool => 18,
    clock_failure_detector_status: bool => 19,
    clock_failure_detector_fault_output_status: bool => 20
);

register_field_enum!(
    LowPowerMode, 0b1;
    WfiOrWfe = 0,
    Wfe = 1
);

register_type!(
    FastStartupMode;
    fast_startup_input_0: bool => 0,
    fast_startup_input_1: bool => 1,
    fast_startup_input_2: bool => 2,
    fast_startup_input_3: bool => 3,
    fast_startup_input_4: bool => 4,
    fast_startup_input_5: bool => 5,
    fast_startup_input_6: bool => 6,
    fast_startup_input_7: bool => 7,
    fast_startup_input_8: bool => 8,
    fast_startup_input_9: bool => 9,
    fast_startup_input_10: bool => 10,
    fast_startup_input_11: bool => 11,
    fast_startup_input_12: bool => 12,
    fast_startup_input_13: bool => 13,
    fast_startup_input_14: bool => 14,
    fast_startup_input_15: bool => 15,
    rtt_alarm: bool => 16,
    rtc_alarm: bool => 17,
    usb_alarm: bool => 18,
    low_power_mode: LowPowerMode => 20
);

register_type!(
    FastStartupPolarity;
    input_0: bool => 0,
    input_1: bool => 1,
    input_2: bool => 2,
    input_3: bool => 3,
    input_4: bool => 4,
    input_5: bool => 5,
    input_6: bool => 6,
    input_7: bool => 7,
    input_8: bool => 8,
    input_9: bool => 9,
    input_10: bool => 10,
    input_11: bool => 11,
    input_12: bool => 12,
    input_13: bool => 13,
    input_14: bool => 14,
    input_15: bool => 15
);

register_type!(
    WriteProtectMode;
    enable: bool => 0,
    key: U24 => 8
);

register_type!(
    WriteProtectStatus;
    violation: bool => 0,
    violation_source: u16 => 0
);

register_field_enum!(
    Command, 0b1;
    Read = 0,
    Write = 1
);

register_field_enum!(
    Divisor, 0b11;
    Div1 = 0,
    Div2 = 1,
    Div4 = 2
);

register_type!(
    PeripheralControl;
    peripheral_id: U6 => 0,
    command: Command => 12,
    divisor: Divisor => 16,
    corresponding_clock: bool => 28

);

#[repr(C)]
struct RegisterBlock {
    pub system_clock: Srs<SystemClock>,
    _r0: Reserved<Byte4>,
    pub peripheral_clock_0: Srs<PeripheralClock0>,
    pub utmi_clock: Rw<UtmiClock>,
    pub main_oscillator: Rw<MainOscillator>,
    pub clock_frequency: Ro<ClockFrequency>,
    pub plla: Rw<Plla>,
    _r1: Reserved<Byte4>,
    pub master_clock: Rw<MasterClock>,
    _r2: Reserved<Byte4>,
    pub usb_clock: Rw<UsbClock>,
    _r3: Reserved<Byte4>,
    pub programmable_clock_0: Rw<ProgrammableClock>,
    pub programmable_clock_1: Rw<ProgrammableClock>,
    pub programmable_clock_2: Rw<ProgrammableClock>,
    _r4: Reserved<Byte16>,
    _r5: Reserved<Byte4>,
    pub interrupt: Sr<Interrupt>,
    pub status: Ro<Status>,
    pub interrupt_mask: Ro<Interrupt>,
    pub fast_startup_mode: Rw<FastStartupMode>,
    pub fast_startup_polarity: Rw<FastStartupPolarity>,
    pub fault_output_clear: Wo<bool>,
    _r6: Reserved<Byte8>,
    _r7: Reserved<Byte16>,
    _r8: Reserved<Byte16>,
    _r9: Reserved<Byte16>,
    _r10: Reserved<Byte16>,
    _r11: Reserved<Byte16>,
    _r12: Reserved<Byte16>,
    ///KEY always reads 0
    pub write_protect_mode: Rw<WriteProtectMode>,
    pub write_protect_status: Ro<WriteProtectStatus>,
    _r13: Reserved<Byte16>,
    _r14: Reserved<Byte4>,
    pub peripheral_clock_1: Srs<PeripheralClock1>,
    pub peripheral_control: Rw<PeripheralControl>,
}

struct Enabled;
struct Disabled;

pub enum ProgrammableClockOutput {
    Pc0,
    Pc1,
    Pc2,
}

pub enum PmcConfigurablePeripheral {
    SmcSdramc = 9,
    PioA = 11,
    PioB = 12,
    PioC = 13,
    PioD = 14,
    PioE = 15,
    PioF = 16,
    Usart0 = 17,
    Usart1 = 18,
    Usart2 = 19,
    Usart3 = 20,
    Hsmci = 21,
    Twi0 = 22,
    Twi1 = 23,
    Spi0 = 24,
    Spi1 = 25,
    Ssc = 26,
    Tc0 = 27,
    Tc1 = 28,
    Tc2 = 29,
    Tc3 = 30,
    Tc4 = 31,
    Tc5 = 32,
    Tc6 = 33,
    Tc7 = 34,
    Tc8 = 35,
    Pwm = 36,
    Adc = 37,
    Dacc = 38,
    Dmac = 39,
    Uotghs = 40,
    Trng = 41,
    Emac = 42,
    Can0 = 43,
    Can1 = 44,
}

const ADDRESS: u32 = 0x400E0600;

///incomplete safe impl
struct PowerManagementController<WPEN> {
    register_block: &'static mut RegisterBlock,
    _wpen: PhantomData<WPEN>,
}

impl<WPEN> PowerManagementController<WPEN> {
    pub fn new() -> PowerManagementController<WPEN> {
        PowerManagementController {
            register_block: unsafe { from_mut_ptr!(ADDRESS, RegisterBlock) },
            _wpen: PhantomData,
        }
    }
}

impl PowerManagementController<Disabled> {
    pub fn enable_usb_otg(&mut self) {
        unsafe {
            self.register_block
                .system_clock
                .set_fields(|w| w.usb_otg_clock(true));
        }
    }
    pub fn disable_usb_otg(&mut self) {
        unsafe {
            self.register_block
                .system_clock
                .set_fields(|w| w.usb_otg_clock(false));
        }
    }

    pub fn enable_programmable_clock(&mut self, id: ProgrammableClockOutput) {
        unsafe {
            self.register_block.system_clock.set_fields(|w| match id {
                ProgrammableClockOutput::Pc0 => w.programmable_clock_0(true),
                ProgrammableClockOutput::Pc1 => w.programmable_clock_1(true),
                ProgrammableClockOutput::Pc2 => w.programmable_clock_2(true),
            })
        };
    }
    pub fn disable_programmable_clock(&mut self, id: ProgrammableClockOutput) {
        unsafe {
            self.register_block.system_clock.set_fields(|w| match id {
                ProgrammableClockOutput::Pc0 => w.programmable_clock_0(false),
                ProgrammableClockOutput::Pc1 => w.programmable_clock_1(false),
                ProgrammableClockOutput::Pc2 => w.programmable_clock_2(false),
            })
        };
    }

    pub fn enable_peripheral_clock(&mut self, pid: PmcConfigurablePeripheral) {
        unsafe {
            if reref!(pid, PmcConfigurablePeripheral, u32) < 32 {
                self.register_block
                    .peripheral_clock_0
                    .set(1 << (pid as u32));
            }
        }
    }
    pub fn disable_peripheral_clock(&mut self, pid: PmcConfigurablePeripheral) {
        unsafe {
            if reref!(pid, PmcConfigurablePeripheral, u32) < 32 {
                self.register_block
                    .peripheral_clock_0
                    .reset(1 << (pid as u32));
            }
        }
    }

    pub fn into_write_protected(self) -> PowerManagementController<Enabled> {
        unsafe {
            self.register_block
                .write_protect_mode
                .set_fields(|w| w.key(0x504D43u32.into()).enable(true));
        }
        PowerManagementController {
            register_block: unsafe { from_mut_ptr!(ADDRESS, RegisterBlock) },
            _wpen: PhantomData,
        }
    }
}

impl PowerManagementController<Enabled> {
    pub fn into_writable(self) -> PowerManagementController<Disabled> {
        unsafe {
            self.register_block
                .write_protect_mode
                .set_fields(|w| w.key(0x504D43u32.into()).enable(false));
        }
        PowerManagementController {
            register_block: unsafe { from_mut_ptr!(ADDRESS, RegisterBlock) },
            _wpen: PhantomData,
        }
    }
}
