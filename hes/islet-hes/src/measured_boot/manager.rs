use super::*;
use crate::{BootMeasurement, ValueHash};
use alloc::vec::Vec;
use measurement::*;
use sha2::{Digest, Sha256, Sha384, Sha512};

#[derive(Debug)]
/// Keeps measurement in a slot.
struct MeasurementSlot {
    /// Measurement metadata and value.
    measurement: Measurement,
    /// Indicates whether the slot is locked and cannot be further extended.
    is_locked: bool,
    /// Indicates whether the slot is populated with an actual measurement.
    is_populated: bool,
}

impl Default for MeasurementSlot {
    /// Create an instance of `MeasurementSlot` with default values.
    fn default() -> Self {
        Self {
            is_locked: false,
            is_populated: false,
            measurement: Measurement::default(),
        }
    }
}

impl MeasurementSlot {
    /// Extends slot metadata and value.
    pub fn extend(&mut self, measurement: Measurement, lock: bool) {
        self.extend_metadata(measurement.metadata);
        self.extend_value(measurement.value, lock);
    }

    /// Extends slot metadata. Updates only sw type and version, as signer_id
    /// and algorithm never change.
    fn extend_metadata(&mut self, metadata: MeasurementMetaData) {
        // RSS does something strange here - it zero'es metadatas version and sw type
        // see https://git.trustedfirmware.org/TF-M/tf-m-extras.git/tree/partitions/measured_boot/measured_boot.c#n221
        // I think it was supposed to clear and then update the version and sw type.
        self.measurement.metadata.sw_type = metadata.sw_type;
        self.measurement.metadata.sw_version = metadata.sw_version;
    }

    /// Extends slot value only. Recalculates value hash based on current slot value
    /// and new measurement value.
    fn extend_value(&mut self, value: ValueHash, lock: bool) {
        let measurement_value_len = self.measurement.metadata.algorithm.hash_len();

        let total_len = measurement_value_len + value.len();
        let mut temp = Vec::with_capacity(total_len);
        temp.resize(total_len, 0);
        temp[..measurement_value_len]
            .copy_from_slice(&self.measurement.value[..measurement_value_len]);
        temp[measurement_value_len..].copy_from_slice(&value);

        match self.measurement.metadata.algorithm {
            MeasurementType::Sha256 => {
                let mut hasher = Sha256::new();
                hasher.update(&temp);
                let result = hasher.finalize();
                self.measurement.value[..result.len()].copy_from_slice(&result);
            }
            MeasurementType::Sha384 => {
                let mut hasher = Sha384::new();
                hasher.update(&temp);
                let result = hasher.finalize();
                self.measurement.value[..result.len()].copy_from_slice(&result);
            }
            MeasurementType::Sha512 => {
                let mut hasher = Sha512::new();
                hasher.update(&temp);
                let result = hasher.finalize();
                self.measurement.value[..result.len()].copy_from_slice(&result);
            }
        };

        self.is_populated = true;
        self.is_locked = lock;
    }

    /// Overwrites measurement metadata and value. Should be used only when
    /// upopulated slot is extended for the first time.
    pub fn initialize(&mut self, measurement: Measurement, lock: bool) {
        assert!(!self.is_populated, "Do not initialize a populated slot!");

        self.set_metadata(measurement.metadata);
        self.measurement
            .value
            .resize(self.measurement.metadata.algorithm.hash_len(), 0);
        self.extend_value(measurement.value, lock);
    }

    /// Overwrites measurement metadata. Should be used only when unpopulated
    /// slot is extended for the first time.
    fn set_metadata(&mut self, metadata: MeasurementMetaData) {
        self.measurement.metadata = metadata;
    }

    pub fn is_locked(&self) -> bool {
        self.is_locked
    }

    pub fn is_populated(&self) -> bool {
        self.is_populated
    }

    pub fn mark_as_populated(&mut self) {
        self.is_populated = true;
    }

    /// Checks if slot extending is prohibited. The singer_id and algorithm
    /// cannot change.
    pub fn is_prohibited(&self, metadata: &MeasurementMetaData) -> bool {
        metadata.signer_id != self.measurement.metadata.signer_id
            || metadata.algorithm != self.measurement.metadata.algorithm
    }
}

#[derive(Debug)]
/// Responsible for storing all software components measurements and performing
/// the `read_measurement` and `extend_measurement` functions.
pub struct MeasurementMgr {
    measurements: [MeasurementSlot; NUM_OF_MEASUREMENT_SLOTS],
}

/// Maximum number of slots - based on the RSS implementation.
pub const NUM_OF_MEASUREMENT_SLOTS: usize = 32;

impl MeasurementMgr {
    /// Initializes with `BootMeasurement`s repacked and stores as `Measurement` slots.
    pub fn init(
        boot_measurements: Vec<BootMeasurement>,
    ) -> Result<MeasurementMgr, MeasurementError> {
        assert!(
            boot_measurements.len() <= NUM_OF_MEASUREMENT_SLOTS,
            "MeasurementMgr cannot contain HW measurements"
        );

        let mut measurements = core::array::from_fn(|_| MeasurementSlot::default());

        boot_measurements
            .into_iter()
            .enumerate()
            .try_for_each(|(index, boot_measurement)| {
                let measurement = boot_measurement.try_into()?;
                measurements[index].initialize(measurement, false);
                Ok::<(), MeasurementError>(())
            })
            .unwrap();

        Ok(MeasurementMgr { measurements })
    }

    /// Returns measurement metadata, value and locked attribute from given slot_id.
    /// Returns [`MeasurementError::InvalidArgument`], when slot_id is out of bounds.
    /// Returns [`MeasurementError::DoesNotExist`], when is not populated.
    pub fn read_measurement(
        &self,
        slot_id: usize,
    ) -> Result<(&Measurement, bool), MeasurementError> {
        if slot_id >= NUM_OF_MEASUREMENT_SLOTS {
            return Err(MeasurementError::InvalidArgument);
        }
        let slot = &self.measurements[slot_id];

        if !slot.is_populated() {
            return Err(MeasurementError::DoesNotExist);
        }
        Ok((&slot.measurement, slot.is_locked))
    }

    /// Extends measurement with updated metadata, value and locked attribute in given slot_id.
    /// Returns [`MeasurementError::InvalidArgument`], when slot_id is out of bounds.
    /// Returns [`MeasurementError::BadState`], when measurement is locked.
    /// Returns [`MeasurementError::NotPermitted`], when measurements signer id's
    /// and algorithm do not match.
    pub fn extend_measurement(
        &mut self,
        slot_id: usize,
        measurement: Measurement,
        lock: bool,
    ) -> Result<(), MeasurementError> {
        if slot_id >= NUM_OF_MEASUREMENT_SLOTS {
            return Err(MeasurementError::InvalidArgument);
        }

        let slot = &mut self.measurements[slot_id];

        if slot.is_locked() {
            return Err(MeasurementError::BadState);
        }

        if slot.is_populated() {
            if slot.is_prohibited(&measurement.metadata) {
                return Err(MeasurementError::NotPermitted);
            }
            slot.extend(measurement, lock);
        } else {
            slot.initialize(measurement, lock);
            slot.mark_as_populated();
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use crate::hw::BootMeasurementMetadata;
    use alloc::{string::ToString, vec::Vec};
    use core::{iter, str::from_utf8};

    use super::*;
    use tinyvec::ArrayVec;

    fn metadata(algorithm: MeasurementType) -> MeasurementMetaData {
        let signer_id: SignerHash = ArrayVec::from([
            0x01, 0x05, 0x01, 0xEF, 0x68, 0x07, 0x88, 0xCC, 0x33, 0x06, 0x54, 0xAB, 0x09, 0x01,
            0x74, 0x77, 0x49, 0x08, 0x93, 0xA8, 0x01, 0x07, 0xEF, 0x01, 0x83, 0x09, 0x22, 0xCD,
            0x09, 0x61, 0xB6, 0xFF, 0x01, 0x05, 0x01, 0xEF, 0x68, 0x07, 0x88, 0xCC, 0x33, 0x06,
            0x54, 0xAB, 0x09, 0x01, 0x74, 0x77, 0x49, 0x08, 0x93, 0xA8, 0x01, 0x07, 0xEF, 0x01,
            0x83, 0x09, 0x22, 0xCD, 0x09, 0x61, 0xB6, 0xFF,
        ]);

        let sw_version: SWVersion = from_utf8(&[
            0x32, 0x35, 0x35, 0x2E, 0x32, 0x35, 0x35, 0x2E, 0x36, 0x35, 0x35, 0x33, 0x35, 0x0,
        ])
        .unwrap()
        .to_string();

        let sw_type: SWType = from_utf8(&[
            0x4D, 0x45, 0x41, 0x53, 0x55, 0x52, 0x45, 0x44, 0x5F, 0x42, 0x4F, 0x4F, 0x54, 0x5F,
            0x54, 0x45, 0x53, 0x54, 0x53, 0x0,
        ])
        .unwrap()
        .to_string();

        MeasurementMetaData {
            signer_id,
            sw_version,
            algorithm,
            sw_type,
        }
    }

    fn value(len: usize) -> ValueHash {
        let mut value: ValueHash = ValueHash::from([
            0x8a, 0x66, 0x01, 0xf6, 0x70, 0x74, 0x8b, 0xe2, 0x33, 0xff, 0x5d, 0x75, 0xd7, 0xea,
            0x89, 0xa8, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0x01, 0x05, 0x01, 0xEF,
            0x68, 0x07, 0x88, 0xCC, 0x83, 0x09, 0x22, 0xCD, 0x09, 0x61, 0xB6, 0xFF, 0xbb, 0xbb,
            0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0x56, 0x46, 0x58, 0x49, 0x99, 0x31, 0xcf, 0x59,
            0x7d, 0xbc, 0x3a, 0x4e, 0x68, 0x79, 0x8a, 0x1c,
        ]);
        value.truncate(len);
        value
    }

    fn test_extend(
        mgr: &mut MeasurementMgr,
        slot_id: usize,
        algorithm: MeasurementType,
        value: ValueHash,
        expected_value: &[u8],
    ) {
        let metadata = metadata(algorithm);
        let measurement = Measurement {
            metadata: metadata.clone(),
            value,
        };

        mgr.extend_measurement(slot_id, measurement, false).unwrap();

        let (read_measurement, lock) = mgr.read_measurement(slot_id).unwrap();

        assert_eq!(lock, false);
        assert_eq!(metadata, read_measurement.metadata);
        assert_eq!(&read_measurement.value[..], expected_value);
    }

    //---------------POSITIVE EXTEND/READ MEASUREMENT TESTS--------------------

    #[test]
    fn extend_256_with_256() {
        let slot_id = 20;
        let algorithm = MeasurementType::Sha256;

        // First extend on slot 20
        let value_256_0: ValueHash = [
            0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
            0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
            0xbb, 0xbb, 0xbb, 0xbb,
        ]
        .iter()
        .cloned()
        .collect();

        let expected_value_256_0 = [
            0x86, 0xbf, 0xbc, 0xe7, 0xf8, 0x8e, 0x77, 0xda, 0xb6, 0xbb, 0xfb, 0x92, 0x3b, 0xb7,
            0x0e, 0x24, 0x11, 0xd3, 0x74, 0xdc, 0x65, 0x8d, 0xb7, 0x51, 0xc9, 0xbd, 0xec, 0x43,
            0x8f, 0x5c, 0xce, 0x54,
        ];

        let mut mgr = MeasurementMgr::init(Vec::new()).unwrap();

        test_extend(
            &mut mgr,
            slot_id,
            algorithm,
            value_256_0,
            &expected_value_256_0,
        );

        // First extend on slot 21
        let value_256_different_slot: ValueHash = [
            0x75, 0xb2, 0x20, 0xfd, 0x5f, 0xfc, 0xdb, 0xfc, 0x20, 0x73, 0xe4, 0xa0, 0x2f, 0x8c,
            0x53, 0x38, 0x3c, 0x74, 0xbd, 0xf3, 0xb6, 0xab, 0x9f, 0xb6, 0xb3, 0xd2, 0xbb, 0x7a,
            0xa2, 0x08, 0xc5, 0x98,
        ]
        .iter()
        .cloned()
        .collect();

        let expected_value_256_different_slot = [
            0x84, 0x8A, 0x8C, 0xA6, 0x4F, 0x17, 0x54, 0x6A, 0x71, 0x8E, 0xE2, 0x59, 0x45, 0x92,
            0x01, 0xBE, 0x31, 0xF8, 0x85, 0xF1, 0xD3, 0x3F, 0x38, 0x09, 0xE6, 0xF6, 0xF1, 0x82,
            0x3A, 0x90, 0x04, 0x5A,
        ];

        test_extend(
            &mut mgr,
            slot_id + 1,
            algorithm,
            value_256_different_slot,
            &expected_value_256_different_slot,
        );

        // Second extend on slot 20
        let value_256_1: ValueHash = [
            0x01, 0x05, 0x01, 0xEF, 0x68, 0x07, 0x88, 0xCC, 0x33, 0x06, 0x54, 0xAB, 0x09, 0x01,
            0x74, 0x77, 0x49, 0x08, 0x93, 0xA8, 0x01, 0x07, 0xEF, 0x01, 0x83, 0x09, 0x22, 0xCD,
            0x09, 0x61, 0xB6, 0xFF,
        ]
        .iter()
        .cloned()
        .collect();

        let expected_value_256_1 = [
            0xd2, 0xab, 0xc1, 0x26, 0xeb, 0x5c, 0x25, 0x9d, 0x30, 0x33, 0x9c, 0x02, 0xd7, 0x49,
            0x4f, 0x04, 0xe2, 0xd4, 0x49, 0xbe, 0x81, 0xa9, 0x60, 0x39, 0x56, 0x0e, 0x56, 0x90,
            0xb3, 0xda, 0xaf, 0x25,
        ];

        test_extend(
            &mut mgr,
            slot_id,
            algorithm,
            value_256_1,
            &expected_value_256_1,
        );
    }

    #[test]
    fn test_extend_256_with_512() {
        let slot_id = 22;
        let algorithm = MeasurementType::Sha256;

        let value_512_0 = ArrayVec::from([
            0x8a, 0x66, 0x01, 0xf6, 0x70, 0x74, 0x8b, 0xe2, 0x33, 0xff, 0x5d, 0x75, 0xd7, 0xea,
            0x89, 0xa8, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0x01, 0x05, 0x01, 0xEF,
            0x68, 0x07, 0x88, 0xCC, 0x83, 0x09, 0x22, 0xCD, 0x09, 0x61, 0xB6, 0xFF, 0xbb, 0xbb,
            0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0x56, 0x46, 0x58, 0x49, 0x99, 0x31, 0xcf, 0x59,
            0x7d, 0xbc, 0x3a, 0x4e, 0x68, 0x79, 0x8a, 0x1c,
        ]);

        let expected_value_256_0 = [
            0x75, 0xb2, 0x20, 0xfd, 0x5f, 0xfc, 0xdb, 0xfc, 0x20, 0x73, 0xe4, 0xa0, 0x2f, 0x8c,
            0x53, 0x38, 0x3c, 0x74, 0xbd, 0xf3, 0xb6, 0xab, 0x9f, 0xb6, 0xb3, 0xd2, 0xbb, 0x7a,
            0xa2, 0x08, 0xc5, 0x98,
        ];

        let mut mgr = MeasurementMgr::init(Vec::new()).unwrap();

        test_extend(
            &mut mgr,
            slot_id,
            algorithm,
            value_512_0,
            &expected_value_256_0,
        );

        let value_512_1 = ArrayVec::from([
            0x56, 0x46, 0x58, 0x49, 0x99, 0x31, 0xcf, 0x59, 0x7d, 0xbc, 0x3a, 0x4e, 0x68, 0x79,
            0x8a, 0x1c, 0x01, 0x05, 0x01, 0xEF, 0x68, 0x07, 0x88, 0xCC, 0x83, 0x09, 0x22, 0xCD,
            0x09, 0x61, 0xB6, 0xFF, 0x7d, 0xbc, 0x3a, 0x4e, 0x68, 0x79, 0x8a, 0x1c, 0x01, 0x05,
            0x01, 0xEF, 0x68, 0x07, 0x88, 0xCC, 0x8a, 0x66, 0x01, 0xf6, 0x70, 0x74, 0x8b, 0xe2,
            0x83, 0x09, 0x22, 0xCD, 0x09, 0x61, 0xB6, 0xFF,
        ]);

        let expected_value_256_1 = [
            0x79, 0xc3, 0xb9, 0xca, 0xf7, 0x7b, 0xa7, 0xf3, 0x20, 0x65, 0x77, 0x04, 0x09, 0x60,
            0x8a, 0x02, 0x4f, 0xa5, 0x45, 0x9c, 0x6d, 0x10, 0xa7, 0xca, 0x59, 0x3f, 0xf1, 0x4d,
            0x9b, 0x37, 0xe8, 0x3c,
        ];

        test_extend(
            &mut mgr,
            slot_id,
            algorithm,
            value_512_1,
            &expected_value_256_1,
        );
    }

    #[test]
    fn test_extend_512_with_256() {
        let slot_id = 23;
        let algorithm = MeasurementType::Sha512;

        let value_256_0: ValueHash = [
            0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
            0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb,
            0xbb, 0xbb, 0xbb, 0xbb,
        ]
        .iter()
        .cloned()
        .collect();

        let expected_value_512_0 = [
            0xa6, 0x34, 0xb5, 0xb6, 0xe5, 0xe9, 0x71, 0x5d, 0x47, 0x88, 0xf7, 0x2f, 0x1a, 0x39,
            0xc3, 0xe2, 0xd2, 0xe1, 0x03, 0xde, 0xcf, 0xb8, 0xf8, 0xd6, 0x22, 0xcc, 0xff, 0x8c,
            0xb2, 0xe3, 0xe8, 0x22, 0x74, 0x31, 0x6c, 0x4b, 0x8d, 0x66, 0x25, 0x11, 0xcc, 0x1c,
            0xf9, 0x0a, 0x24, 0x56, 0xdf, 0xf2, 0xb6, 0x20, 0xd3, 0xbe, 0xf0, 0xe0, 0xb9, 0x56,
            0xcf, 0x3a, 0xcc, 0x78, 0xe6, 0x58, 0xbe, 0x40,
        ];

        let mut mgr = MeasurementMgr::init(Vec::new()).unwrap();

        test_extend(
            &mut mgr,
            slot_id,
            algorithm,
            value_256_0,
            &expected_value_512_0,
        );

        let value_256_1: ValueHash = [
            0x01, 0x05, 0x01, 0xEF, 0x68, 0x07, 0x88, 0xCC, 0x33, 0x06, 0x54, 0xAB, 0x09, 0x01,
            0x74, 0x77, 0x49, 0x08, 0x93, 0xA8, 0x01, 0x07, 0xEF, 0x01, 0x83, 0x09, 0x22, 0xCD,
            0x09, 0x61, 0xB6, 0xFF,
        ]
        .iter()
        .cloned()
        .collect();

        let expected_value_512_1 = [
            0x53, 0x2e, 0x19, 0xa7, 0xdf, 0x21, 0xe7, 0x7f, 0x6a, 0xe0, 0xe6, 0x30, 0xda, 0xfb,
            0x39, 0x67, 0x8c, 0xdd, 0x74, 0x5f, 0x30, 0x9f, 0x10, 0x80, 0xb3, 0xfa, 0x88, 0x75,
            0x33, 0x31, 0x00, 0x46, 0x59, 0x4f, 0x76, 0xb8, 0xfe, 0x5b, 0xae, 0xc4, 0xeb, 0xe2,
            0xe9, 0x15, 0x02, 0x4f, 0x34, 0x4e, 0x88, 0xc0, 0x43, 0xcd, 0x90, 0x14, 0xcd, 0xaa,
            0xd1, 0xc5, 0x73, 0xba, 0xc3, 0x4c, 0x56, 0xbe,
        ];

        test_extend(
            &mut mgr,
            slot_id,
            algorithm,
            value_256_1,
            &expected_value_512_1,
        );
    }

    #[test]
    fn extend_512_with_512() {
        let slot_id = NUM_OF_MEASUREMENT_SLOTS - 1;
        let algorithm = MeasurementType::Sha512;

        let value_512_0: ValueHash = [
            0x8a, 0x66, 0x01, 0xf6, 0x70, 0x74, 0x8b, 0xe2, 0x33, 0xff, 0x5d, 0x75, 0xd7, 0xea,
            0x89, 0xa8, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0x01, 0x05, 0x01, 0xEF,
            0x68, 0x07, 0x88, 0xCC, 0x83, 0x09, 0x22, 0xCD, 0x09, 0x61, 0xB6, 0xFF, 0xbb, 0xbb,
            0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0x56, 0x46, 0x58, 0x49, 0x99, 0x31, 0xcf, 0x59,
            0x7d, 0xbc, 0x3a, 0x4e, 0x68, 0x79, 0x8a, 0x1c,
        ]
        .iter()
        .cloned()
        .collect();

        let expected_value_512_0 = [
            0x8e, 0x17, 0x5a, 0x1d, 0xcd, 0x79, 0xb8, 0xb5, 0x1c, 0xe9, 0xe2, 0x59, 0xc2, 0x56,
            0x83, 0x05, 0xb7, 0x3f, 0x5f, 0x26, 0xf5, 0x67, 0x3a, 0x8c, 0xf7, 0x81, 0xa9, 0x45,
            0x98, 0xe4, 0x4f, 0x67, 0xfd, 0xf4, 0x92, 0x68, 0x69, 0xee, 0x76, 0x67, 0xe9, 0x12,
            0x0b, 0x5c, 0x1b, 0x97, 0x62, 0x5c, 0xc9, 0x6d, 0x34, 0x7c, 0x23, 0xce, 0x3c, 0x5f,
            0x76, 0x3b, 0xf1, 0xd9, 0xb5, 0x47, 0x81, 0xf6,
        ];

        let mut mgr = MeasurementMgr::init(Vec::new()).unwrap();

        test_extend(
            &mut mgr,
            slot_id,
            algorithm,
            value_512_0,
            &expected_value_512_0,
        );

        let value_512_1: ValueHash = [
            0x56, 0x46, 0x58, 0x49, 0x99, 0x31, 0xcf, 0x59, 0x7d, 0xbc, 0x3a, 0x4e, 0x68, 0x79,
            0x8a, 0x1c, 0x01, 0x05, 0x01, 0xEF, 0x68, 0x07, 0x88, 0xCC, 0x83, 0x09, 0x22, 0xCD,
            0x09, 0x61, 0xB6, 0xFF, 0x7d, 0xbc, 0x3a, 0x4e, 0x68, 0x79, 0x8a, 0x1c, 0x01, 0x05,
            0x01, 0xEF, 0x68, 0x07, 0x88, 0xCC, 0x8a, 0x66, 0x01, 0xf6, 0x70, 0x74, 0x8b, 0xe2,
            0x83, 0x09, 0x22, 0xCD, 0x09, 0x61, 0xB6, 0xFF,
        ]
        .iter()
        .cloned()
        .collect();

        let expected_value_512_1 = [
            0x69, 0x8f, 0xc1, 0x9d, 0xc0, 0xfb, 0x93, 0xc9, 0x78, 0x31, 0x52, 0xd9, 0x33, 0x6f,
            0x35, 0xa7, 0x9a, 0x2d, 0x48, 0xdb, 0x45, 0xa8, 0xd4, 0xc4, 0x8c, 0x0e, 0xef, 0xcb,
            0xeb, 0xc0, 0x11, 0x0d, 0xa2, 0xe4, 0x0f, 0x62, 0x78, 0x34, 0xdd, 0x8e, 0x46, 0xa9,
            0x2b, 0xaa, 0x23, 0x00, 0x6b, 0x36, 0xc6, 0x79, 0xc0, 0x4e, 0x14, 0xca, 0x91, 0x3f,
            0xd2, 0xde, 0xe2, 0x38, 0x58, 0xd5, 0x43, 0xd2,
        ];

        test_extend(
            &mut mgr,
            slot_id,
            algorithm,
            value_512_1,
            &expected_value_512_1,
        );
    }

    //---------------NEGATIVE WRONG PARAMETERS TESTS---------------------------

    #[test]
    fn test_prohibited_extend_after_lock() {
        let slot_id = 9;

        let measurement = Measurement {
            metadata: metadata(MeasurementType::Sha256),
            value: value(32),
        };

        let mut mgr = MeasurementMgr::init(Vec::new()).unwrap();

        mgr.extend_measurement(slot_id, measurement.clone(), true)
            .unwrap();

        assert_eq!(
            mgr.extend_measurement(slot_id, measurement, false)
                .unwrap_err(),
            MeasurementError::BadState
        );
    }

    #[test]
    fn test_prohibited_algorithm_change() {
        let slot_id = 10;
        let value = value(32);
        let mut metadata = metadata(MeasurementType::Sha256);

        let measurement = Measurement {
            metadata: metadata.clone(),
            value: value.clone(),
        };

        let mut mgr = MeasurementMgr::init(Vec::new()).unwrap();

        mgr.extend_measurement(slot_id, measurement, false).unwrap();

        metadata.algorithm = MeasurementType::Sha384;

        let measurement = Measurement {
            metadata: metadata.clone(),
            value: value.clone(),
        };

        assert_eq!(
            mgr.extend_measurement(slot_id, measurement, false)
                .unwrap_err(),
            MeasurementError::NotPermitted
        );
    }

    #[test]
    fn test_prohibited_signer_change() {
        let slot_id = 10;
        let value = value(32);
        let mut metadata = metadata(MeasurementType::Sha256);

        let measurement = Measurement {
            metadata: metadata.clone(),
            value: value.clone(),
        };

        let mut mgr = MeasurementMgr::init(Vec::new()).unwrap();

        mgr.extend_measurement(slot_id, measurement, false).unwrap();

        metadata.signer_id[13] = 13;

        let measurement = Measurement {
            metadata: metadata.clone(),
            value: value.clone(),
        };

        assert_eq!(
            mgr.extend_measurement(slot_id, measurement, false)
                .unwrap_err(),
            MeasurementError::NotPermitted
        );
    }

    #[test]
    fn test_extend_slot_out_of_bounds() {
        let slot_id = NUM_OF_MEASUREMENT_SLOTS;
        let value = value(32);
        let metadata = metadata(MeasurementType::Sha256);

        let mut mgr = MeasurementMgr::init(Vec::new()).unwrap();

        assert_eq!(
            mgr.extend_measurement(slot_id, Measurement { metadata, value }, false)
                .unwrap_err(),
            MeasurementError::InvalidArgument
        );
    }

    #[test]
    fn test_read_slot_out_of_bounds() {
        let slot_id = NUM_OF_MEASUREMENT_SLOTS;
        let mgr = MeasurementMgr::init(Vec::new()).unwrap();

        assert_eq!(
            mgr.read_measurement(slot_id).unwrap_err(),
            MeasurementError::InvalidArgument
        );
    }

    #[test]
    fn test_read_slot_unpopulated() {
        let slot_id = 20;
        let mgr = MeasurementMgr::init(Vec::new()).unwrap();

        assert_eq!(
            mgr.read_measurement(slot_id).unwrap_err(),
            MeasurementError::DoesNotExist
        );
    }

    //---------------BOOT MEASUREMENTS INITIALIZATION TESTS--------------------
    fn boot_measurements() -> Vec<BootMeasurement> {
        Vec::from([
            BootMeasurement {
                measurement_value: [
                    0x61, 0x97, 0x3b, 0x4f, 0x62, 0x0c, 0x2a, 0xe6, 0xc7, 0x63, 0x51, 0x18, 0xa0,
                    0xb4, 0x37, 0x6d, 0x15, 0x34, 0x4c, 0x1c, 0x53, 0xa2, 0x17, 0x89, 0xb1, 0xaa,
                    0x95, 0xd2, 0x0f, 0x3c, 0x45, 0x06,
                ]
                .iter()
                .cloned()
                .collect(),
                metadata: BootMeasurementMetadata {
                    signer_id: [
                        0xc6, 0xc3, 0x2a, 0x95, 0x7d, 0xf4, 0xc6, 0x69, 0x8c, 0x55, 0x0b, 0x69,
                        0x5d, 0x02, 0x2e, 0xd5, 0x18, 0x0c, 0xae, 0x71, 0xf8, 0xb4, 0x9c, 0xbb,
                        0x75, 0xe6, 0x06, 0x1c, 0x2e, 0xf4, 0x97, 0xe1,
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                    measurement_type: 0,
                    sw_type: b"BL1".iter().cloned().collect(),
                    sw_version: b"0.1.0".iter().cloned().collect(),
                },
            },
            BootMeasurement {
                measurement_value: [
                    0xf1, 0x5f, 0x95, 0x3b, 0xe5, 0x0d, 0xad, 0x92, 0xc3, 0xb2, 0xaa, 0x32, 0x97,
                    0xe6, 0xa4, 0xa8, 0xd6, 0x6d, 0x33, 0x63, 0x84, 0x49, 0xec, 0x19, 0x22, 0xb4,
                    0xa7, 0x92, 0x4a, 0x7b, 0x30, 0x22,
                ]
                .iter()
                .cloned()
                .collect(),
                metadata: BootMeasurementMetadata {
                    signer_id: [
                        0xa0, 0x64, 0xb1, 0xad, 0x60, 0xfa, 0x18, 0x33, 0x94, 0xdd, 0xa5, 0x78,
                        0x91, 0x35, 0x7f, 0x97, 0x2e, 0x4f, 0xe7, 0x22, 0x78, 0x2a, 0xdf, 0xf1,
                        0x85, 0x4c, 0x8b, 0x2a, 0x14, 0x2c, 0x04, 0x10,
                    ]
                    .iter()
                    .cloned()
                    .collect(),
                    measurement_type: 0,
                    sw_type: b"BL2".iter().cloned().collect(),
                    sw_version: b"1.9.0+0".iter().cloned().collect(),
                },
            },
        ])
    }

    const BL_1_MEASUREMENT_SLOT_ID: usize = 0;
    const BL_2_MEASUREMENT_SLOT_ID: usize = 1;

    #[test]
    fn test_boot_measurements_ok() {
        let boot_measurements = boot_measurements();
        let mgr = MeasurementMgr::init(boot_measurements.clone()).unwrap();

        let bl1_measurement = &boot_measurements[0];
        let (measurement, lock) = mgr.read_measurement(BL_1_MEASUREMENT_SLOT_ID).unwrap();

        let expected_measurement_value: ValueHash = [
            0x69, 0x7d, 0xe4, 0x40, 0x7d, 0xae, 0x45, 0xc0, 0x75, 0x06, 0xd1, 0xf0, 0x0b, 0x3d,
            0xbf, 0x5c, 0xe1, 0xdb, 0x41, 0xf6, 0x9e, 0x17, 0x50, 0xa3, 0x11, 0xf9, 0x1d, 0x21,
            0x3e, 0x11, 0x98, 0x89,
        ]
        .iter()
        .cloned()
        .collect();

        assert_eq!(lock, false);
        assert_eq!(
            measurement.metadata,
            <BootMeasurement as TryInto<Measurement>>::try_into(bl1_measurement.clone())
                .unwrap()
                .metadata
        );
        assert_eq!(measurement.value, expected_measurement_value);

        let bl2_measurement = &boot_measurements[1];
        let (measurement, lock) = mgr.read_measurement(BL_2_MEASUREMENT_SLOT_ID).unwrap();

        let expected_measurement_value_2: ValueHash = [
            0xa1, 0x9b, 0x4c, 0xf9, 0x32, 0x89, 0xd6, 0x89, 0xc8, 0xa9, 0xb7, 0xfe, 0x16, 0x6b,
            0x4c, 0x5c, 0x12, 0xab, 0xc1, 0x12, 0xb3, 0x5f, 0xba, 0x81, 0xdf, 0x12, 0xf1, 0x4a,
            0x99, 0x6f, 0x9d, 0x81,
        ]
        .iter()
        .cloned()
        .collect();

        assert_eq!(lock, false);
        assert_eq!(
            measurement.metadata,
            <BootMeasurement as TryInto<Measurement>>::try_into(bl2_measurement.clone())
                .unwrap()
                .metadata
        );
        assert_eq!(measurement.value, expected_measurement_value_2);
    }

    #[test]
    #[should_panic(expected = "MeasurementMgr cannot contain HW measurements")]
    fn test_boot_measurements_too_many() {
        let boot_measurements = Vec::from_iter(
            iter::repeat(BootMeasurement::default()).take(NUM_OF_MEASUREMENT_SLOTS + 1),
        );

        let _ = MeasurementMgr::init(boot_measurements);
    }
}
