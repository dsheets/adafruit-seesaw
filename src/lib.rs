#![no_std]
#![allow(const_evaluatable_unchecked, incomplete_features)]
#![feature(array_try_map, generic_const_exprs)]
#![feature(async_fn_in_trait)]

// TODO improve the organization of the exports/visibility
mod common;
pub mod devices;
mod driver;
mod macros;
pub mod modules;
pub use common::*;
pub use devices::*;
pub use driver::*;
use embedded_hal_async::i2c;

pub mod prelude {
    pub use super::{
        devices::*,
        driver::Driver,
        modules::{adc::*, encoder::*, gpio::*, neopixel::*, status::*, timer::*},
        SeesawDevice, SeesawDeviceInit,
    };
}

pub const DEFAULT_MAX_I2C_WRITE: u8 = 32;

#[derive(Debug)]
pub enum SeesawError<P: Platform> {
    /// I2C bus error
    I2c(<<P as Platform>::I2c as i2c::ErrorType>::Error),
    /// Occurs when an invalid hardware ID is read
    InvalidHardwareId(u8),
}

pub trait SeesawDevice {
    type Platform: Platform;

    const DEFAULT_ADDR: u8;
    const HARDWARE_ID: HardwareId;
    const PRODUCT_ID: u16;

    fn addr(&self) -> u8;

    fn driver(&mut self) -> &mut Driver<Self::Platform>;

    fn new(addr: u8, driver: Driver<Self::Platform>) -> Self;

    fn new_with_default_addr(driver: Driver<Self::Platform>) -> Self;
    fn error_i2c(
        e: <<Self::Platform as Platform>::I2c as i2c::ErrorType>::Error,
    ) -> SeesawError<Self::Platform>;
    fn error_invalid_hardware_id(id: u8) -> SeesawError<Self::Platform>;
}

/// At startup, Seesaw devices typically have a unique set of initialization
/// calls to be made. e.g. for a Neokey1x4, we're need to enable the on-board
/// neopixel and also do some pin mode setting to get everything working.
/// All devices implement `DeviceInit` with a set of sensible defaults. You can
/// override the default initialization function with your own by calling
/// `Seesaw::connect_with` instead of `Seesaw::connect`.
pub trait SeesawDeviceInit: SeesawDevice
where
    Self: Sized,
{
    async fn init(self) -> Result<Self, SeesawError<Self::Platform>>;
}
