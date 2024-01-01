pub type Reg = [u8; 2];

pub struct RegValue<const N: usize>
where
    [(); 2 + N]: Sized,
{
    data: [u8; 2 + N],
}

impl<const N: usize> RegValue<N>
where
    [(); 2 + N]: Sized,
{
    pub fn new(reg: &Reg) -> Self {
        let mut data = [0; 2 + N];
        data[0..2].copy_from_slice(reg);
        Self { data }
    }

    pub fn with_bytes(mut self, bytes: &[u8]) -> Self {
        let len = bytes.len();
        if len > N {
            panic!("can't copy {len} bytes into {N} bytes")
        }

        self.data[2..].copy_from_slice(bytes);
        self
    }

    pub fn buffer(&self) -> &[u8] {
        &self.data
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HardwareId {
    ATTINY817 = 0x87,
    SAMD09 = 0x55,
}

impl From<HardwareId> for u8 {
    fn from(value: HardwareId) -> Self {
        value as u8
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Modules {
    Status = 0x00,
    Gpio = 0x01,
    Sercom0 = 0x02,
    Timer = 0x08,
    Adc = 0x09,
    /// `Dac` has a value in the C++ Seesaw library but is not used
    Dac = 0x0A,
    /// `Interrupt` has a value in the C++ Seesaw library but is not used
    Interrupt = 0x0B,
    /// `Dap` has a value in the C++ Seesaw library but is not used
    Dap = 0x0C,
    Eeprom = 0x0D,
    Neopixel = 0x0E,
    Touch = 0x0F,
    Keypad = 0x10,
    Encoder = 0x11,
    Spectrum = 0x12,
}

impl Modules {
    pub const fn into_u8(self) -> u8 {
        self as u8
    }
}
