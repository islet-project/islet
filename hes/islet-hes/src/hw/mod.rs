//! Hardware specific structures and interfaces.
use alloc::vec::Vec;
use tinyvec::ArrayVec;

pub const MAX_HW_HASH_VALUE_SIZE: usize = 64;
pub const MAX_HW_SW_TYPE_SIZE: usize = 10;
pub const MAX_HW_SW_VERSION_SIZE: usize = 14;

/// Represents a hash value. Calculated by sha256, sha384 or sha512.
pub type HWHash = ArrayVec<[u8; MAX_HW_HASH_VALUE_SIZE]>;
/// Represents a software type value.
pub type HWSWType = ArrayVec<[u8; MAX_HW_SW_TYPE_SIZE]>;
/// Represents a software version value.
pub type HWSWVersion = ArrayVec<[u8; MAX_HW_SW_VERSION_SIZE]>;

/// Structure representing boot measurement metadata,
/// as it is stored in emulated HW.
#[derive(Debug, Default, Clone)]
pub struct BootMeasurementMetadata {
    /// Identifier of the measurement method used
    /// to compute the measurement value.
    pub measurement_type: u16,
    /// Signer identity (hash of public key)
    pub signer_id: HWHash,
    /// Representing the role of the SW component
    pub sw_type: HWSWType,
    /// Version of the SW component in the form of:
    /// "major.minor.revision+build"
    pub sw_version: HWSWVersion,
}

/// Structure representing the boot measurement metadata and value.
#[derive(Debug, Default, Clone)]
pub struct BootMeasurement {
    /// Contains boot measurement metadata
    pub metadata: BootMeasurementMetadata,
    /// Value of boot measurement (hash)
    pub measurement_value: HWHash,
}

/// Represents a binary hardware 256bit symmetric key
pub type HWSymmetricKey = ArrayVec<[u8; 32]>;
/// Represents a binary hardware asymmetric key.
/// Currently using ECC Curve-P384 (384bit), might be a subject to change.
pub type HWAsymmetricKey = ArrayVec<[u8; 48]>;

/// Interface for fetching hardware specific data:
/// - Boot measurements,
/// - Hardware keys,
/// - Claims data,
/// In compliance with
/// [documentation-service.arm.com/static/610aaec33d73a34b640e333b](Arm CCA
/// Security Model 1.0).
pub trait HWData {
    type Error;
    // ---------------HW Keys -------------------
    /// Hardware unique 256bit symmetric key. It represents a randomly unique
    /// seed for each manufactured instance of CCA enabled system.
    fn huk(&self) -> Result<HWSymmetricKey, Self::Error>;
    /// Group unique 256bit symmetric key.
    /// It represents a randomly unique seed that may be shared
    /// with some group of manufactured CCA enabled systems
    /// with the same immutable hardware security properties.
    fn guk(&self) -> Result<HWSymmetricKey, Self::Error>;
    /// Byte string representing CCA Platform Attestation Key.
    /// Optional, can be derived in runtime.
    fn cpak(&self) -> Result<Option<HWAsymmetricKey>, Self::Error>;

    // ---------------HW Bootloader Hash---------
    /// BL2 image signed hash.
    fn bl_hash(&self) -> Result<HWHash, Self::Error>;

    // -------------- HW claims -----------------
    /// Software state of the system. Each entry represents a
    /// [`BootMeasurement`] of software component within the device.
    fn boot_measurements(&self) -> Result<Vec<BootMeasurement>, Self::Error>;

    /// A byte string representing the original implementation signer
    /// of the attestation key and indentifies contract between the report
    /// and verification.
    fn implementation_id(&self) -> Result<[u8; 32], Self::Error>;
    /// Represents the current lifecycle state of the instance.
    /// Custom claim with a value encoded as integer that
    /// is divided to convey a major state and a minor state. The
    /// PSA state and implementation state are encoded as follows:
    /// - version\[15:8\] - PSA lifecycle state - major
    /// - version\[7:0\]  - IMPLEMENTATION DEFINED state - minor
    /// Possible PSA lifecycle states:
    /// - Unknown (0x1000u),
    /// - PSA_RoT_Provisioning (0x2000u),
    /// - Secured (0x3000u),
    /// - Non_PSA_RoT_Debug(0x4000u),
    /// - Recoverable_PSA_RoT_Debug (0x5000u),
    /// - Decommissioned (0x6000u)
    fn security_lifecycle(&self) -> Result<u32, Self::Error>;
    /// Contains the name of a document that describes the 'profile'
    /// of the token, being a full description of the claims, their usage,
    /// verification and token signing. The document name may include
    /// versioning. Custom claim with a value encoded as text string.
    fn profile_definition(&self) -> Result<Option<ArrayVec<[u8; 32]>>, Self::Error>;
    /// The value is a text string that can be used to locate the
    /// service or a URL specifying the address of the service. t is used by
    /// a Relying Party to locate a validation service for the token.
    fn verification_service_url(&self) -> Result<Option<ArrayVec<[u8; 32]>>, Self::Error>;
    /// Describes the set of chosen implementation options of the CCA platform.
    fn platform_config(&self) -> Result<ArrayVec<[u8; 32]>, Self::Error>;
}
