use crate::{
    common::{Modules, Reg},
    driver::Driver,
    DriverExt, SeesawDevice, SeesawError,
};

#[allow(dead_code)]
const STATUS: &Reg = &[Modules::Keypad.into_u8(), 0x00];
const EVENT: &Reg = &[Modules::Keypad.into_u8(), 0x01];
const INT_SET: &Reg = &[Modules::Keypad.into_u8(), 0x02];
const INT_CLR: &Reg = &[Modules::Keypad.into_u8(), 0x03];
const COUNT: &Reg = &[Modules::Keypad.into_u8(), 0x04];
const FIFO: &Reg = &[Modules::Keypad.into_u8(), 0x10];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u8)]
pub enum EventType {
    /// steady-state key is down
    #[default]
    IsDown = 0,
    /// steady-state key is up
    IsUp = 1,
    /// one-shot as key is released
    Released = 2,
    /// one-shot as key is pressed
    Pressed = 3,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct KeyEvent {
    pub event: EventType,
    pub key: u8,
}

pub trait KeypadModule<D: Driver>: SeesawDevice<Driver = D> {
    fn disable_interrupt(&mut self) -> Result<(), SeesawError<D::I2cError>> {
        let addr = self.addr();
        self.driver()
            .write_u8(addr, INT_CLR, 1)
            .map_err(SeesawError::I2c)
    }

    fn enable_interrupt(&mut self) -> Result<(), SeesawError<D::I2cError>> {
        let addr = self.addr();
        self.driver()
            .write_u8(addr, INT_SET, 1)
            .map_err(SeesawError::I2c)
    }

    fn watch_event(
        &mut self,
        key: u8,
        types: &[EventType],
        enable: bool,
    ) -> Result<(), SeesawError<D::I2cError>> {
        let mut v = types.iter().map(|e| 2_u8 << (*e as u8)).sum();
        if enable {
            v += 1;
        }
        let addr = self.addr();
        self.driver()
            .register_write(addr, EVENT, &[key, v])
            .map_err(SeesawError::I2c)
    }

    fn poll(&mut self) -> Result<KeyEventIter, crate::SeesawError<D::I2cError>> {
        let addr = self.addr();
        let mut kei = KeyEventIter {
            count: self
                .driver()
                .read_u8(addr, COUNT)
                .map_err(SeesawError::I2c)?,
            ..Default::default()
        };
        if kei.count == 0 {
            return Ok(kei);
        }
        if kei.count > 16 {
            kei.count = 16;
        }

        kei.buf = self
            .driver()
            .register_read(addr, FIFO)
            .map_err(SeesawError::I2c)?;
        Ok(kei)
    }
}

#[derive(Default)]
pub struct KeyEventIter {
    count: u8,
    cur: u8,
    buf: [u8; 16],
}

impl Iterator for KeyEventIter {
    type Item = KeyEvent;

    fn next(&mut self) -> Option<Self::Item> {
        if self.cur >= self.count {
            return None;
        }
        let mut rec: u8 = self.buf[self.cur as usize];
        self.cur += 1;
        let event = match rec & 3 {
            0 => EventType::IsDown,
            1 => EventType::IsUp,
            2 => EventType::Released,
            3 => EventType::Pressed,
            _ => unreachable!(),
        };
        rec >>= 2;
        let key = rec;
        Some(KeyEvent { event, key })
    }
}
