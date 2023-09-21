use alloc::boxed::Box;
use sha2::Digest;
use sha2::{digest::DynDigest, Sha256, Sha512};

use crate::{
    measurement::MeasurementError,
    rmi::{HASH_ALGO_SHA256, HASH_ALGO_SHA512},
};

pub struct HashWrapper {
    pub hash_func: Box<dyn DynDigest>,
}

impl HashWrapper {
    pub fn hash(&mut self, data: impl AsRef<[u8]>) {
        self.hash_func.update(data.as_ref());
    }

    pub fn hash_u8(&mut self, data: u8) {
        self.hash_func.update(data.to_le_bytes().as_slice());
    }

    pub fn hash_u16(&mut self, data: u16) {
        self.hash_func.update(data.to_le_bytes().as_slice());
    }

    pub fn hash_u32(&mut self, data: u32) {
        self.hash_func.update(data.to_le_bytes().as_slice());
    }

    pub fn hash_u64(&mut self, data: u64) {
        self.hash_func.update(data.to_le_bytes().as_slice());
    }

    pub fn hash_usize(&mut self, data: usize) {
        self.hash_func.update(data.to_le_bytes().as_slice());
    }

    pub fn hash_u64_array(&mut self, array: &[u64]) {
        for el in array.iter() {
            self.hash_func.update(el.to_le_bytes().as_slice());
        }
    }

    fn finish(&mut self, mut out: impl AsMut<[u8]>) -> Result<(), MeasurementError> {
        self.hash_func
            .finalize_into_reset(&mut out.as_mut()[0..self.hash_func.output_size()])
            .map_err(|_| MeasurementError::OutputBufferTooSmall)
    }
}

pub struct Hasher {
    factory: Box<dyn Fn() -> Box<dyn DynDigest>>,
}

impl Hasher {
    pub fn from_hash_algo(hash_algo: u8) -> Result<Self, MeasurementError> {
        let factory: Box<dyn Fn() -> Box<dyn DynDigest>> = match hash_algo {
            HASH_ALGO_SHA256 => Box::new(|| Box::new(Sha256::new())),
            HASH_ALGO_SHA512 => Box::new(|| Box::new(Sha512::new())),
            _ => return Err(MeasurementError::InvalidHashAlgorithmValue(hash_algo)),
        };

        Ok(Self { factory })
    }

    pub fn hash_fields_into(
        &self,
        out: impl AsMut<[u8]>,
        f: impl Fn(&mut HashWrapper),
    ) -> Result<(), MeasurementError> {
        let mut wrapper = HashWrapper {
            hash_func: (self.factory)(),
        };
        f(&mut wrapper);
        wrapper.finish(out)
    }

    pub fn hash_object_into(
        &self,
        obj: &dyn Hashable,
        mut out: impl AsMut<[u8]>,
    ) -> Result<(), MeasurementError> {
        obj.hash(&self, out.as_mut())
    }
}

pub trait Hashable {
    fn hash(&self, hasher: &Hasher, out: &mut [u8]) -> Result<(), MeasurementError>;
}
