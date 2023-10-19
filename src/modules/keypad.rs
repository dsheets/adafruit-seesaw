extern crate alloc;

use core::fmt::Debug;

use alloc::vec;
use alloc::vec::Vec;
use bitflags::bitflags;
use embassy_time::Duration;
use esp_println::println;

use crate::{Modules, Reg, SeesawDevice, SeesawError};

/// WO - 16 bits
/// This register sets event subscriptions
const SET_EVENT: &Reg = &[Modules::Keypad.into_u8(), 0x01];
/// RO - 8 bits
/// The number of FIFO events
const COUNT: &Reg = &[Modules::Keypad.into_u8(), 0x04];
/// RO - variable bits
/// The button event queue
const FIFO: &Reg = &[Modules::Keypad.into_u8(), 0x10];

#[derive(Clone, Copy, Debug)]
pub enum KeypadEventKind {
    High = 0,
    Low = 1,
    Falling = 2,
    Rising = 3,
}

use KeypadEventKind::{Falling, High, Low, Rising};

bitflags! {
    #[derive(Clone, Copy)]
    pub struct KeypadEventSpec: u8 {
        const ENABLED = 0b_0000_0001;
        const HIGH    = 1 << (High as u8 + 1);
        const LOW     = 1 << (Low as u8 + 1);
        const FALLING = 1 << (Falling as u8 + 1);
        const RISING  = 1 << (Rising as u8 + 1);
    }
}

struct KeyIndex(u8);
struct KeyCode(u8);

fn key_code(key_index: KeyIndex) -> KeyCode {
    KeyCode(((key_index.0 >> 2) << 3) + (key_index.0 & 0b011))
}

fn key_index(key_code: KeyCode) -> KeyIndex {
    KeyIndex(((key_code.0 >> 3) << 2) + (key_code.0 & 0b111))
}

#[derive(Clone, Copy)]
pub struct KeypadEvent(u8);

impl KeypadEvent {
    pub fn is_kind(self, kind: KeypadEventKind) -> bool {
        kind as u8 == self.0 as u8 & 0b11
    }

    pub fn kind(self) -> KeypadEventKind {
        [High, Low, Falling, Rising][(self.0 as u8 & 0b11) as usize]
    }

    pub fn num(self) -> u8 {
        key_index(KeyCode(self.0 as u8 >> 2)).0
    }
}

impl Debug for KeypadEvent {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("KeypadEvent").field("kind", &self.kind()).field("num", &self.num()).finish()
    }
}

impl From<&u8> for KeypadEvent {
    fn from(value: &u8) -> Self {
        Self(*value)
    }
}

pub trait KeypadModule<const SIZE: usize>: SeesawDevice {
    async fn set_event(
        &mut self,
        key: usize,
        spec: KeypadEventSpec,
    ) -> Result<(), SeesawError<Self::Platform>> {
        let addr = self.addr();
        self.driver()
            .register_write(addr, SET_EVENT, &[key_code(KeyIndex(key as u8)).0, spec.bits()])
            .await
            .map_err(Self::error_i2c)
    }

    async fn set_all_events(
        &mut self,
        spec: KeypadEventSpec,
    ) -> Result<(), SeesawError<Self::Platform>> {
        for key in 0..SIZE {
            self.set_event(key, spec).await?
        }
        Ok(())
    }

    async fn set_events(
        &mut self,
        keys: [KeypadEventSpec; SIZE],
    ) -> Result<(), SeesawError<Self::Platform>> {
        for (key, spec) in keys.iter().enumerate() {
            self.set_event(key, *spec).await?
        }
        Ok(())
    }

    async fn read_keypad(&mut self) -> Result<Vec<KeypadEvent>, SeesawError<Self::Platform>> {
        let addr = self.addr();
        let fifo_len: usize = self
            .driver()
            .read_u8(addr, COUNT)
            .await
            .map_err(Self::error_i2c)? as usize;
        let mut bytes = vec![0; fifo_len];
        self.driver()
            .register_read_into(addr, FIFO, bytes.as_mut_slice())
            .await
            .map_err(Self::error_i2c)?;
        let events = bytes.iter().map(Into::into).collect();
        Ok(events)
    }
}
