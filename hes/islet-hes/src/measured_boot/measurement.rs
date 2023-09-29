use core::str::from_utf8;

use alloc::string::{String, ToString};
use tinyvec::ArrayVec;

use super::MeasurementError;
use crate::BootMeasurement;
use crate::ValueHash;

/// Minimal size based on the shortest hash algorithm - sha256
pub const MEASUREMENT_VALUE_MIN_SIZE: usize = 32;
/// Maximal size based on the longest hash algorithm - sha512
pub const MEASUREMENT_VALUE_MAX_SIZE: usize = 64;
/// Minimal size based on the shortest hash algorithm - sha256
pub const SIGNER_ID_MIN_SIZE: usize = MEASUREMENT_VALUE_MIN_SIZE;
/// Maximal size based on the longest hash algorithm - sha512
pub const SIGNER_ID_MAX_SIZE: usize = MEASUREMENT_VALUE_MAX_SIZE;
/// Set based on RSS imlementation.
pub const VERSION_MAX_SIZE: usize = 14;
/// Set based on RSS imlementation.
pub const SW_TYPE_MAX_SIZE: usize = 20;

/// Error message for MessageError::InvalidData(), when converted signer_id has wrong size.
const SIGNER_ID_SIZE_ERROR_MSG: &'static str = "SignerIdSize";
/// Error message for MessageError::InvalidData(), when converted sw_version has wrong size.
const SW_VERSION_SIZE_ERROR_MSG: &'static str = "SWVersionSize";
/// Error message for MessageError::InvalidData(), when converted sw_version is not a valid utf-8.
const SW_VERSION_VALUE_ERROR_MSG: &'static str = "SWVersionNotUtf8";
/// Error message for MessageError::InvalidData(), when converted sw_type has wrong size.
const SW_TYPE_SIZE_ERROR_MSG: &'static str = "SWTypeSize";
/// Error message for MessageError::InvalidData(), when converted sw_type is not a valid utf-8.
const SW_TYPE_VALUE_ERROR_MSG: &'static str = "SWTypeNotUtf8";
/// Error message for MessageError::InvalidData(), when converted measurement_value has wrong size.
const MEASUREMENT_VALUE_SIZE_ERROR_MSG: &'static str = "MeasurementValueSize";

/// Error message for MessageError::InvalidData(), when converted measurement_type has wrong value.
const MEASUREMENT_TYPE_ERROR_MSG: &'static str = "MeasurementType";

/// Represents hash algorithm used for calculating measurement value.
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum MeasurementType {
    Sha256,
    Sha384,
    Sha512,
}

impl Default for MeasurementType {
    /// Arbitrary default value - won't be used, because the root
    /// MeasurementSlot will be set as not populated by default.
    fn default() -> Self {
        Self::Sha256
    }
}

impl TryFrom<u16> for MeasurementType {
    type Error = MeasurementError;
    /// Converts the `MeasurementType` from given [`u16`].
    /// Returns [`MeasurementError::InvalidData`] when `value` is out of scope
    /// of `MeasurementType`.
    fn try_from(value: u16) -> Result<Self, Self::Error> {
        match value {
            0 => Ok(Self::Sha256),
            1 => Ok(Self::Sha384),
            2 => Ok(Self::Sha512),
            _ => Err(MeasurementError::InvalidData(MEASUREMENT_TYPE_ERROR_MSG)),
        }
    }
}

impl MeasurementType {
    pub fn hash_len(&self) -> usize {
        match self {
            MeasurementType::Sha256 => 32,
            MeasurementType::Sha384 => 48,
            MeasurementType::Sha512 => 64,
        }
    }
}

pub type SignerHash = ArrayVec<[u8; SIGNER_ID_MAX_SIZE]>;
pub type SWVersion = String;
pub type SWType = String;

/// Keeps measurement slot metadata.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct MeasurementMetaData {
    /// Represents the hash o signing authority public key.
    pub signer_id: SignerHash,
    /// Represents the issued software version.
    pub sw_version: SWVersion,
    /// Represents the way in which the measurement value of
    /// the software component is computed.
    pub algorithm: MeasurementType,
    /// Represents the role of the software component.
    pub sw_type: SWType,
}

/// Keeps measurement metadata and value.
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Measurement {
    /// Keeps measurement metadata.
    pub metadata: MeasurementMetaData,
    /// Represent the hash value of the measurement.
    pub value: ValueHash,
}

impl TryFrom<BootMeasurement> for Measurement {
    type Error = MeasurementError;
    /// Tries to convert the given [`BootMeasurement`] to `Measurement`.
    /// Returns [`MeasurementError::InvalidData`], when values cannot be properly converted.
    fn try_from(value: BootMeasurement) -> Result<Self, Self::Error> {
        if value.metadata.signer_id.len() < SIGNER_ID_MIN_SIZE
            || value.metadata.signer_id.len() > SIGNER_ID_MAX_SIZE
        {
            return Err(MeasurementError::InvalidData(SIGNER_ID_SIZE_ERROR_MSG));
        }

        if value.metadata.sw_version.len() > VERSION_MAX_SIZE {
            return Err(MeasurementError::InvalidData(SW_VERSION_SIZE_ERROR_MSG));
        }

        if value.metadata.sw_type.len() > SW_TYPE_MAX_SIZE {
            return Err(MeasurementError::InvalidData(SW_TYPE_SIZE_ERROR_MSG));
        }

        if value.measurement_value.len() < MEASUREMENT_VALUE_MIN_SIZE
            || value.measurement_value.len() > MEASUREMENT_VALUE_MAX_SIZE
        {
            return Err(MeasurementError::InvalidData(
                MEASUREMENT_VALUE_SIZE_ERROR_MSG,
            ));
        }

        let sw_version = match from_utf8(&value.metadata.sw_version) {
            Ok(version_str) => version_str,
            Err(_) => return Err(MeasurementError::InvalidData(SW_VERSION_VALUE_ERROR_MSG)),
        }
        .to_string();

        let sw_type = match from_utf8(&value.metadata.sw_type) {
            Ok(type_str) => type_str,
            Err(_) => return Err(MeasurementError::InvalidData(SW_TYPE_VALUE_ERROR_MSG)),
        }
        .to_string();

        Ok(Self {
            metadata: MeasurementMetaData {
                signer_id: value.metadata.signer_id.iter().cloned().collect(),
                sw_version,
                algorithm: value.metadata.measurement_type.try_into()?,
                sw_type,
            },
            value: value.measurement_value,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        hw::{
            BootMeasurementMetadata, HWHash, HWSWType, HWSWVersion, MAX_HW_HASH_VALUE_SIZE,
            MAX_HW_SW_TYPE_SIZE, MAX_HW_SW_VERSION_SIZE,
        },
        BootMeasurement,
    };
    use alloc::{vec, vec::Vec};

    fn boot_measurement_value(len: usize) -> HWHash {
        let mut value: HWHash = HWHash::from([
            0x8a, 0x66, 0x01, 0xf6, 0x70, 0x74, 0x8b, 0xe2, 0x33, 0xff, 0x5d, 0x75, 0xd7, 0xea,
            0x89, 0xa8, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0x01, 0x05, 0x01, 0xEF,
            0x68, 0x07, 0x88, 0xCC, 0x83, 0x09, 0x22, 0xCD, 0x09, 0x61, 0xB6, 0xFF, 0xbb, 0xbb,
            0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0x56, 0x46, 0x58, 0x49, 0x99, 0x31, 0xcf, 0x59,
            0x7d, 0xbc, 0x3a, 0x4e, 0x68, 0x79, 0x8a, 0x1c,
        ]);
        value.truncate(len);
        value
    }

    fn boot_signer_id(len: usize) -> HWHash {
        let mut signer_id: HWHash = HWHash::from([
            0x01, 0x05, 0x01, 0xEF, 0x68, 0x07, 0x88, 0xCC, 0x33, 0x06, 0x54, 0xAB, 0x09, 0x01,
            0x74, 0x77, 0x49, 0x08, 0x93, 0xA8, 0x01, 0x07, 0xEF, 0x01, 0x83, 0x09, 0x22, 0xCD,
            0x09, 0x61, 0xB6, 0xFF, 0x01, 0x05, 0x01, 0xEF, 0x68, 0x07, 0x88, 0xCC, 0x33, 0x06,
            0x54, 0xAB, 0x09, 0x01, 0x74, 0x77, 0x49, 0x08, 0x93, 0xA8, 0x01, 0x07, 0xEF, 0x01,
            0x83, 0x09, 0x22, 0xCD, 0x09, 0x61, 0xB6, 0xFF,
        ]);
        signer_id.truncate(len);
        signer_id
    }

    fn boot_sw_type(len: usize) -> HWSWType {
        let mut sw_type: HWSWType =
            HWSWType::from([0x4D, 0x45, 0x41, 0x53, 0x55, 0x52, 0x45, 0x44, 0x5F, 0x42]);
        sw_type.truncate(len);
        sw_type
    }

    fn boot_sw_version(len: usize) -> HWSWVersion {
        let mut sw_version: HWSWVersion = HWSWVersion::from([
            0x32, 0x35, 0x35, 0x2E, 0x32, 0x35, 0x35, 0x2E, 0x36, 0x35, 0x35, 0x33, 0x35, 0x0,
        ]);
        sw_version.truncate(len);
        sw_version
    }

    #[test]
    fn measurement_type_conversion_ok() {
        let measurement_types_u16: Vec<u16> = vec![0, 1, 2];
        let measurement_types: Vec<MeasurementType> = vec![
            MeasurementType::Sha256,
            MeasurementType::Sha384,
            MeasurementType::Sha512,
        ];

        for (measurement_type_u16, measurement_type) in measurement_types_u16
            .into_iter()
            .zip(measurement_types.into_iter())
        {
            assert_eq!(
                <u16 as TryInto<MeasurementType>>::try_into(measurement_type_u16).unwrap(),
                measurement_type
            );
        }
    }

    #[test]
    fn measurement_conversion_ok() {
        let boot_measurement = BootMeasurement {
            measurement_value: boot_measurement_value(MEASUREMENT_VALUE_MIN_SIZE),
            metadata: BootMeasurementMetadata {
                measurement_type: 0,
                signer_id: boot_signer_id(MAX_HW_HASH_VALUE_SIZE),
                sw_type: boot_sw_type(MAX_HW_SW_TYPE_SIZE),
                sw_version: boot_sw_version(MAX_HW_SW_VERSION_SIZE),
            },
        };

        let measurement: Measurement = boot_measurement.clone().try_into().unwrap();
        assert_eq!(boot_measurement.measurement_value, measurement.value);

        let boot_metadata = &boot_measurement.metadata;
        let metadata = &measurement.metadata;

        assert_eq!(boot_metadata.signer_id, metadata.signer_id);
        assert_eq!(
            <u16 as TryInto<MeasurementType>>::try_into(boot_metadata.measurement_type).unwrap(),
            metadata.algorithm
        );
        assert_eq!(
            &boot_metadata.sw_type[..],
            &metadata.sw_type[..MAX_HW_SW_TYPE_SIZE].as_bytes()[..]
        );
        assert_eq!(boot_metadata.sw_version, metadata.sw_version.as_bytes());
    }

    #[test]
    fn measurement_conversion_too_short_value() {
        let boot_measurement = BootMeasurement {
            // too short
            measurement_value: boot_measurement_value(MEASUREMENT_VALUE_MIN_SIZE - 1),
            metadata: BootMeasurementMetadata {
                measurement_type: 0,
                signer_id: boot_signer_id(MAX_HW_HASH_VALUE_SIZE),
                sw_type: boot_sw_type(MAX_HW_SW_TYPE_SIZE),
                sw_version: boot_sw_version(MAX_HW_SW_VERSION_SIZE),
            },
        };

        assert_eq!(
            <BootMeasurement as TryInto<Measurement>>::try_into(boot_measurement).unwrap_err(),
            MeasurementError::InvalidData(MEASUREMENT_VALUE_SIZE_ERROR_MSG)
        );
    }

    #[test]
    fn measurement_conversion_too_short_signer_id() {
        let boot_measurement = BootMeasurement {
            measurement_value: boot_measurement_value(MAX_HW_HASH_VALUE_SIZE),
            metadata: BootMeasurementMetadata {
                measurement_type: 0,
                // too short
                signer_id: boot_signer_id(SIGNER_ID_MIN_SIZE - 1),
                sw_type: boot_sw_type(MAX_HW_SW_TYPE_SIZE),
                sw_version: boot_sw_version(MAX_HW_SW_VERSION_SIZE),
            },
        };

        assert_eq!(
            <BootMeasurement as TryInto<Measurement>>::try_into(boot_measurement).unwrap_err(),
            MeasurementError::InvalidData(SIGNER_ID_SIZE_ERROR_MSG)
        );
    }

    #[test]
    fn measurement_conversion_bad_measurement_type() {
        let boot_measurement = BootMeasurement {
            measurement_value: boot_measurement_value(MAX_HW_HASH_VALUE_SIZE),
            metadata: BootMeasurementMetadata {
                // doesn't map to any proper value
                measurement_type: 10,
                signer_id: boot_signer_id(MAX_HW_HASH_VALUE_SIZE),
                sw_type: boot_sw_type(MAX_HW_SW_TYPE_SIZE),
                sw_version: boot_sw_version(MAX_HW_SW_VERSION_SIZE),
            },
        };

        assert_eq!(
            <BootMeasurement as TryInto<Measurement>>::try_into(boot_measurement).unwrap_err(),
            MeasurementError::InvalidData(MEASUREMENT_TYPE_ERROR_MSG)
        );
    }
}
