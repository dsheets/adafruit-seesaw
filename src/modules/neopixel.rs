use core::mem;

use crate::{common::Modules, driver::Driver, DriverExt, Reg, RegValue, SeesawDevice, SeesawError};

/// WO - 8 bits
/// This register sets the pin number (PORTA) that is used for the NeoPixel
/// output.
const SET_PIN: &Reg = &[Modules::Neopixel.into_u8(), 0x01];
/// WO - 8 bits
/// The protocol speed. (see `NeopixelSpeed`) Default is 800khz.
const SET_SPEED: &Reg = &[Modules::Neopixel.into_u8(), 0x02];
/// WO - 16 bits
/// The number of bytes currently used for the pixel array. This is
/// dependent on when the pixels you are using are RGB or RGBW.
const SET_LEN: &Reg = &[Modules::Neopixel.into_u8(), 0x03];
/// WO - 256 bits (32 bytes)
/// The data buffer. The first 2 bytes are the start address, and the data
/// to write follows. Data should be written in blocks of maximum size 30
/// bytes at a time.
const SET_BUF: &Reg = &[Modules::Neopixel.into_u8(), 0x04];
/// W0 - Zero bits
/// Sending the SHOW command will cause the output to update. There's no
/// arguments/data after the command.
const SHOW: &Reg = &[Modules::Neopixel.into_u8(), 0x05];

pub trait ColorVector {
    const DIMS: usize;
}

pub type ColorRGB = (u8, u8, u8);

impl ColorVector for ColorRGB {
    const DIMS: usize = 3;
}

pub type ColorRGBW = (u8, u8, u8, u8);

impl ColorVector for ColorRGBW {
    const DIMS: usize = 4;
}

/// See <https://github.com/adafruit/Adafruit_NeoPixel/blob/fe882b84951bed066764f9350e600a2ec2aa5a9e/Adafruit_NeoPixel.h#L64>
pub trait ColorLayout {
    type Vector: ColorVector;
    fn blit(c: &Self::Vector, buf: &mut [u8]);
}

pub struct RGB;
impl ColorLayout for RGB {
    type Vector = ColorRGB;

    #[inline]
    fn blit((r, g, b): &Self::Vector, buf: &mut [u8]) {
        buf[0] = *r;
        buf[1] = *g;
        buf[2] = *b;
    }
}

pub struct GRB;
impl ColorLayout for GRB {
    type Vector = ColorRGB;

    #[inline]
    fn blit((r, g, b): &Self::Vector, buf: &mut [u8]) {
        buf[0] = *g;
        buf[1] = *r;
        buf[2] = *b;
    }
}

pub struct RGBW;
impl ColorLayout for RGBW {
    type Vector = ColorRGBW;

    #[inline]
    fn blit((r, g, b, w): &Self::Vector, buf: &mut [u8]) {
        buf[0] = *r;
        buf[1] = *g;
        buf[2] = *b;
        buf[3] = *w;
    }
}

pub trait NeopixelModule<D: Driver, C: ColorLayout>: SeesawDevice<Driver = D> {
    const PIN: u8;

    /// The number of neopixels on the device
    const N_LEDS: u16 = 1;

    fn enable_neopixel(&mut self) -> Result<(), SeesawError<D::I2cError>> {
        let addr = self.addr();

        self.driver()
            .write_u8(addr, SET_PIN, Self::PIN)
            .and_then(|_| {
                self.driver().delay_us(10_000);
                self.driver()
                    .write_u16(addr, SET_LEN, C::Vector::DIMS as u16 * Self::N_LEDS)
            })
            .map(|_| self.driver().delay_us(10_000))
            .map_err(SeesawError::I2c)
    }

    fn set_neopixel_speed(&mut self, speed: NeopixelSpeed) -> Result<(), SeesawError<D::I2cError>> {
        let addr = self.addr();

        self.driver()
            .write_u8(
                addr,
                SET_SPEED,
                match speed {
                    NeopixelSpeed::Khz400 => 0,
                    NeopixelSpeed::Khz800 => 1,
                },
            )
            .map(|_| self.driver().delay_us(10_000))
            .map_err(SeesawError::I2c)
    }

    fn set_neopixel_color(&mut self, c: C::Vector) -> Result<(), SeesawError<D::I2cError>>
    where
        [(); mem::size_of::<u16>() + C::Vector::DIMS]: Sized,
        [(); 2 + (mem::size_of::<u16>() + C::Vector::DIMS)]: Sized,
    {
        self.set_nth_neopixel_color(0, c)
    }

    fn set_nth_neopixel_color(
        &mut self,
        n: u16,
        color: C::Vector,
    ) -> Result<(), SeesawError<D::I2cError>>
    where
        [(); mem::size_of::<u16>() + C::Vector::DIMS]: Sized,
        [(); 2 + (mem::size_of::<u16>() + C::Vector::DIMS)]: Sized,
    {
        assert!(n < Self::N_LEDS);
        let offset = u16::to_be_bytes(C::Vector::DIMS as u16 * n);
        let mut regval = RegValue::<{ mem::size_of::<u16>() + C::Vector::DIMS }>::new(SET_BUF);
        regval[0..2].copy_from_slice(&offset);
        C::blit(&color, &mut regval[2..]);
        let addr = self.addr();

        self.driver()
            .register_write(addr, &regval)
            .map_err(SeesawError::I2c)
    }

    fn set_neopixel_colors(
        &mut self,
        colors: &[C::Vector; Self::N_LEDS as usize],
    ) -> Result<(), SeesawError<D::I2cError>>
    where
        [(); Self::N_LEDS as usize]: Sized,
        [(); mem::size_of::<u16>() + C::Vector::DIMS]: Sized,
        [(); 2 + (mem::size_of::<u16>() + C::Vector::DIMS)]: Sized,
    {
        let addr = self.addr();

        (0..Self::N_LEDS)
            .try_for_each(|n| {
                let offset = u16::to_be_bytes(C::Vector::DIMS as u16 * n);
                let mut regval =
                    RegValue::<{ mem::size_of::<u16>() + C::Vector::DIMS }>::new(SET_BUF);
                regval[0..2].copy_from_slice(&offset);
                let color = &colors[n as usize];
                C::blit(color, &mut regval[2..]);
                self.driver().register_write(addr, &regval)
            })
            .map_err(SeesawError::I2c)
    }

    fn sync_neopixel(&mut self) -> Result<(), SeesawError<D::I2cError>> {
        let addr = self.addr();

        self.driver()
            .register_write(addr, &RegValue::<0>::new(SHOW))
            .map(|_| self.driver().delay_us(125))
            .map_err(SeesawError::I2c)
    }
}

/// NeopixelModule: The Neopixel protocol speed
#[derive(Debug, Default)]
pub enum NeopixelSpeed {
    Khz400 = 0,
    #[default]
    Khz800 = 1,
}
