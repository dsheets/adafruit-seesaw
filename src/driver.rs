extern crate alloc;

use crate::common::Reg;
use embassy_time::{Duration, Instant, Timer};
use embedded_hal_async::i2c::{self, I2c};

const DELAY: Duration = Duration::from_micros(125);

macro_rules! impl_integer_write {
    ($fn:ident $nty:tt) => {
        pub async fn $fn(
            &mut self,
            addr: i2c::SevenBitAddress,
            reg: &Reg,
            value: $nty,
        ) -> Result<(), <P::I2c as i2c::ErrorType>::Error> {
            self.register_write(addr, reg, &<$nty>::to_be_bytes(value)[..])
                .await
        }
    };
}

macro_rules! impl_integer_read {
    ($fn:ident $nty:tt) => {
        pub async fn $fn(
            &mut self,
            addr: i2c::SevenBitAddress,
            reg: &Reg,
        ) -> Result<$nty, <P::I2c as i2c::ErrorType>::Error> {
            self.register_read::<{ ($nty::BITS / 8) as usize }>(addr, reg)
                .await
                .map($nty::from_be_bytes)
        }
    };
}

pub trait Platform {
    type I2c: i2c::I2c;

    fn i2c(&mut self) -> &mut Self::I2c;
}

#[derive(Debug)]
pub struct Driver<P> {
    platform: P,
    timeout: Instant,
}

impl<P: Platform> Driver<P> {
    pub fn new(platform: P) -> Self {
        Self {
            platform,
            timeout: Instant::from_ticks(0),
        }
    }

    fn lock_timeout(&self) -> Duration {
        self.timeout.saturating_duration_since(Instant::now())
    }

    pub fn set_timeout(&mut self, timeout: Duration) {
        self.timeout = Instant::now() + timeout;
    }

    pub async fn register_read_into(
        &mut self,
        addr: i2c::SevenBitAddress,
        reg: &Reg,
        buf: &mut [u8],
    ) -> Result<(), <P::I2c as i2c::ErrorType>::Error> {
        let dur = self.lock_timeout();
        if dur.as_ticks() > 0 {
            Timer::after(dur).await;
        }

        match self.platform.i2c().write(addr, reg).await {
            Ok(()) => {
                Timer::after(DELAY).await;
                self.platform.i2c().read(addr, buf).await?;
                Ok(())
            }
            Err(e) => {
                self.set_timeout(DELAY);
                Err(e)
            }
        }
    }

    pub async fn register_read<const N: usize>(
        &mut self,
        addr: i2c::SevenBitAddress,
        reg: &Reg,
    ) -> Result<[u8; N], <P::I2c as i2c::ErrorType>::Error> {
        let mut buffer = [0u8; N];
        self.register_read_into(addr, reg, &mut buffer).await?;
        Ok(buffer)
    }

    pub async fn register_write(
        &mut self,
        addr: i2c::SevenBitAddress,
        reg: &Reg,
        bytes: &[u8],
    ) -> Result<(), <P::I2c as i2c::ErrorType>::Error> {
        let dur = self.lock_timeout();
        if dur.as_ticks() > 0 {
            Timer::after(dur).await;
        }

        // let mut ops: [Operation; 2] = [Write(reg), Write(bytes)];
        // self.platform
        //     .i2c()
        //     .transaction(addr, ops.as_mut_slice())
        //     .await?;

        let buf = [reg, bytes].concat();
        let res = self.platform.i2c().write(addr, buf.as_slice()).await;

        self.set_timeout(DELAY);
        res
    }

    impl_integer_read! { read_u8 u8 }
    impl_integer_read! { read_u16 u16 }
    impl_integer_read! { read_u32 u32 }
    impl_integer_read! { read_u64 u64 }
    impl_integer_read! { read_i8 i8 }
    impl_integer_read! { read_i16 i16 }
    impl_integer_read! { read_i32 i32 }
    impl_integer_read! { read_i64 i64 }
    impl_integer_write! { write_u8 u8 }
    impl_integer_write! { write_u16 u16 }
    impl_integer_write! { write_u32 u32 }
    impl_integer_write! { write_u64 u64 }
    impl_integer_write! { write_i8 i8 }
    impl_integer_write! { write_i16 i16 }
    impl_integer_write! { write_i32 i32 }
    impl_integer_write! { write_i64 i64 }
}
