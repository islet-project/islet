use super::{
    Hasher, Measurement, MeasurementError, MEASUREMENTS_SLOT_RIM, MEASURE_DESC_TYPE_DATA,
    MEASURE_DESC_TYPE_REC, MEASURE_DESC_TYPE_RIPAS, RMI_MEASURE_CONTENT,
};
use crate::rmi::rec::params::Params as RecParams;
use crate::{host::DataPage, realm::rd::Rd, rmi::realm::params::Params as RealmParams, rsi};

pub struct HashContext<'a> {
    hasher: Hasher,
    rd: &'a mut Rd,
}

impl<'a> HashContext<'a> {
    pub fn new(rd: &'a mut Rd) -> Result<Self, MeasurementError> {
        Ok(Self {
            hasher: Hasher::from_hash_algo(rd.hash_algo())?,
            rd,
        })
    }

    pub fn measure_realm_create(&mut self, params: &RealmParams) -> Result<(), rsi::error::Error> {
        crate::rsi::measurement::extend(self.rd, MEASUREMENTS_SLOT_RIM, |rim| {
            self.hasher.hash_object_into(params, rim)
        })
    }

    pub fn extend_measurement(
        &mut self,
        buffer: &[u8],
        index: usize,
    ) -> Result<(), rsi::error::Error> {
        crate::rsi::measurement::extend(self.rd, index, |current| {
            let old_value = *current;

            self.hasher.hash_fields_into(current, |h| {
                h.hash(&old_value.as_ref()[0..self.hasher.output_size()]);
                h.hash(buffer);
            })
        })
    }

    pub fn measure_data_granule(
        &mut self,
        data: &DataPage,
        ipa: usize,
        flags: usize,
    ) -> Result<(), rsi::error::Error> {
        let mut data_measurement = Measurement::empty();

        if flags == RMI_MEASURE_CONTENT {
            self.hasher.hash_fields_into(&mut data_measurement, |h| {
                h.hash(data.as_slice());
            })?;
        }

        crate::rsi::measurement::extend(self.rd, MEASUREMENTS_SLOT_RIM, |current| {
            let oldrim = *current;

            self.hasher.hash_fields_into(current, |h| {
                h.hash_u8(MEASURE_DESC_TYPE_DATA); // desc type
                h.hash([0u8; 7]); // padding
                h.hash_u64(0x100); // desc struct size
                h.hash(oldrim); // old RIM value
                h.hash_usize(ipa); // ipa
                h.hash_usize(flags); // flags
                h.hash(data_measurement); // data granule hash
                h.hash([0u8; 0x100 - 0xa0]); // padding
            })
        })
    }

    pub fn measure_rec_params(&mut self, params: &RecParams) -> Result<(), rsi::error::Error> {
        let mut params_measurement = Measurement::empty();
        self.hasher
            .hash_object_into(params, &mut params_measurement)?;

        crate::rsi::measurement::extend(self.rd, MEASUREMENTS_SLOT_RIM, |current| {
            let oldrim = *current;

            self.hasher.hash_fields_into(current, |h| {
                h.hash_u8(MEASURE_DESC_TYPE_REC); // desc type
                h.hash([0u8; 7]); // padding
                h.hash_u64(0x100); // desc struct size
                h.hash(oldrim); // old RIM value
                h.hash(params_measurement); // REC params hash
                h.hash([0u8; 0x100 - 0x90]); // padding
            })
        })
    }

    pub fn measure_ripas_granule(&mut self, base: u64, top: u64) -> Result<(), rsi::error::Error> {
        crate::rsi::measurement::extend(self.rd, MEASUREMENTS_SLOT_RIM, |current| {
            let oldrim = *current;

            self.hasher.hash_fields_into(current, |h| {
                h.hash_u8(MEASURE_DESC_TYPE_RIPAS); // desc type
                h.hash([0u8; 7]); // padding
                h.hash_u64(0x100); // desc struct size
                h.hash(oldrim); // old RIM value
                h.hash_u64(base); // ipa
                h.hash_u64(top); // level
                h.hash([0u8; 0x100 - 0x60]); // padding to 0x100 size
            })
        })
    }
}
