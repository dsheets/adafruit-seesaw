use embassy_time::Duration;

use crate::{
    common::{Modules, Reg},
    SeesawDevice, SeesawError,
};

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

pub trait ColorType {
    const DIMS: usize;
    type Color;
}

pub type ColorRGB = (u8, u8, u8);

impl ColorType for ColorRGB {
    const DIMS: usize = 3;

    type Color = Self;
}

pub type ColorRGBW = (u8, u8, u8, u8);

impl ColorType for ColorRGBW {
    const DIMS: usize = 4;

    type Color = Self;
}

/// See <https://github.com/adafruit/Adafruit_NeoPixel/blob/fe882b84951bed066764f9350e600a2ec2aa5a9e/Adafruit_NeoPixel.h#L64>
pub trait Neopixel {
    type Color: ColorType;
    fn blit(c: &Self::Color, buf: &mut [u8]);
}

pub struct RGB;
impl Neopixel for RGB {
    type Color = ColorRGB;

    #[inline]
    fn blit((r, g, b): &Self::Color, buf: &mut [u8]) {
        buf[0] = *r;
        buf[1] = *g;
        buf[2] = *b;
    }
}

pub struct GRB;
impl Neopixel for GRB {
    type Color = ColorRGB;

    #[inline]
    fn blit((r, g, b): &Self::Color, buf: &mut [u8]) {
        buf[0] = *g;
        buf[1] = *r;
        buf[2] = *b;
    }
}

pub struct RGBW;
impl Neopixel for RGBW {
    type Color = ColorRGBW;

    #[inline]
    fn blit((r, g, b, w): &Self::Color, buf: &mut [u8]) {
        buf[0] = *r;
        buf[1] = *g;
        buf[2] = *b;
        buf[3] = *w;
    }
}

pub const fn max_reg_write(max_i2c_write: usize) -> usize {
    max_i2c_write - 2
}

pub const fn max_write(dims: usize, max_i2c_write: usize) -> usize {
    let max_reg: usize = max_reg_write(max_i2c_write);
    max_reg - (max_reg % dims)
}

const fn max_colors(dims: usize, max_i2c_write: usize) -> usize {
    max_reg_write(max_i2c_write) / dims
}

pub const fn mod_colors(n: usize, dims: usize, max_i2c_write: usize) -> usize {
    dims * (n % max_colors(dims, max_i2c_write))
}

pub const fn tail_colors(n: usize, max_colors: usize) -> usize {
    n % max_colors
}

const fn bulk_colors(n: usize, max_colors: usize) -> usize {
    n - tail_colors(n, max_colors)
}

pub trait NeopixelModule<C: Neopixel, const MAX_I2C_WRITE: usize>: SeesawDevice {
    const PIN: u8;

    /// The number of neopixels on the device
    const N_LEDS: u16 = 1;

    const MAX_COLORS: usize = max_colors(C::Color::DIMS, MAX_I2C_WRITE);

    const MAX_WRITE: usize = max_write(C::Color::DIMS, MAX_I2C_WRITE);

    async fn enable_neopixel(&mut self) -> Result<(), SeesawError<Self::Platform>> {
        let addr = self.addr();

        self.driver()
            .write_u8(addr, SET_PIN, Self::PIN)
            .await
            .map_err(Self::error_i2c)?;
        self.driver().set_timeout(Duration::from_micros(10_000));
        self.driver()
            .write_u16(addr, SET_LEN, C::Color::DIMS as u16 * Self::N_LEDS)
            .await
            .map_err(Self::error_i2c)?;
        Ok(self.driver().set_timeout(Duration::from_micros(10_000)))
    }

    async fn set_neopixel_speed(
        &mut self,
        speed: NeopixelSpeed,
    ) -> Result<(), SeesawError<Self::Platform>> {
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
            .await
            .map(|_| self.driver().set_timeout(Duration::from_micros(10_000)))
            .map_err(Self::error_i2c)
    }

    async fn set_neopixel_color(&mut self, c: C::Color) -> Result<(), SeesawError<Self::Platform>>
    where
        [(); Self::MAX_COLORS]: Sized,
        [(); 2 + Self::MAX_COLORS * C::Color::DIMS]: Sized,
        [(); tail_colors(1, Self::MAX_COLORS)]: Sized,
        [(); 2 + tail_colors(1, Self::MAX_COLORS) * C::Color::DIMS]: Sized,
    {
        self.set_nth_neopixel_color(0, c).await
    }

    async fn set_nth_neopixel_color(
        &mut self,
        n: u16,
        c: C::Color,
    ) -> Result<(), SeesawError<Self::Platform>>
    where
        [(); Self::MAX_COLORS]: Sized,
        [(); 2 + Self::MAX_COLORS * C::Color::DIMS]: Sized,
        [(); tail_colors(1, Self::MAX_COLORS)]: Sized,
        [(); 2 + tail_colors(1, Self::MAX_COLORS) * C::Color::DIMS]: Sized,
    {
        self.set_neopixel_colors(n as usize, &[c; 1]).await
    }

    #[inline]
    async fn write_neopixel_buf<const N: usize>(
        &mut self,
        addr: u8,
        reg_off: u16,
        colors: &[C::Color; N],
    ) -> Result<(), SeesawError<Self::Platform>>
    where
        [(); 2 + N * C::Color::DIMS]: Sized,
    {
        let mut buf = [0u8; 2 + N * C::Color::DIMS];
        let reg_off_bytes = u16::to_be_bytes(reg_off);
        buf[0] = reg_off_bytes[0];
        buf[1] = reg_off_bytes[1];
        for (i, color) in colors.iter().enumerate() {
            let boff = i * C::Color::DIMS;
            C::blit(color, &mut buf[2 + boff..2 + boff + C::Color::DIMS])
        }
        self.driver()
            .register_write(addr, SET_BUF, &buf)
            .await
            .map_err(Self::error_i2c)
    }

    async fn set_neopixel_colors<const N: usize>(
        &mut self,
        offset: usize,
        colors: &[C::Color; N],
    ) -> Result<(), SeesawError<Self::Platform>>
    where
        [(); Self::MAX_COLORS]: Sized,
        [(); 2 + Self::MAX_COLORS * C::Color::DIMS]: Sized,
        [(); tail_colors(N, Self::MAX_COLORS)]: Sized,
        [(); 2 + tail_colors(N, Self::MAX_COLORS) * C::Color::DIMS]: Sized,
    {
        assert!(offset + N <= Self::N_LEDS as usize);

        let addr = self.addr();

        let mut reg_off: u16 = (offset * C::Color::DIMS) as u16;
        let mut color_off: usize = 0;
        if N >= Self::MAX_COLORS {
            while color_off < bulk_colors(N, Self::MAX_COLORS) {
                let sub_colors: &[C::Color; Self::MAX_COLORS] = colors
                    [color_off..color_off + Self::MAX_COLORS]
                    .try_into()
                    .unwrap();
                self.write_neopixel_buf::<{ Self::MAX_COLORS }>(addr, reg_off, sub_colors)
                    .await?;

                color_off += Self::MAX_COLORS;
                reg_off += Self::MAX_WRITE as u16;
            }
        }
        if tail_colors(N, Self::MAX_COLORS) > 0 {
            let sub_colors: &[C::Color; tail_colors(N, Self::MAX_COLORS)] = colors
                [color_off..color_off + tail_colors(N, Self::MAX_COLORS)]
                .try_into()
                .unwrap();
            self.write_neopixel_buf::<{ tail_colors(N, Self::MAX_COLORS) }>(
                addr, reg_off, sub_colors,
            )
            .await?;
        }

        Ok(())
    }

    async fn sync_neopixel(&mut self) -> Result<(), SeesawError<Self::Platform>> {
        let addr = self.addr();

        self.driver()
            .register_write(addr, SHOW, &[])
            .await
            .map_err(Self::error_i2c)
    }
}

/// NeopixelModule: The Neopixel protocol speed
#[derive(Debug, Default)]
pub enum NeopixelSpeed {
    Khz400 = 0,
    #[default]
    Khz800 = 1,
}
