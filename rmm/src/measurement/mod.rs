mod ctx;
mod error;
mod hash;

pub use ctx::HashContext;
pub use error::MeasurementError;
pub use hash::Hashable;
pub use hash::Hasher;

pub const MEASUREMENTS_SLOT_MAX_SIZE: usize = 512 / 8;
pub const MEASUREMENTS_SLOT_NR: usize = 5;
pub const MEASUREMENTS_SLOT_RIM: usize = 0;

pub const RMI_MEASURE_CONTENT: usize = 1;

pub const MEASURE_DESC_TYPE_DATA: u8 = 0;
pub const MEASURE_DESC_TYPE_REC: u8 = 1;
pub const MEASURE_DESC_TYPE_RIPAS: u8 = 2;

#[derive(Copy, Clone, Debug)]
pub struct Measurement([u8; MEASUREMENTS_SLOT_MAX_SIZE]);

impl Measurement {
    pub const fn empty() -> Self {
        Self([0u8; MEASUREMENTS_SLOT_MAX_SIZE])
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.0
    }
    pub fn as_mut_slice(&mut self) -> &mut [u8] {
        &mut self.0
    }
}

impl AsMut<[u8]> for Measurement {
    fn as_mut(&mut self) -> &mut [u8] {
        self.as_mut_slice()
    }
}

impl AsRef<[u8]> for Measurement {
    fn as_ref(&self) -> &[u8] {
        self.as_slice()
    }
}
