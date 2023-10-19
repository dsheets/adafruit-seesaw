#[macro_export(local_inner_macros)]

macro_rules! seesaw_device {
    (
        $(#[$attr:meta])*
        name: $name:ident,
        hardware_id: $hardware_id:expr,
        product_id: $product_id:expr,
        default_addr: $default_addr:expr,
        modules: [
            $($module_name:ident$(<$module_param:ty>)? $({
                $($const_name:ident: $const_value:expr $(,)?),*
            })?),*
            $(,)?
        ]
        $(,)?
    ) => {
        $(#[$attr])*
        ///
        #[doc=core::concat!("[Adafruit Product Page](https://www.adafruit.com/product/", core::stringify!($product_id),")")]
        #[derive(Debug)]
        pub struct $name<P: $crate::Platform, const MAX_I2C_WRITE: usize> {
            addr: u8,
            driver: Driver<P>,
        }

        impl<P: $crate::Platform, const N: usize> $name<P, N> {
            pub const fn default_addr() -> u8 {
                $default_addr
            }
            pub const fn hardware_id() -> $crate::HardwareId {
                $hardware_id
            }
            pub const fn product_id() -> u16 {
                $product_id
            }
        }

        impl<P: $crate::Platform, const N: usize> $crate::SeesawDevice for $name<P, N> {
            type Platform = P;
            const DEFAULT_ADDR: u8 = $default_addr;
            const HARDWARE_ID: $crate::HardwareId = $hardware_id;
            const PRODUCT_ID: u16 = $product_id;

            fn addr(&self) -> u8 {
                self.addr
            }

            fn driver(&mut self) -> &mut Driver<P> {
                &mut self.driver
            }

            fn new(addr: u8, driver: Driver<P>) -> Self {
                Self {
                    addr, driver
                }
            }

            fn new_with_default_addr(driver: Driver<P>) -> Self {
                Self::new(Self::DEFAULT_ADDR, driver)
            }

            fn error_i2c(e: <P::I2c as embedded_hal_async::i2c::ErrorType>::Error) -> $crate::SeesawError<P> {
                $crate::SeesawError::I2c(e)
            }

            fn error_invalid_hardware_id(id: u8) -> $crate::SeesawError<P> {
                $crate::SeesawError::InvalidHardwareId(id)
            }
        }

        $(
            impl_device_module! { $name, $module_name$(<$module_param>)? $({$($const_name: $const_value),*})* }
        )*
    };
}

#[doc(hidden)]
#[macro_export(local_inner_macros)]
macro_rules! impl_device_module {
    ($device:ident, AdcModule $({})?) => {
        impl<P: $crate::Platform, const N: usize> $crate::modules::adc::AdcModule
            for $device<P, N>
        {
        }
    };
    ($device:ident, EncoderModule { button_pin: $button_pin:expr }) => {
        impl<P: $crate::Platform, const N: usize> $crate::modules::encoder::EncoderModule
            for $device<P, N>
        {
            const ENCODER_BTN_PIN: u8 = $button_pin;
        }
    };
    ($device:ident, GpioModule $({})?) => {
        impl<P: $crate::Platform, const N: usize> $crate::modules::gpio::GpioModule
            for $device<P, N>
        {
        }
    };
    ($device:ident, KeypadModule { num_keys: $num_keys:expr }) => {
        impl<P: $crate::Platform, const N: usize> $crate::modules::keypad::KeypadModule<$num_keys>
            for $device<P, N>
        {
        }
    };
    ($device:ident, NeopixelModule<$colors:ty> { num_leds: $num_leds:expr, pin: $pin:expr }) => {
        impl<P: $crate::Platform, const N: usize>
            $crate::modules::neopixel::NeopixelModule<$colors, N> for $device<P, N>
        {
            const N_LEDS: u16 = $num_leds;
            const PIN: u8 = $pin;
        }
    };
    ($device:ident, StatusModule $({})?) => {
        impl<P: $crate::Platform, const N: usize> $crate::modules::StatusModule for $device<P, N> {}
    };
    ($device:ident, TimerModule $({})?) => {
        impl<P: $crate::Platform, const N: usize> $crate::modules::timer::TimerModule
            for $device<P, N>
        {
        }
    };
}
