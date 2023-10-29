use core::{marker::PhantomData, ptr::read_volatile, ptr::write_volatile};
mod pmc;
mod rtt;

trait RegisterType {}

struct RegisterReader<T: RegisterType> {
    value: u32,
    _type: PhantomData<T>,
}

impl<T: RegisterType> RegisterReader<T> {
    pub fn new() -> RegisterReader<T> {
        RegisterReader {
            value: 0,
            _type: PhantomData,
        }
    }
}

impl<T: RegisterType> From<u32> for RegisterReader<T> {
    fn from(value: u32) -> Self {
        RegisterReader {
            value,
            _type: PhantomData,
        }
    }
}

struct RegisterWriter<T: RegisterType> {
    value: u32,
    _type: PhantomData<T>,
}

impl<T: RegisterType> RegisterWriter<T> {
    pub fn new(value: u32) -> RegisterWriter<T> {
        RegisterWriter {
            value,
            _type: PhantomData,
        }
    }
}

impl<T: RegisterType> From<RegisterWriter<T>> for u32 {
    fn from(value: RegisterWriter<T>) -> Self {
        value.value
    }
}

struct MaskedRegisterWriter<T: RegisterType> {
    value: u32,
    mask: u32,
    _type: PhantomData<T>,
}

impl<T: RegisterType> MaskedRegisterWriter<T> {
    pub fn new() -> MaskedRegisterWriter<T> {
        MaskedRegisterWriter {
            value: 0,
            mask: 0,
            _type: PhantomData,
        }
    }
}

impl RegisterType for bool {}

impl RegisterReader<bool> {
    pub fn get(self) -> bool {
        self.value == 1
    }
}

impl RegisterWriter<bool> {
    pub fn set(self, value: bool) -> RegisterWriter<bool> {
        RegisterWriter {
            value: value as u32,
            _type: PhantomData,
        }
    }
}

impl MaskedRegisterWriter<bool> {
    pub fn set(self, value: bool) -> MaskedRegisterWriter<bool> {
        MaskedRegisterWriter {
            value: value as u32,
            mask: 1,
            _type: PhantomData,
        }
    }
}

struct Ro<T: RegisterType> {
    pub value: u32,
    _type: PhantomData<T>,
}

impl<T: RegisterType> Ro<T> {
    pub unsafe fn read(&self) -> RegisterReader<T> {
        read_volatile(&self.value).into()
    }
}

struct Wo<T: RegisterType> {
    pub value: u32,
    _type: PhantomData<T>,
}

impl<T: RegisterType> Wo<T> {
    pub unsafe fn write(&mut self, value: u32) {
        write_volatile(&mut self.value, value);
    }

    pub unsafe fn write_with_zero<F>(&mut self, function: F)
    where
        F: FnOnce(RegisterWriter<T>) -> RegisterWriter<T>,
    {
        self.write(function(RegisterWriter::new(0)).into());
    }
}

struct Rw<T: RegisterType> {
    pub value: u32,
    _type: PhantomData<T>,
}

impl<T: RegisterType> Rw<T> {
    pub unsafe fn write(&mut self, value: u32) {
        write_volatile(&mut self.value, value);
    }

    pub unsafe fn write_with_zero<F>(&mut self, function: F)
    where
        F: FnOnce(RegisterWriter<T>) -> RegisterWriter<T>,
    {
        self.write(function(RegisterWriter::new(0)).into());
    }

    pub unsafe fn read(&self) -> RegisterReader<T> {
        read_volatile(&self.value).into()
    }

    pub unsafe fn set_fields<F>(&mut self, function: F)
    where
        F: FnOnce(RegisterWriter<T>) -> RegisterWriter<T>,
    {
        self.write(function(RegisterWriter::new(self.read().value)).into());
    }
}

struct Srs<T: RegisterType> {
    pub set: Wo<T>,
    pub reset: Wo<T>,
    pub status: Ro<T>,
}

impl<T: RegisterType> Srs<T> {
    pub unsafe fn set(&mut self, value: u32) {
        self.set.write(value);
    }

    pub unsafe fn reset(&mut self, value: u32) {
        self.reset.write(value);
    }

    pub unsafe fn set_fields<F>(&mut self, function: F)
    where
        F: FnOnce(MaskedRegisterWriter<T>) -> MaskedRegisterWriter<T>,
    {
        let value_and_mask = function(MaskedRegisterWriter::new());
        self.set.write(value_and_mask.value);
        self.reset
            .write((!value_and_mask.value) & value_and_mask.mask);
    }

    pub unsafe fn read(&self) -> RegisterReader<T> {
        self.status.read()
    }
}

struct Sr<T: RegisterType> {
    pub set: Wo<T>,
    pub reset: Wo<T>,
}

impl<T: RegisterType> Sr<T> {
    pub unsafe fn set(&mut self, value: u32) {
        self.set.write(value);
    }

    pub unsafe fn reset(&mut self, value: u32) {
        self.reset.write(value);
    }

    pub unsafe fn set_fields<F>(&mut self, function: F)
    where
        F: FnOnce(MaskedRegisterWriter<T>) -> MaskedRegisterWriter<T>,
    {
        let value_and_mask = function(MaskedRegisterWriter::new());
        self.set.write(value_and_mask.value);
        self.reset
            .write((!value_and_mask.value) & value_and_mask.mask);
    }
}

impl RegisterType for u32 {}

trait RegisterField {
    const MASK: u32;
    fn into_u32(self) -> u32;
    fn from_u32(value: u32) -> Self;
}

impl RegisterField for bool {
    const MASK: u32 = 1;
    fn into_u32(self) -> u32 {
        self as u32
    }
    fn from_u32(value: u32) -> Self {
        value == 1
    }
}

impl RegisterField for u16 {
    const MASK: u32 = u16::MAX as u32;
    fn into_u32(self) -> u32 {
        self as u32
    }
    fn from_u32(value: u32) -> Self {
        value as u16
    }
}

impl RegisterField for u8 {
    const MASK: u32 = u8::MAX as u32;
    fn into_u32(self) -> u32 {
        self as u32
    }
    fn from_u32(value: u32) -> Self {
        value as u8
    }
}

mod special_ints {
    use super::RegisterField;
    use core::convert::TryInto;
    pub struct U4 {
        value: u8,
    }

    impl RegisterField for U4 {
        const MASK: u32 = 0xf;
        fn into_u32(self) -> u32 {
            self.value as u32
        }

        fn from_u32(value: u32) -> Self {
            Self {
                value: value.try_into().unwrap(),
            }
        }
    }

    impl From<u32> for U4 {
        fn from(value: u32) -> Self {
            Self {
                value: unsafe { value.try_into().unwrap_unchecked() },
            }
        }
    }

    impl From<U4> for u32 {
        fn from(value: U4) -> Self {
            value.value as u32
        }
    }

    pub struct U6 {
        value: u8,
    }

    impl RegisterField for U6 {
        const MASK: u32 = 0b11_1111;
        fn into_u32(self) -> u32 {
            self.value as u32
        }

        fn from_u32(value: u32) -> Self {
            Self {
                value: value.try_into().unwrap(),
            }
        }
    }

    impl From<u32> for U6 {
        fn from(value: u32) -> Self {
            Self {
                value: unsafe { value.try_into().unwrap_unchecked() },
            }
        }
    }

    impl From<U6> for u32 {
        fn from(value: U6) -> Self {
            value.value as u32
        }
    }

    pub struct U11 {
        value: u16,
    }

    impl RegisterField for U11 {
        const MASK: u32 = 0b111_1111_1111;
        fn into_u32(self) -> u32 {
            self.value as u32
        }

        fn from_u32(value: u32) -> Self {
            Self {
                value: value.try_into().unwrap(),
            }
        }
    }

    impl From<u32> for U11 {
        fn from(value: u32) -> Self {
            Self {
                value: unsafe { value.try_into().unwrap_unchecked() },
            }
        }
    }

    impl From<U11> for u32 {
        fn from(value: U11) -> Self {
            value.value as u32
        }
    }

    pub struct U24 {
        value: u32,
    }

    impl RegisterField for U24 {
        const MASK: u32 = 0b1111_1111_1111_1111_1111_1111;
        fn into_u32(self) -> u32 {
            self.value
        }

        fn from_u32(value: u32) -> Self {
            Self { value }
        }
    }

    impl From<u32> for U24 {
        fn from(value: u32) -> Self {
            Self { value }
        }
    }

    impl From<U24> for u32 {
        fn from(value: U24) -> Self {
            value.value
        }
    }
}

trait ReservedConfig {
    fn new() -> Self;
}

struct Byte4 {
    value: u32,
}

impl ReservedConfig for Byte4 {
    fn new() -> Self {
        Byte4 { value: 0 }
    }
}

struct Byte8 {
    value0: u32,
    value1: u32,
}

impl ReservedConfig for Byte8 {
    fn new() -> Self {
        Byte8 {
            value0: 0,
            value1: 0,
        }
    }
}

struct Byte16 {
    value0: u32,
    value1: u32,
    value2: u32,
    value3: u32,
}

impl ReservedConfig for Byte16 {
    fn new() -> Self {
        Byte16 {
            value0: 0,
            value1: 0,
            value2: 0,
            value3: 0,
        }
    }
}

struct Reserved<T: ReservedConfig> {
    _reserved: T,
}

impl<T: ReservedConfig> Reserved<T> {
    fn new() -> Reserved<T> {
        Reserved {
            _reserved: T::new(),
        }
    }
}

macro_rules! register_type {
    (
        $name:ident;
        $($field:ident: $type:ty => $index:expr),*
    ) => {
        pub struct $name;
        impl RegisterType for $name {}
        impl RegisterReader<$name> {
            $(
                pub fn $field(&self) -> $type {
                    <$type>::from_u32(self.value >> $index & <$type>::MASK)
                }
            )*
        }
        impl RegisterWriter<$name> {
            $(
                pub fn $field(&self, value: $type) -> RegisterWriter<$name> {
                    RegisterWriter {
                        value: self.value & (<$type>::MASK << $index) | (value.into_u32() << $index),
                        _type: self._type,
                    }
                }
            )*
        }
        impl MaskedRegisterWriter<$name> {
            $(
                pub fn $field(&self, value: $type) -> MaskedRegisterWriter<$name> {
                    MaskedRegisterWriter {
                        value: self.value & (<$type>::MASK << $index) | (value.into_u32() << $index),
                        mask: self.mask | (<$type>::MASK << $index),
                        _type: self._type,
                    }
                }
            )*
        }
    };
}

macro_rules! from_mut_ptr {
    ($ptr:expr, $type:ty) => {
        &mut *($ptr as *mut $type)
    };
}

macro_rules! reref {
    ($var:expr, $src:ty, $target:ty) => {
        *((&$var) as *const $src as *const $target)
    };
}

macro_rules! register_field_enum {
    (
        $name:ident, $mask:expr;
        $($variation:ident = $index:expr),+
    ) => {
        #[derive(PartialEq)]
        pub enum $name {
            $($variation = $index),+
        }

        impl RegisterField for $name {
            const MASK: u32 = $mask;
            fn from_u32(value: u32) -> Self {
                match value {
                    $($index => $name::$variation,)+
                    _ => panic!()
                }
            }
            fn into_u32(self) -> u32 {
                self as u32
            }
        }
    };
}

use from_mut_ptr;
use register_field_enum;
use register_type;
use reref;

pub struct Peripherals {
    rtt: Option<rtt::RealTimeTimer>,
}

impl Peripherals {
    fn rtt(&mut self) -> rtt::RealTimeTimer {
        let p = self.rtt.take();
        p.unwrap()
    }
}

pub fn peripherals() -> Peripherals {
    Peripherals {
        rtt: Some(rtt::RealTimeTimer::new()),
    }
}
