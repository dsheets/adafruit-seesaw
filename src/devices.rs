use crate::{
    driver::Driver,
    modules::{
        adc::AdcModule,
        encoder::EncoderModule,
        gpio::{GpioModule, PinMode},
        neopixel::{NeopixelModule, GRB, RGB},
        status::StatusModule,
        timer::TimerModule,
    },
    seesaw_device, HardwareId, Platform, SeesawDeviceInit, SeesawError,
};

/// All devices implement the status module
impl<T: super::SeesawDevice> StatusModule for T {}

seesaw_device! {
    #[doc(hidden)]
    name: GenericDevice,
    hardware_id: HardwareId::SAMD09,
    product_id: 0,
    default_addr: 0x49,
    modules: []
}

impl<P: Platform, const N: usize> SeesawDeviceInit for GenericDevice<P, N> {
    async fn init(mut self) -> Result<Self, SeesawError<Self::Platform>> {
        self.reset().await.map(|_| self)
    }
}

seesaw_device! {
    /// ArcadeButton1x4
    ///
    /// Button | Pin | GPIO
    /// ---|---|---
    /// SW1 | 18 | PA01
    /// SW2 | 19 | PA02
    /// SW3 | 20 | PA03
    /// SW4 | 2 | PA06
    ///
    /// LED | PIN | GPIO
    /// ---|---|---
    /// PWM1 | 12 | PC00
    /// PWM2 | 13 | PC01
    /// PWM3 | 0 | PA04
    /// PWM4 | 1 | PA05
    ///
    name: ArcadeButton1x4,
    hardware_id: HardwareId::ATTINY817,
    product_id: 5296,
    default_addr: 0x3A,
    modules: [
        GpioModule,
        TimerModule
    ]
}

impl<P: Platform, const N: usize> SeesawDeviceInit for ArcadeButton1x4<P, N> {
    async fn init(mut self) -> Result<Self, SeesawError<Self::Platform>> {
        self.reset_and_verify_seesaw().await?;
        self.enable_buttons().await?;
        Ok(self)
    }
}

impl<P: Platform, const N: usize> ArcadeButton1x4<P, N> {
    pub async fn button_values(&mut self) -> Result<[bool; 4], SeesawError<P>> {
        let mut btns = [false; 4];
        for (i, &pin) in [18u8, 19, 20, 2].iter().enumerate() {
            btns[i] = self.digital_read(pin).await?;
        }
        Ok::<[bool; 4], SeesawError<P>>(btns)
    }

    /// Set the pin mode of the 4 buttons to input pullup:
    pub async fn enable_buttons(&mut self) -> Result<(), SeesawError<P>> {
        self.set_pin_mode(18, PinMode::InputPullup).await?;
        self.set_pin_mode(19, PinMode::InputPullup).await?;
        self.set_pin_mode(20, PinMode::InputPullup).await?;
        self.set_pin_mode(2, PinMode::InputPullup).await
    }

    pub async fn set_led_duty_cycles(&mut self, pwms: &[u8; 4]) -> Result<(), SeesawError<P>> {
        for (i, &pin) in [12u8, 13, 0, 1].iter().enumerate() {
            self.analog_write(pin, pwms[i]).await?
        }
        Ok::<(), SeesawError<P>>(())
    }
}

seesaw_device! {
    /// NeoKey1x4
    name: NeoKey1x4,
    hardware_id: HardwareId::SAMD09,
    product_id: 4980,
    default_addr: 0x30,
    modules: [
        GpioModule,
        NeopixelModule<GRB> { num_leds: 4, pin: 3 },
    ]
}

impl<P: Platform, const N: usize> SeesawDeviceInit for NeoKey1x4<P, N> {
    async fn init(mut self) -> Result<Self, SeesawError<P>> {
        self.reset_and_verify_seesaw().await?;
        self.enable_neopixel().await?;
        self.enable_button_pins().await?;
        Ok(self)
    }
}

impl<P: Platform, const N: usize> NeoKey1x4<P, N> {
    pub async fn enable_button_pins(&mut self) -> Result<(), SeesawError<P>> {
        self.set_pin_mode_bulk(
            (1 << 4) | (1 << 5) | (1 << 6) | (1 << 7),
            PinMode::InputPullup,
        )
        .await
    }

    pub async fn keys(&mut self) -> Result<u8, SeesawError<P>> {
        self.digital_read_bulk().await.map(|r| (r >> 4 & 0xF) as u8)
    }
}

seesaw_device!(
    /// NeoSlider
    name: NeoSlider,
    hardware_id: HardwareId::ATTINY817,
    product_id: 5295,
    default_addr: 0x30,
    modules: [
        AdcModule,
        GpioModule,
        NeopixelModule<RGB> { num_leds: 4, pin: 14 },
    ]
);

impl<P: Platform, const N: usize> SeesawDeviceInit for NeoSlider<P, N> {
    async fn init(mut self) -> Result<Self, SeesawError<P>> {
        self.reset_and_verify_seesaw().await?;
        self.enable_neopixel().await?;
        Ok(self)
    }
}

impl<P: Platform, const N: usize> NeoSlider<P, N> {
    pub async fn slider_value(&mut self) -> Result<u16, SeesawError<P>> {
        self.analog_read(18).await
    }
}

seesaw_device! {
    /// RotaryEncoder
    name: RotaryEncoder,
    hardware_id: HardwareId::SAMD09,
    product_id: 4991,
    default_addr: 0x36,
    modules:  [
        EncoderModule { button_pin: 24 },
        GpioModule,
        NeopixelModule<RGB> { num_leds: 1, pin: 6 },
    ]
}

impl<P: Platform, const N: usize> SeesawDeviceInit for RotaryEncoder<P, N> {
    async fn init(mut self) -> Result<Self, SeesawError<P>> {
        self.reset_and_verify_seesaw().await?;
        self.enable_button().await?;
        self.enable_neopixel().await?;
        Ok(self)
    }
}
