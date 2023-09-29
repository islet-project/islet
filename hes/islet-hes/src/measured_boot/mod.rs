//! Error codes and utilities for error code conversion and formatting.
mod manager;
mod measurement;

pub use manager::{MeasurementMgr, NUM_OF_MEASUREMENT_SLOTS};
pub use measurement::{
    Measurement, MeasurementMetaData, MeasurementType, SWType, SWVersion, SignerHash,
    SW_TYPE_MAX_SIZE, VERSION_MAX_SIZE, SIGNER_ID_MAX_SIZE, SIGNER_ID_MIN_SIZE,
    MEASUREMENT_VALUE_MAX_SIZE, MEASUREMENT_VALUE_MIN_SIZE
};

/// Measurement error enumeration.
#[derive(Debug, PartialEq)]
pub enum MeasurementError {
    /// TODO
    InvalidArgument,
    /// Signer_id doesn't match
    NotPermitted,
    /// Wrong slot_id
    DoesNotExist,
    /// Slot_id extension is locked
    BadState,
    /// HW data is out of bounds
    InvalidData(&'static str),
}
