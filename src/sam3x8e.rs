use core::{marker::PhantomData, mem::replace};

pub fn fill(amount: u32) -> u32 {
    (0..amount).map(|index| 1 << index).sum()
}

pub trait RegisterAreas {
    fn offset(&self) -> u32;
    fn size(&self) -> u32;
    fn extract(&self, value: u32) -> u32 {
        value >> self.offset() & fill(self.size())
    }

    fn insert(&self, value: u32) -> u32 {
        value & fill(self.size()) << self.offset()
    }

    fn mask(&self) -> u32 {
        fill(self.size()) << self.offset()
    }
}

impl RegisterAreas for u32 {
    fn offset(&self) -> u32 {
        0
    }

    fn size(&self) -> u32 {
        32
    }

    fn extract(&self, value: u32) -> u32 {
        value
    }

    fn insert(&self, value: u32) -> u32 {
        value
    }

    fn mask(&self) -> u32 {
        0xffff_ffff
    }
}

macro_rules! areas_generate {
    ($name:ident;$($area_name:ident = $size:expr, $offset:expr),+;$($regimpl:item)*) => {
        pub enum $name {
            $($area_name),+
        }

        impl RegisterAreas for $name {
            fn size(&self) -> u32 {
                match self {
                    $($name::$area_name => $size),+
                }
            }

            fn offset(&self) -> u32 {
                match self {
                    $($name::$area_name => $offset),+
                }
            }

            $($regimpl)*
        }
    };
}

pub struct ReadRegister<T: RegisterAreas> {
    offset: u32,
    _marker: PhantomData<T>,
}

impl<T: RegisterAreas> ReadRegister<T> {
    pub fn new(offset: u32) -> ReadRegister<T> {
        ReadRegister {
            offset,
            _marker: PhantomData,
        }
    }

    pub unsafe fn read(&self) -> u32 {
        core::ptr::read_volatile(self.offset as *mut u32)
    }

    pub unsafe fn read_area(&self, area: T) -> u32 {
        area.extract(self.read())
    }
}

pub struct WriteRegister<T: RegisterAreas> {
    offset: u32,
    _marker: PhantomData<T>,
}

impl<T: RegisterAreas> WriteRegister<T> {
    pub fn new(offset: u32) -> WriteRegister<T> {
        WriteRegister {
            offset,
            _marker: PhantomData,
        }
    }

    pub unsafe fn write(&self, value: u32) {
        core::ptr::write_volatile(self.offset as *mut u32, value);
    }

    pub unsafe fn write_area(&self, area: T, value: u32) {
        core::ptr::write_volatile(self.offset as *mut u32, area.insert(value));
    }
}

pub struct ReadWriteRegister<T: RegisterAreas> {
    offset: u32,
    _marker: PhantomData<T>,
}

impl<T: RegisterAreas> ReadWriteRegister<T> {
    pub fn new(offset: u32) -> ReadWriteRegister<T> {
        ReadWriteRegister {
            offset,
            _marker: PhantomData,
        }
    }
    pub unsafe fn read(&self) -> u32 {
        core::ptr::read_volatile(self.offset as *mut u32)
    }

    pub unsafe fn read_area(&self, area: T) -> u32 {
        area.extract(self.read())
    }

    pub unsafe fn write(&self, value: u32) {
        core::ptr::write_volatile(self.offset as *mut u32, value);
    }

    pub unsafe fn write_area(&self, area: T, value: u32) {
        self.write(self.read() & !(area.mask()) | area.insert(value));
    }
}

pub struct SetResetStatusRegister<T: RegisterAreas> {
    pub set: WriteRegister<T>,
    pub reset: WriteRegister<T>,
    pub status: ReadRegister<T>,
}

impl<T: RegisterAreas> SetResetStatusRegister<T> {
    fn new(offset: u32) -> SetResetStatusRegister<T> {
        SetResetStatusRegister {
            set: WriteRegister::new(offset),
            reset: WriteRegister::new(offset + 4),
            status: ReadRegister::new(offset + 8),
        }
    }

    pub unsafe fn read(&self) -> u32 {
        self.status.read()
    }

    pub unsafe fn read_area(&self, area: T) -> u32 {
        self.status.read_area(area)
    }

    pub unsafe fn set_bits(&self, bits: u32) {
        self.set.write(bits);
    }

    pub unsafe fn set_area(&self, area: T, value: u32) {
        self.set.write(area.insert(value));
    }

    pub unsafe fn reset_bits(&self, bits: u32) {
        self.reset.write(bits);
    }

    pub unsafe fn reset_area(&self, area: T, value: u32) {
        self.reset.write(area.insert(value));
    }

    pub unsafe fn write(&self, bits: u32) {
        self.set.write(bits);
        self.reset.write(!bits);
    }

    pub unsafe fn write_area(&self, area: T, value: u32) {
        self.set.write(area.insert(value));
        self.reset.write((!area.insert(value)) & area.mask());
    }
}

pub mod rtt {
    use super::{ReadRegister, ReadWriteRegister, RegisterAreas};

    areas_generate!(ModeAreas;
        Prescaler = 16, 0,
        AlarmInterrupt = 1, 16,
        IncrementInterrupt = 1, 17,
        TimerRestart = 1, 18;
    );

    areas_generate!(StatusAreas;
        AlarmStatus = 1, 0,
        Increment = 1, 1;
    );

    pub struct Rtt {
        pub mode: ReadWriteRegister<ModeAreas>,
        pub alarm: ReadWriteRegister<u32>,
        pub value: ReadRegister<u32>,
        pub status: ReadRegister<StatusAreas>,
    }

    impl Rtt {
        pub fn new() -> Rtt {
            const ADDRESS: u32 = 0x400E1A30;

            Rtt {
                mode: ReadWriteRegister::new(ADDRESS),
                alarm: ReadWriteRegister::new(ADDRESS + 0x4),
                value: ReadRegister::new(ADDRESS + 0x8),
                status: ReadRegister::new(0xC),
            }
        }
    }

    pub fn rtt() -> Rtt {
        Rtt::new()
    }
}

pub mod pmc {
    use super::{ReadRegister, ReadWriteRegister, RegisterAreas, SetResetStatusRegister};

    areas_generate!(
        SystemClockAreas;
        EnableUsbOtgClock = 1,
        5,
        ProgrammableClock1 = 1,
        8,
        ProgrammableClock2 = 1,
        9,
        ProgrammableClock3 = 1,
        10;
        fn extract(&self, value: u32) -> u32 {
            value >> self.offset() & 1
        }

        fn insert(&self, value: u32) -> u32 {
            value & 1 << self.offset()
        }

        fn mask(&self) -> u32 {
            1 << self.offset()
        }
    );

    areas_generate!(
        PeripheralClockAreas;
        PeripheralClock2 = 1,
        2,
        PeripheralClock3 = 1,
        3,
        PeripheralClock4 = 1,
        4,
        PeripheralClock5 = 1,
        5,
        PeripheralClock6 = 1,
        6,
        PeripheralClock7 = 1,
        7,
        PeripheralClock8 = 1,
        8,
        PeripheralClock9 = 1,
        9,
        PeripheralClock10 = 1,
        10,
        PeripheralClock11 = 1,
        11,
        PeripheralClock12 = 1,
        12,
        PeripheralClock13 = 1,
        13,
        PeripheralClock14 = 1,
        14,
        PeripheralClock15 = 1,
        15,
        PeripheralClock16 = 1,
        16,
        PeripheralClock17 = 1,
        17,
        PeripheralClock18 = 1,
        18,
        PeripheralClock19 = 1,
        19,
        PeripheralClock20 = 1,
        20,
        PeripheralClock21 = 1,
        21,
        PeripheralClock22 = 1,
        22,
        PeripheralClock23 = 1,
        23,
        PeripheralClock24 = 1,
        24,
        PeripheralClock25 = 1,
        25,
        PeripheralClock26 = 1,
        26,
        PeripheralClock27 = 1,
        27,
        PeripheralClock28 = 1,
        28,
        PeripheralClock29 = 1,
        29,
        PeripheralClock30 = 1,
        30,
        PeripheralClock31 = 1,
        31;
        fn extract(&self, value: u32) -> u32 {
            value >> self.offset() & 1
        }

        fn insert(&self, value: u32) -> u32 {
            value & 1 << self.offset()
        }

        fn mask(&self) -> u32 {
            1 << self.offset()
        }
    );

    areas_generate!(UtmiClockAreas; UtmiPll = 1, 16, UtmiPllStartUpTime = 4, 20;);

    areas_generate!(
        MainOscillatorAreas;
        MainCrystalOscillator = 1,
        0,
        MainCrystalOscillatorBypass = 1,
        1,
        MainOnChipRcOscillator = 1,
        3,
        MainOnChipRcOscillatorFrequency = 3,
        4,
        MainCrystalOscillatorStartUpTime = 8,
        8,
        Key = 8,
        16,
        MainOscillatorSelection = 1,
        24,
        ClockFailureDetector = 1,
        25;
    );

    areas_generate!(
        ClockFrequencyAreas;
        ClockFrequency = 16,
        0,
        ClockReady = 1,
        16;
    );

    areas_generate!(
        PllaAreas;
        Divider = 8,
        0,
        PllaCounter = 6,
        8,
        PllaMultiplier = 11,
        16,
        One = 29,
        1;
    );

    pub struct Pmc {
        pub system_clock: SetResetStatusRegister<SystemClockAreas>,
        pub peripheral_clock_0: SetResetStatusRegister<PeripheralClockAreas>,
        pub utmi_clock_configuration: ReadWriteRegister<UtmiClockAreas>,
        pub main_oscillator: ReadWriteRegister<MainOscillatorAreas>,
        pub main_clock_frequency: ReadRegister<ClockFrequencyAreas>,
        pub plla: ReadWriteRegister<PllaAreas>,
        // !UNFINISHED
    }

    impl Pmc {
        pub fn new() -> Pmc {
            const PMC_ADDRESS: u32 = 0x400E0600;
            Pmc {
                system_clock: SetResetStatusRegister::new(PMC_ADDRESS),
                peripheral_clock_0: SetResetStatusRegister::new(PMC_ADDRESS + 0x10),
                utmi_clock_configuration: ReadWriteRegister::new(PMC_ADDRESS + 0x1C),
                main_oscillator: ReadWriteRegister::new(PMC_ADDRESS + 0x20),
                main_clock_frequency: ReadRegister::new(PMC_ADDRESS + 0x24),
                plla: ReadWriteRegister::new(PMC_ADDRESS + 0x28),
            }
        }
    }

    pub fn pmc() -> Pmc {
        Pmc::new()
    }
}

pub mod pio {
    use super::{
        ReadRegister, ReadWriteRegister, RegisterAreas, SetResetStatusRegister, WriteRegister,
    };

    areas_generate!(
        IoLine;
        L0 = 1,
        0,
        L1 = 1,
        1,
        L2 = 1,
        2,
        L3 = 1,
        3,
        L4 = 1,
        4,
        L5 = 1,
        5,
        L6 = 1,
        6,
        L7 = 1,
        7,
        L8 = 1,
        8,
        L9 = 1,
        9,
        L10 = 1,
        10,
        L11 = 1,
        11,
        L12 = 1,
        12,
        L13 = 1,
        13,
        L14 = 1,
        14,
        L15 = 1,
        15,
        L16 = 1,
        16,
        L17 = 1,
        17,
        L18 = 1,
        18,
        L19 = 1,
        19,
        L20 = 1,
        20,
        L21 = 1,
        21,
        L22 = 1,
        22,
        L23 = 1,
        23,
        L24 = 1,
        24,
        L25 = 1,
        25,
        L26 = 1,
        26,
        L27 = 1,
        27,
        L28 = 1,
        28,
        L29 = 1,
        29,
        L30 = 1,
        30,
        L31 = 1,
        31;
        fn extract(&self, value: u32) -> u32 {
            value >> self.offset() & 1
        }

        fn insert(&self, value: u32) -> u32 {
            value & 1 << self.offset()
        }

        fn mask(&self) -> u32 {
            1 << self.offset()
        }
    );

    pub struct InterruptRegister {
        pub enable: WriteRegister<IoLine>,
        pub disable: WriteRegister<IoLine>,
        pub mask: ReadRegister<IoLine>,
        pub status: ReadRegister<IoLine>,
    }

    impl InterruptRegister {
        pub unsafe fn set_enabled(&self, value: u32) {
            self.enable.write(value);
            self.disable.write(!value);
        }

        pub unsafe fn get_enabled(&self) -> u32 {
            self.mask.read()
        }

        pub unsafe fn get_enabled_pin(&self, pin: IoLine) -> bool {
            self.mask.read_area(pin) == 1
        }

        pub unsafe fn get_changed(&self) -> u32 {
            self.status.read()
        }

        pub unsafe fn get_changed_pin(&self, pin: IoLine) -> bool {
            self.mask.read_area(pin) == 1
        }

        pub unsafe fn enable_pins(&self, pins: u32) {
            self.enable.write(pins);
        }

        pub unsafe fn enable_pin(&self, pin: IoLine) {
            self.enable.write(pin.insert(1));
        }

        pub unsafe fn disable_pins(&self, pins: u32) {
            self.disable.write(pins);
        }

        pub unsafe fn disable_pin(&self, pin: IoLine) {
            self.enable.write(pin.insert(1));
        }
    }

    impl InterruptRegister {
        fn new(offset: u32) -> InterruptRegister {
            InterruptRegister {
                enable: WriteRegister::new(offset),
                disable: WriteRegister::new(offset + 4),
                mask: ReadRegister::new(offset + 8),
                status: ReadRegister::new(offset + 0xC),
            }
        }
    }

    pub enum InputFilterMode {
        Glitch = 0,
        Debounce = 1,
    }

    impl From<bool> for InputFilterMode {
        fn from(value: bool) -> Self {
            if value {
                InputFilterMode::Debounce
            } else {
                InputFilterMode::Glitch
            }
        }
    }

    pub struct InputFilter {
        active: SetResetStatusRegister<IoLine>,
        mode: SetResetStatusRegister<IoLine>,
        slow_clock_divider: ReadWriteRegister<u32>,
    }

    impl InputFilter {
        fn new(pio_controller_offset: u32) -> InputFilter {
            InputFilter {
                active: SetResetStatusRegister::new(pio_controller_offset + 0x20),
                mode: SetResetStatusRegister::new(pio_controller_offset + 0x80),
                slow_clock_divider: ReadWriteRegister::new(pio_controller_offset + 0x8C),
            }
        }

        pub unsafe fn set_enabled(&self, value: u32) {
            self.active.write(value);
        }

        pub unsafe fn get_enabled(&self) -> u32 {
            self.active.read()
        }

        pub unsafe fn get_enabled_pin(&self, pin: IoLine) -> bool {
            self.active.read_area(pin) == 1
        }

        pub unsafe fn enable_pins(&self, pins: u32) {
            self.active.set.write(pins);
        }

        pub unsafe fn enable_pin(&self, pin: IoLine) {
            self.active.set.write(pin.insert(1));
        }

        pub unsafe fn disable_pins(&self, pins: u32) {
            self.active.reset.write(pins);
        }

        pub unsafe fn disable_pin(&self, pin: IoLine) {
            self.active.reset.write(pin.insert(1));
        }

        pub unsafe fn set_pin_mode(&self, pin: IoLine, mode: InputFilterMode) {
            self.mode.write_area(pin, mode as u32)
        }

        pub unsafe fn get_pin_mode(&self, pin: IoLine) -> InputFilterMode {
            (self.mode.read_area(pin) == 1).into()
        }

        pub unsafe fn set_slow_clock_divider(&self, divider: u32) {
            self.slow_clock_divider.write(divider);
        }

        pub unsafe fn get_slow_clock_divider(&self) -> u32 {
            self.slow_clock_divider.read()
        }
    }

    pub struct Pio {
        pub pio: SetResetStatusRegister<IoLine>,
        pub output: SetResetStatusRegister<IoLine>,
        pub input_filter: InputFilter,
        pub output_data: SetResetStatusRegister<IoLine>,
        pub pin_data: ReadRegister<IoLine>,
        pub interrupt: InterruptRegister,
        pub multi_driver: SetResetStatusRegister<IoLine>,
        pub pull_up_disable: SetResetStatusRegister<IoLine>,
        pub peripheral_ab_select: ReadWriteRegister<IoLine>,
        pub output_write: SetResetStatusRegister<IoLine>,
        // !ADDITIONAL INTERRUPT MODES not implemented
        // !WRITE PROTECT MODE not implemented
    }

    impl Pio {
        fn new(offset: u32) -> Pio {
            Pio {
                pio: SetResetStatusRegister::new(offset),
                output: SetResetStatusRegister::new(offset + 0x10),
                input_filter: InputFilter::new(offset),
                output_data: SetResetStatusRegister::new(offset + 0x30),
                pin_data: ReadRegister::new(offset + 0x3C),
                interrupt: InterruptRegister::new(offset + 0x40),
                multi_driver: SetResetStatusRegister::new(offset + 0x50),
                pull_up_disable: SetResetStatusRegister::new(offset + 0x60),
                peripheral_ab_select: ReadWriteRegister::new(offset + 0x70),
                output_write: SetResetStatusRegister::new(offset + 0xA0),
            }
        }
    }

    pub fn pio_a() -> Pio {
        Pio::new(0x400E_0E00)
    }
    pub fn pio_b() -> Pio {
        Pio::new(0x400E_1000)
    }
    pub fn pio_c() -> Pio {
        Pio::new(0x400E_1200)
    }
    pub fn pio_d() -> Pio {
        Pio::new(0x400E_1400)
    }
}

struct Peripherals {
    rtt: Option<rtt::Rtt>,
    pmc: Option<pmc::Pmc>,
    pio_a: Option<pio::Pio>,
    pio_b: Option<pio::Pio>,
    pio_c: Option<pio::Pio>,
    pio_d: Option<pio::Pio>,
}

impl Peripherals {
    fn rtt(&mut self) -> rtt::Rtt {
        let s = replace(&mut self.rtt, None);
        s.unwrap()
    }

    fn pmc(&mut self) -> pmc::Pmc {
        let s = replace(&mut self.pmc, None);
        s.unwrap()
    }

    fn pio_a(&mut self) -> pio::Pio {
        let s = replace(&mut self.pio_a, None);
        s.unwrap()
    }

    fn pio_b(&mut self) -> pio::Pio {
        let s = replace(&mut self.pio_b, None);
        s.unwrap()
    }

    fn pio_c(&mut self) -> pio::Pio {
        let s = replace(&mut self.pio_c, None);
        s.unwrap()
    }

    fn pio_d(&mut self) -> pio::Pio {
        let s = replace(&mut self.pio_d, None);
        s.unwrap()
    }
}

static mut PERIPHERALS: Peripherals = Peripherals {
    rtt: Some(rtt::rtt()),
    pmc: Some(pmc::pmc()),
    pio_a: Some(pio::pio_a()),
    pio_b: Some(pio::pio_a()),
    pio_c: Some(pio::pio_a()),
    pio_d: Some(pio::pio_a()),
};
