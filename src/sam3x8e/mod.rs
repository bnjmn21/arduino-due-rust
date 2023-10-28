use core::{marker::PhantomData, ptr::read_volatile, ptr::write_volatile};
mod rtt;

trait RegisterType {}

struct RegisterReader<T: RegisterType> {
    value: u32,
    _type: PhantomData<T>,
}

impl<T: RegisterType> RegisterReader<T> {
    fn new() -> RegisterReader<T> {
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
    fn new(value: u32) -> RegisterWriter<T> {
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

struct RO<T: RegisterType> {
    pub value: u32,
    _type: PhantomData<T>,
}

impl<T: RegisterType> RO<T> {
    pub unsafe fn read(&self) -> RegisterReader<T> {
        read_volatile(&self.value).into()
    }
}

struct WO<T: RegisterType> {
    pub value: u32,
    _type: PhantomData<T>,
}

impl<T: RegisterType> WO<T> {
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

struct RW<T: RegisterType> {
    pub value: u32,
    _type: PhantomData<T>,
}

impl<T: RegisterType> RW<T> {
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
    };
}

use register_type;

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
