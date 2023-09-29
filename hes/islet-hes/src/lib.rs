#![no_std]
//! Islet HES library.

extern crate alloc;

/// Submodule implementing the attestation functionality.
mod attestation;
/// Submodule containing hardware data trait.
mod hw;
/// Submodule implementing the measured boot functionality.
mod measured_boot;

use core::{fmt::Debug, str::from_utf8};

use alloc::{string::ToString, vec::Vec};
use coset::CoseSign1;
use tinyvec::ArrayVec;

pub use measured_boot::{
    Measurement, MeasurementError, MeasurementMetaData, MeasurementMgr, MeasurementType, SWType,
    SWVersion, SignerHash, NUM_OF_MEASUREMENT_SLOTS, SW_TYPE_MAX_SIZE, VERSION_MAX_SIZE, SIGNER_ID_MAX_SIZE, SIGNER_ID_MIN_SIZE,
    MEASUREMENT_VALUE_MAX_SIZE, MEASUREMENT_VALUE_MIN_SIZE
};

pub use hw::{
    BootMeasurement, BootMeasurementMetadata, HWAsymmetricKey, HWData, HWHash, HWSWType,
    HWSWVersion, HWSymmetricKey,
};

pub use attestation::{
    calculate_public_key_hash, AttestationError, AttestationMgr, ECCFamily, HWClaims, HashAlgo,
    KeyBits, KeyMaterialData,
};

pub const MAX_HASH_VALUE_SIZE: usize = 64;
pub type ValueHash = ArrayVec<[u8; MAX_HASH_VALUE_SIZE]>;

/// Possible PSA lifecycle states (major):
pub mod security_lifecycle {
    pub const UNKNOWN: u32 = 0x1000;
    pub const PSA_ROT_PROVISIONNING: u32 = 0x2000;
    pub const SECURED: u32 = 0x3000;
    pub const NON_PSA_ROT_DEBUG: u32 = 0x4000;
    pub const RECOVERABLE_PSA_ROT_DEBUG: u32 = 0x5000;
    pub const DECOMISSIONED: u32 = 0x6000;
}

pub struct IsletHES {
    measured_boot_mgr: MeasurementMgr,
    attestation_mgr: AttestationMgr,
}

#[derive(Debug)]
pub enum IsletHESError {
    /// Implementations may use this error code if none of the other is applicable.
    GenericError,
    /// The parameters passed to the function are invalid.
    InvalidArgument,
    /// Measurement signer_id doesn't match for extend operation
    NotPermitted,
    /// Slot_id of a Measurement is not populated
    DoesNotExist,
    /// Slot_id of a Measurement is lock for extend operation
    BadState,
    /// Requested key size of DAK is not supported
    NotSupported,
}

impl From<MeasurementError> for IsletHESError {
    fn from(value: MeasurementError) -> Self {
        match value {
            MeasurementError::BadState => Self::BadState,
            MeasurementError::DoesNotExist => Self::DoesNotExist,
            MeasurementError::InvalidArgument => Self::InvalidArgument,
            MeasurementError::InvalidData(_) => Self::InvalidArgument,
            MeasurementError::NotPermitted => Self::NotPermitted,
        }
    }
}

impl From<AttestationError> for IsletHESError {
    fn from(value: AttestationError) -> Self {
        match value {
            AttestationError::InvalidArgument => Self::InvalidArgument,
            AttestationError::GenericError => Self::GenericError,
            AttestationError::NotSupported => Self::NotSupported,
        }
    }
}

impl IsletHES {
    /// Initializes IsletHes with data for [`HWData`] interface
    pub fn init<H: HWData>(hw_data: H) -> Result<Self, IsletHESError>
    where
        <H as HWData>::Error: Debug,
    {
        let measured_boot_mgr = MeasurementMgr::init(
            hw_data
                .boot_measurements()
                .map_err(|_| IsletHESError::InvalidArgument)?
        )?;

        let profile_definition = match hw_data
            .profile_definition()
            .map_err(|_| IsletHESError::InvalidArgument)?
        {
            Some(p) => Some(
                from_utf8(&p)
                    .map_err(|_| IsletHESError::InvalidArgument)?
                    .to_string(),
            ),
            None => None,
        };

        let verification_service_url = match hw_data
            .verification_service_url()
            .map_err(|_| IsletHESError::InvalidArgument)?
        {
            Some(p) => Some(
                from_utf8(&p)
                    .map_err(|_| IsletHESError::InvalidArgument)?
                    .to_string(),
            ),
            None => None,
        };

        let attestation_mgr = AttestationMgr::init(
            KeyMaterialData {
                hash: hw_data
                    .bl_hash()
                    .map_err(|_| IsletHESError::InvalidArgument)?,
                guk: hw_data
                    .guk()
                    .map_err(|_| IsletHESError::InvalidArgument)?,
            },
            HWClaims {
                implementation_id: hw_data
                    .implementation_id()
                    .map_err(|_| IsletHESError::InvalidArgument)?,
                security_lifecycle: hw_data
                    .security_lifecycle()
                    .map_err(|_| IsletHESError::InvalidArgument)?,
                profile_definition,
                verification_service_url,
                platform_config: hw_data
                    .platform_config()
                    .map_err(|_| IsletHESError::InvalidArgument)?,
            },
        );

        Ok(IsletHES {
            measured_boot_mgr,
            attestation_mgr,
        })
    }

    /// Resets the measurements database and unmarks DAK key as generated
    pub fn reset<H: HWData>(&mut self, hw_data: H) -> Result<(), IsletHESError>{
        self.measured_boot_mgr = MeasurementMgr::init(
            hw_data
                .boot_measurements()
                .map_err(|_| IsletHESError::InvalidArgument)?
        )?;

        self.attestation_mgr.reset();
        Ok(())
    }

    /// Returns measurement metadata, value and locked attribute from given slot_id.
    /// Returns [`IsletHESError::InvalidArgument`], when slot_id is out of bounds.
    /// Returns [`IsletHESError::DoesNotExist`], when is not populated.
    pub fn read_measurement(
        &self,
        slot_id: usize,
    ) -> Result<(&Measurement, bool), IsletHESError> {
        Ok(self.measured_boot_mgr.read_measurement(slot_id)?)
    }

    /// Extends measurement with updated metadata, value and locked attribute in given slot_id.
    /// Returns [`IsletHESError::InvalidArgument`], when slot_id is out of bounds.
    /// Returns [`IsletHESError::BadState`], when measurement is locked.
    /// Returns [`IsletHESError::NotPermitted`], when measurements signer id's
    /// and algorithm do not match.
    pub fn extend_measurement(
        &mut self,
        slot_id: usize,
        measurement: Measurement,
        lock: bool,
    ) -> Result<(), IsletHESError> {
        Ok(self
            .measured_boot_mgr
            .extend_measurement(slot_id, measurement, lock)?)
    }

    fn fetch_current_measurements(&self) -> Result<Vec<Measurement>, IsletHESError> {
        let mut measurements = Vec::new();
        for i in 0..measured_boot::NUM_OF_MEASUREMENT_SLOTS {
            match self.measured_boot_mgr.read_measurement(i) {
                Ok((measurement, _)) => measurements.push(measurement.clone()),
                Err(MeasurementError::DoesNotExist) => continue,
                Err(e) => return Err(e.into()),
            }
        }
        Ok(measurements)
    }

    /// Generates DAK with [`ECCFamily`] and uses `measurements` ([`Measurement`])
    /// as salt in the process.
    /// Returns bytes of a scalar primitive, which can be used to recreate DAK Private Key.
    /// [`HashAlgo`] is used for verification process, when `get_platform_token` is called.
    /// Returns [`IsletHESError::GenericError`], when CBOR or crypto operation fails.
    pub fn get_delegated_key(
        &mut self,
        ecc_family: ECCFamily,
        key_bits: KeyBits,
        hash_algo: HashAlgo,
    ) -> Result<Vec<u8>, IsletHESError> {
        let measurements = self.fetch_current_measurements()?;

        Ok(self.attestation_mgr.get_delegated_key(
            ecc_family,
            key_bits,
            hash_algo,
            &measurements,
        )?)
    }

    /// Creates a tagged [`CoseSign1`] of the platform token.
    /// `dak_pub_hash` must be a valid hash of DAK Public Key using [`HashAlgo`] passed
    /// in [`IsletHES::get_delegated_key`].
    /// Returns [`IsletHESError::GenericError`], when CBOR or crypto operation fails.
    /// Returns [`IsletHESError::InvalidArgument`], when DAK was not requsted before
    /// this operation, or `dak_pub_hash` is not a valid hash of DAK Public Key.
    pub fn get_platform_token(
        &mut self,
        dak_pub_hash: &[u8],
    ) -> Result<CoseSign1, IsletHESError> {
        let measurements = self.fetch_current_measurements()?;

        Ok(self
            .attestation_mgr
            .get_platform_token(dak_pub_hash, &measurements)?)
    }
}
