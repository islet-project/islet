use alloc::string::{String, ToString};
use alloc::vec::Vec;
use ciborium::{ser, Value};
use coset::{CoseSign1, CoseSign1Builder, HeaderBuilder};
use key_derivation::{derive_p256_key, derive_p384_key, generate_seed};
use p384::ecdsa::{signature::Signer, Signature as P384Signature};
use sha2::{Digest, Sha256, Sha384, Sha512};
use tinyvec::ArrayVec;

use crate::{HWHash, HWSymmetricKey, Measurement, MeasurementType};

/// Supported ecc family types.
#[derive(Copy, Clone, Debug)]
pub enum ECCFamily {
    /// The only one compliant with CCA Security Model 1.0
    SecpR1,
}

/// Supported ecc key bit size.
#[derive(Copy, Clone, Debug)]
pub enum KeyBits {
    Bits256,
    Bits384,
    // Not Supported
    Bits521,
}

/// Supported public dak hash algorithms.
#[derive(Copy, Clone, Debug)]
pub enum HashAlgo {
    Sha256,
    Sha384,
    Sha512,
}

impl HashAlgo {
    /// Converts the given `HashAlgo` into [`usize`] length.
    pub fn len(self) -> usize {
        match self {
            HashAlgo::Sha256 => 32,
            HashAlgo::Sha384 => 48,
            HashAlgo::Sha512 => 64,
        }
    }
}

impl Into<String> for HashAlgo {
    /// Converts the given `HashAlgo` into [`String`] description.
    fn into(self) -> String {
        match self {
            HashAlgo::Sha256 => "sha-256".to_string(),
            HashAlgo::Sha384 => "sha-384".to_string(),
            HashAlgo::Sha512 => "sha-512".to_string(),
        }
    }
}

impl Into<String> for MeasurementType {
    /// Converts the given `MeasurementType` into [`String`].
    fn into(self) -> String {
        match self {
            MeasurementType::Sha256 => "sha-256".to_string(),
            MeasurementType::Sha384 => "sha-384".to_string(),
            MeasurementType::Sha512 => "sha-512".to_string(),
        }
    }
}

/// Error kinds returned by AttestationMgr
#[derive(Debug, PartialEq)]
pub enum AttestationError {
    /// Some parameter or combination of parameters are recognised as invalid:
    /// - challenge size is not allowed
    /// - challenge object is unavailable
    /// - token buffer is unavailable
    InvalidArgument,
    /// An error occurred that does not correspond to any defined
    /// failure cause.
    GenericError,
    /// The requested operation or a parameter is not supported
    /// by this implementation.
    NotSupported,
}

/// Represents data required for key derivation (CPAK/DAK).
#[derive(Clone, Debug)]
pub struct KeyMaterialData {
    /// Bootloader image singed hash.
    pub hash: HWHash,
    /// Group unique 256bit symmetric key.
    pub guk: HWSymmetricKey,
}

/// Hardware provisioned claims.
#[derive(Clone, Debug)]
pub struct HWClaims {
    pub implementation_id: [u8; 32],
    pub security_lifecycle: u32,
    pub profile_definition: Option<String>,
    pub verification_service_url: Option<String>,
    pub platform_config: ArrayVec<[u8; 32]>,
}

/// Represents DAK specific data.
pub struct DAKInfo {
    /// Hash algorithm as will be passed during token creation.
    hash_algo: HashAlgo,
    /// Key bit size
    key_bits: KeyBits,
    /// Generated DAK private key.
    key: Vec<u8>,
}

type InstanceId = ArrayVec<[u8; 33]>;

/// Represents CPAK specific data.
struct CPAK {
    // TODO: SigningKey is defacto SecretKey. Maybe change key derivation
    // to return SigningKey instead of the SecretKey
    /// CPAK private key.
    key: p384::SecretKey,
    /// CPAK sha256, used for creating instance id.
    //RSS uses sha256 for instance id
    instance_id: InstanceId,
}

/// Attestation manager implementing `get_delegated_key` and `get_platform_token` functinality.
pub struct AttestationMgr {
    /// CPAK data
    cpak: CPAK,
    /// Key derivation material for DAK and CPAK generation
    derivation_material: KeyMaterialData,
    /// Claims from HW
    claims: HWClaims,
    /// When DAK is generated keeps data required for DAK verification and signing
    dak: Option<DAKInfo>,
}

/// Calculate hash for given `key` with chosen [`HashAlgo`]
pub fn calculate_public_key_hash(sec1_public_key: Vec<u8>, hash_algo: HashAlgo) -> Vec<u8> {
    match hash_algo {
        HashAlgo::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(sec1_public_key);
            hasher.finalize().to_vec()
        }
        HashAlgo::Sha384 => {
            let mut hasher = Sha384::new();
            hasher.update(sec1_public_key);
            hasher.finalize().to_vec()
        }
        HashAlgo::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(sec1_public_key);
            hasher.finalize().to_vec()
        }
    }
}

/// Keeps all platform token tag values based on RSS
mod token_tag {
    /* Claims */
    pub const CCA_PLAT_CHALLENGE: u32 = 10;
    pub const CCA_PLAT_INSTANCE_ID: u32 = 256;
    pub const CCA_PLAT_PROFILE: u32 = 265;
    pub const CCA_PLAT_SECURITY_LIFECYCLE: u32 = 2395;
    pub const CCA_PLAT_IMPLEMENTATION_ID: u32 = 2396;
    pub const CCA_PLAT_SW_COMPONENTS: u32 = 2399;
    pub const CCA_PLAT_VERIFICATION_SERVICE: u32 = 2400;
    pub const CCA_PLAT_CONFIGURATION: u32 = 2401;
    pub const CCA_PLAT_HASH_ALGO_DESC: u32 = 2402;

    /* Software components */
    pub const CCA_SW_COMP_TITLE: u32 = 1;
    pub const CCA_SW_COMP_MEASUREMENT_VALUE: u32 = 2;
    pub const CCA_SW_COMP_VERSION: u32 = 4;
    pub const CCA_SW_COMP_SIGNER_ID: u32 = 5;
    pub const CCA_SW_COMP_HASH_ALGORITHM: u32 = 6;
}

impl AttestationMgr {
    const CPAK_SEED_LABEL: &'static [u8] = b"BL1_CPAK_SEED_DERIVATION";
    const DAK_SEED_LABEL: &'static [u8] = b"BL1_DAK_SEED_DERIVATION";

    pub fn calculate_cpak_hash(&self) -> Vec<u8> {
        calculate_public_key_hash(
            self.cpak.key.public_key().to_sec1_bytes().to_vec(),
            HashAlgo::Sha256,
        )
    }

    pub fn calculate_dak_hash(&self, hash_algo: HashAlgo) -> Vec<u8> {
        let dak_info = self.dak.as_ref().unwrap();
        let dak_sec1_bytes = match dak_info.key_bits {
            KeyBits::Bits256 => p256::SecretKey::from_slice(&dak_info.key)
                .unwrap()
                .public_key()
                .to_sec1_bytes()
                .to_vec(),
            KeyBits::Bits384 => p384::SecretKey::from_slice(&dak_info.key)
                .unwrap()
                .public_key()
                .to_sec1_bytes()
                .to_vec(),
            KeyBits::Bits521 => {
                panic!("p521 elliptic curve is not supported");
            }
        };
        calculate_public_key_hash(dak_sec1_bytes, hash_algo)
    }

    fn generate_instance_id(cpak: &p384::SecretKey) -> InstanceId {
        let hash =
            calculate_public_key_hash(cpak.public_key().to_sec1_bytes().to_vec(), HashAlgo::Sha256);

        let mut instance_id: InstanceId = ArrayVec::new();
        instance_id.push(0x01);
        for item in hash {
            instance_id.push(item)
        }
        instance_id
    }

    /// Initialize AttestationMgr using [`KeyMaterialData`] and [`HWClaims`].
    /// Generates a CPAK using `key_material_data`.
    pub fn init(key_derivation_material: KeyMaterialData, claims: HWClaims) -> Self {
        let seed = generate_seed(
            &key_derivation_material.hash,
            &key_derivation_material.guk,
            Self::CPAK_SEED_LABEL,
        );
        let cpak = derive_p384_key(&seed, None);
        Self {
            cpak: CPAK {
                instance_id: Self::generate_instance_id(&cpak).into_iter().collect(),
                key: cpak,
            },
            derivation_material: key_derivation_material,
            claims,
            dak: None,
        }
    }

    /// Unmark DAK as created
    pub fn reset(&mut self) {
        self.dak = None;
    }

    fn encode_measurement(measurement: &Measurement) -> Value {
        let mut map: Vec<(Value, Value)> = Vec::with_capacity(5);

        map.push((
            Value::Integer(token_tag::CCA_SW_COMP_TITLE.into()),
            Value::Text(measurement.metadata.sw_type.clone()),
        ));
        map.push((
            Value::Integer(token_tag::CCA_SW_COMP_HASH_ALGORITHM.into()),
            Value::Text(measurement.metadata.algorithm.into()),
        ));
        map.push((
            Value::Integer(token_tag::CCA_SW_COMP_MEASUREMENT_VALUE.into()),
            Value::Bytes(measurement.value.to_vec()),
        ));
        map.push((
            Value::Integer(token_tag::CCA_SW_COMP_VERSION.into()),
            Value::Text(measurement.metadata.sw_version.clone()),
        ));
        map.push((
            Value::Integer(token_tag::CCA_SW_COMP_SIGNER_ID.into()),
            Value::Bytes(measurement.metadata.signer_id.to_vec()),
        ));

        Value::Map(map)
    }

    fn encode_measurements(measurements: &[Measurement]) -> Value {
        let mut array: Vec<Value> = Vec::with_capacity(measurements.len());

        for measurement in measurements {
            array.push(Self::encode_measurement(measurement));
        }

        Value::Array(array)
    }

    /// Generates DAK with [`ECCFamily`] and uses `measurements` ([`Measurement`])
    /// as salt in the process.
    /// Returns bytes of a scalar primitive, which can be used to recreate DAK Private Key.
    /// [`HashAlgo`] is used for verification process, when `get_platform_token` is called.
    /// Returns [`AttestationError::GenericError`], when CBOR or crypto operation fails.
    pub fn get_delegated_key(
        &mut self,
        _ecc_family: ECCFamily,
        key_bits: KeyBits,
        hash_algo: HashAlgo,
        measurements: &[Measurement],
    ) -> Result<Vec<u8>, AttestationError> {
        let seed = generate_seed(
            &self.derivation_material.hash,
            &self.derivation_material.guk,
            &Self::DAK_SEED_LABEL,
        );

        let salt = Self::encode_measurements(measurements);
        let mut salt_bytes: Vec<u8> = Vec::new();
        ser::into_writer(&salt, &mut salt_bytes).map_err(|_| AttestationError::GenericError)?;

        let dak = match key_bits {
            KeyBits::Bits256 => derive_p256_key(&seed, Some(&salt_bytes))
                .to_bytes()
                .to_vec(),
            KeyBits::Bits384 => derive_p384_key(&seed, Some(&salt_bytes))
                .to_bytes()
                .to_vec(),
            KeyBits::Bits521 => return Err(AttestationError::NotSupported),
        };

        self.dak = Some(DAKInfo {
            hash_algo,
            key_bits,
            key: dak.clone(),
        });

        Ok(dak)
    }

    fn encode_claims(&self, dak_hash: &[u8], measurements: &[Measurement]) -> Value {
        let mut map = Vec::with_capacity(9);

        map.push((
            Value::Integer(token_tag::CCA_PLAT_CHALLENGE.into()),
            Value::Bytes(dak_hash.to_vec()),
        ));
        map.push((
            Value::Integer(token_tag::CCA_PLAT_INSTANCE_ID.into()),
            Value::Bytes(self.cpak.instance_id.to_vec()),
        ));
        if let Some(profile_definition) = &self.claims.profile_definition {
            map.push((
                Value::Integer(token_tag::CCA_PLAT_PROFILE.into()),
                Value::Text(profile_definition.clone()),
            ));
        }

        map.push((
            Value::Integer(token_tag::CCA_PLAT_SECURITY_LIFECYCLE.into()),
            Value::Integer((self.claims.security_lifecycle as u32).into()),
        ));
        map.push((
            Value::Integer(token_tag::CCA_PLAT_IMPLEMENTATION_ID.into()),
            Value::Bytes(self.claims.implementation_id.to_vec()),
        ));
        map.push((
            Value::Integer(token_tag::CCA_PLAT_SW_COMPONENTS.into()),
            Self::encode_measurements(measurements),
        ));
        if let Some(verification_service_url) = &self.claims.verification_service_url {
            map.push((
                Value::Integer(token_tag::CCA_PLAT_VERIFICATION_SERVICE.into()),
                Value::Text(verification_service_url.clone()),
            ));
        }
        map.push((
            Value::Integer(token_tag::CCA_PLAT_CONFIGURATION.into()),
            Value::Bytes(self.claims.platform_config.to_vec()),
        ));
        map.push((
            Value::Integer(token_tag::CCA_PLAT_HASH_ALGO_DESC.into()),
            Value::Text(self.dak.as_ref().unwrap().hash_algo.into()),
        ));
        Value::Map(map)
    }

    fn sign(&self, data: &[u8]) -> Result<Vec<u8>, AttestationError> {
        let signing_key = p384::ecdsa::SigningKey::from_bytes(&self.cpak.key.to_bytes())
            .map_err(|_| AttestationError::GenericError)?;
        let signature: P384Signature = signing_key
            .try_sign(data)
            .map_err(|_| AttestationError::GenericError)?;
        Ok(signature.to_vec())
    }

    fn verify_dak_hash(&self, dak_pub_hash: &[u8]) -> bool {
        let DAKInfo { hash_algo, .. } = self.dak.as_ref().unwrap();
        let calculated_hash = self.calculate_dak_hash(*hash_algo);
        dak_pub_hash == &calculated_hash[..]
    }

    /// Creates a tagged [`CoseSign1`] of the platform token.
    /// `dak_pub_hash` must be a valid hash of DAK Public Key using [`HashAlgo`] passed
    /// in [`AttestationMgr::get_delegated_key`].
    /// Returns [`AttestationError::GenericError`], when CBOR or crypto operation fails.
    /// Returns [`AttestationError::InvalidArgument`], when DAK was not requsted before
    /// this operation, or `dak_pub_hash` is not a valid hash of DAK Public Key.
    pub fn get_platform_token(
        &mut self,
        dak_pub_hash: &[u8],
        measurements: &[Measurement],
    ) -> Result<CoseSign1, AttestationError> {
        if self.dak.is_none() {
            return Err(AttestationError::InvalidArgument);
        }

        if !self.verify_dak_hash(dak_pub_hash) {
            return Err(AttestationError::InvalidArgument);
        }

        let encoded_claims = self.encode_claims(dak_pub_hash, measurements);
        let mut token = Vec::new();
        ser::into_writer(&encoded_claims, &mut token).expect("Unable to encode token");

        let protected = HeaderBuilder::new()
            .algorithm(coset::iana::Algorithm::ES384)
            .build();

        let mut singning_error = None;

        let sign1 = CoseSign1Builder::new()
            .protected(protected)
            .payload(token)
            .create_signature(b"", |payload| match self.sign(payload) {
                Ok(sign) => sign,
                Err(e) => {
                    singning_error = Some(e);
                    Vec::new()
                }
            })
            .build();

        match singning_error {
            None => Ok(sign1),
            Some(e) => Err(e),
        }
    }
}

#[cfg(test)]
mod tests {
    use core::str::from_utf8;

    use alloc::vec;
    use ciborium::de;

    use crate::{MeasurementMetaData, SWType, SWVersion, SignerHash, ValueHash};

    use super::*;

    fn key_derivation_material() -> KeyMaterialData {
        KeyMaterialData {
            hash: [
                0xf1, 0x5f, 0x95, 0x3b, 0xe5, 0x0d, 0xad, 0x92, 0xc3, 0xb2, 0xaa, 0x32, 0x97, 0xe6,
                0xa4, 0xa8, 0xd6, 0x6d, 0x33, 0x63, 0x84, 0x49, 0xec, 0x19, 0x22, 0xb4, 0xa7, 0x92,
                0x4a, 0x7b, 0x30, 0x22,
            ]
            .iter()
            .cloned()
            .collect(),
            guk: [
                0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45, 0x67,
                0x89, 0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45, 0x67, 0x89, 0x01, 0x23, 0x45,
                0x67, 0x89, 0x01, 0x23,
            ]
            .into(),
        }
    }

    fn hw_claims() -> HWClaims {
        HWClaims {
            implementation_id: [
                0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xAA, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB, 0xBB,
                0xBB, 0xBB, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xCC, 0xDD, 0xDD, 0xDD, 0xDD,
                0xDD, 0xDD, 0xDD, 0xDD,
            ],
            security_lifecycle: 0x4000,
            profile_definition: Some("http://arm.com/CCA-SSD/1.0.0".to_string()),
            verification_service_url: Some("http://whatever.com".to_string()),
            platform_config: 0xDEADBEEFu32.to_ne_bytes().iter().cloned().collect(),
        }
    }

    fn measurements() -> Vec<Measurement> {
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

        vec![Measurement {
            metadata: MeasurementMetaData {
                signer_id,
                sw_version,
                algorithm: MeasurementType::Sha512,
                sw_type,
            },
            value: ValueHash::from([
                0x8a, 0x66, 0x01, 0xf6, 0x70, 0x74, 0x8b, 0xe2, 0x33, 0xff, 0x5d, 0x75, 0xd7, 0xea,
                0x89, 0xa8, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0x01, 0x05, 0x01, 0xEF,
                0x68, 0x07, 0x88, 0xCC, 0x83, 0x09, 0x22, 0xCD, 0x09, 0x61, 0xB6, 0xFF, 0xbb, 0xbb,
                0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0xbb, 0x56, 0x46, 0x58, 0x49, 0x99, 0x31, 0xcf, 0x59,
                0x7d, 0xbc, 0x3a, 0x4e, 0x68, 0x79, 0x8a, 0x1c,
            ]),
        }]
    }

    #[test]
    fn token_before_dak_error() {
        let mut mgr = AttestationMgr::init(key_derivation_material(), hw_claims());

        assert_eq!(
            mgr.get_platform_token(&[], &[]).unwrap_err(),
            AttestationError::InvalidArgument
        );
    }

    #[test]
    fn token_bad_dak_hash_value() {
        let mut mgr = AttestationMgr::init(key_derivation_material(), hw_claims());

        let _dak = mgr
            .get_delegated_key(ECCFamily::SecpR1, KeyBits::Bits384, HashAlgo::Sha256, &[])
            .unwrap();

        assert_eq!(
            mgr.get_platform_token(&[], &[]).unwrap_err(),
            AttestationError::InvalidArgument
        );
    }

    #[test]
    fn token_bad_dak_hash_algo() {
        let mut mgr = AttestationMgr::init(key_derivation_material(), hw_claims());

        let dak = mgr
            .get_delegated_key(ECCFamily::SecpR1, KeyBits::Bits384, HashAlgo::Sha256, &[])
            .unwrap();

        let hash = calculate_public_key_hash(
            p384::SecretKey::from_slice(&dak)
                .unwrap()
                .public_key()
                .to_sec1_bytes()
                .to_vec(),
            HashAlgo::Sha512,
        );

        assert_eq!(
            mgr.get_platform_token(&hash, &[]).unwrap_err(),
            AttestationError::InvalidArgument
        );
    }

    #[test]
    fn token_sign1_verify() {
        let key_derivation_material = key_derivation_material();
        let claims = hw_claims();
        let boot_measurements = measurements();
        let hash_algo = HashAlgo::Sha256;

        let mut mgr = AttestationMgr::init(key_derivation_material.clone(), claims.clone());
        let dak = mgr
            .get_delegated_key(
                ECCFamily::SecpR1,
                KeyBits::Bits384,
                hash_algo,
                &boot_measurements,
            )
            .unwrap();

        let hash = calculate_public_key_hash(
            p384::SecretKey::from_slice(&dak)
                .unwrap()
                .public_key()
                .to_sec1_bytes()
                .to_vec(),
            hash_algo,
        );

        let token_sign1 = mgr.get_platform_token(&hash, &boot_measurements).unwrap();

        assert!(token_sign1
            .verify_signature(&[], |sig, data| {
                (mgr.sign(data).unwrap() == sig).then_some(()).ok_or(())
            })
            .is_ok());
    }

    fn claims_occurence_vector() -> Vec<(u32, bool)> {
        let occurence_vec = vec![
            (token_tag::CCA_PLAT_CHALLENGE, false),
            (token_tag::CCA_PLAT_INSTANCE_ID, false),
            (token_tag::CCA_PLAT_SECURITY_LIFECYCLE, false),
            (token_tag::CCA_PLAT_IMPLEMENTATION_ID, false),
            (token_tag::CCA_PLAT_SW_COMPONENTS, false),
            (token_tag::CCA_PLAT_PROFILE, false),
            (token_tag::CCA_PLAT_VERIFICATION_SERVICE, false),
            (token_tag::CCA_PLAT_CONFIGURATION, false),
            (token_tag::CCA_PLAT_HASH_ALGO_DESC, false),
        ];

        assert_eq!(occurence_vec.len(), 9);

        occurence_vec
    }

    fn mark_occurence(vec: &mut Vec<(u32, bool)>, occured_tag: u32) {
        let (_, tag_occurence) = vec
            .iter_mut()
            .find(|&&mut (tag, _)| tag == occured_tag)
            .unwrap();
        *tag_occurence = true;
    }

    fn verify_occurence(vec: &Vec<(u32, bool)>) -> bool {
        !vec.iter()
            .find(|&&(_, occurence)| occurence == false)
            .is_some()
    }

    #[test]
    fn token_decode_verify() {
        let key_derivation_material = key_derivation_material();
        let claims = hw_claims();
        let measurements = measurements();
        let hash_algo = HashAlgo::Sha256;

        let mut mgr = AttestationMgr::init(key_derivation_material.clone(), claims.clone());
        let dak = mgr
            .get_delegated_key(
                ECCFamily::SecpR1,
                KeyBits::Bits384,
                hash_algo,
                &measurements,
            )
            .unwrap();

        let hash = calculate_public_key_hash(
            p384::SecretKey::from_slice(&dak)
                .unwrap()
                .public_key()
                .to_sec1_bytes()
                .to_vec(),
            hash_algo,
        );

        let token_sign1 = mgr.get_platform_token(&hash, &measurements).unwrap();
        let payload = de::from_reader(&token_sign1.payload.unwrap()[..])
            .expect("CoseSign1 is not a cbor Value");

        let token_map = match payload {
            Value::Map(platform_token_map) => platform_token_map,
            _ => panic!("CoseSign1 payload is not a map!"),
        };

        let mut occurence_vec = claims_occurence_vector();

        for (tag_value, value) in token_map {
            let tag: u32 = match tag_value {
                Value::Integer(tag_integer) => tag_integer.try_into().unwrap(),
                _ => panic!("Tag is incorrect"),
            };

            match tag {
                token_tag::CCA_PLAT_CHALLENGE => {
                    if let Value::Bytes(dak_hash) = value {
                        assert_eq!(dak_hash, hash);
                    } else {
                        panic!("CCA_PLAT_CHALLENGE incorrect");
                    }
                }
                token_tag::CCA_PLAT_CONFIGURATION => {
                    if let Value::Bytes(platform_config) = value {
                        assert_eq!(&platform_config[..], &claims.platform_config[..]);
                    } else {
                        panic!("CCA_PLAT_CONFIGURATION incorrect");
                    }
                }
                token_tag::CCA_PLAT_INSTANCE_ID => {
                    if let Value::Bytes(instance_id) = value {
                        assert_eq!(instance_id[0], 0x01);
                        assert_eq!(
                            instance_id[1..],
                            calculate_public_key_hash(
                                mgr.cpak.key.public_key().to_sec1_bytes().to_vec(),
                                HashAlgo::Sha256
                            )
                        );
                    } else {
                        panic!("CCA_PLAT_INSTANCE_ID incorrect");
                    }
                }
                token_tag::CCA_PLAT_PROFILE => {
                    if let Value::Text(profile) = value {
                        assert_eq!(profile, *claims.profile_definition.as_ref().unwrap());
                    } else {
                        panic!("CCA_PLAT_PROFILE incorrect");
                    }
                }
                token_tag::CCA_PLAT_SECURITY_LIFECYCLE => {
                    if let Value::Integer(security_lifecycle) = value {
                        let value_u32: u32 = security_lifecycle.try_into().unwrap();
                        assert_eq!(value_u32, claims.security_lifecycle as u32);
                    } else {
                        panic!("CCA_PLAT_PROFILE incorrect");
                    }
                }
                token_tag::CCA_PLAT_IMPLEMENTATION_ID => {
                    if let Value::Bytes(implementation_id) = value {
                        assert_eq!(implementation_id, &claims.implementation_id);
                    } else {
                        panic!("CCA_PLAT_IMPLEMENTATION_ID incorrect");
                    }
                }
                token_tag::CCA_PLAT_HASH_ALGO_DESC => {
                    if let Value::Text(algo_desc) = value {
                        let claim_desc: String = mgr.dak.as_ref().unwrap().hash_algo.into();
                        assert_eq!(algo_desc, claim_desc);
                    } else {
                        panic!("CCA_PLAT_HASH_ALGO_DESC incorrect");
                    }
                }
                token_tag::CCA_PLAT_VERIFICATION_SERVICE => {
                    if let Value::Text(verficiation_service) = value {
                        assert_eq!(
                            verficiation_service,
                            *claims.verification_service_url.as_ref().unwrap()
                        );
                    }
                }
                token_tag::CCA_PLAT_SW_COMPONENTS => {
                    if let Value::Array(measurements_array) = value {
                        // This code works only for measurements with one entry
                        for measurement_map in measurements_array {
                            if let Value::Map(measurement) = measurement_map {
                                for (tag_value, value) in measurement {
                                    let tag: u32 = match tag_value {
                                        Value::Integer(tag_integer) => {
                                            tag_integer.try_into().unwrap()
                                        }
                                        _ => panic!("SW COMP map tag not an Integer"),
                                    };
                                    match tag {
                                        token_tag::CCA_SW_COMP_HASH_ALGORITHM => {
                                            if let Value::Text(algorithm) = value {
                                                let claim_desc: String =
                                                    measurements[0].metadata.algorithm.into();
                                                assert_eq!(algorithm, claim_desc);
                                            } else {
                                                panic!("CCA_SW_COMP_HASH_ALGORITHM incorrect");
                                            }
                                        }
                                        token_tag::CCA_SW_COMP_TITLE => {
                                            if let Value::Text(title) = value {
                                                assert_eq!(title, measurements[0].metadata.sw_type);
                                            } else {
                                                panic!("CCA_SW_COMP_TITLE incorrect");
                                            }
                                        }
                                        token_tag::CCA_SW_COMP_SIGNER_ID => {
                                            if let Value::Bytes(signer_id) = value {
                                                assert_eq!(
                                                    &signer_id[..],
                                                    &measurements[0].metadata.signer_id[..]
                                                );
                                            } else {
                                                panic!("CCA_SW_COMP_SINGER_ID incorrect");
                                            }
                                        }
                                        token_tag::CCA_SW_COMP_VERSION => {
                                            if let Value::Text(version) = value {
                                                assert_eq!(
                                                    version,
                                                    measurements[0].metadata.sw_version
                                                );
                                            } else {
                                                panic!("CCA_SW_COMP_VERSION incorrect");
                                            }
                                        }
                                        token_tag::CCA_SW_COMP_MEASUREMENT_VALUE => {
                                            if let Value::Bytes(measurement_value) = value {
                                                assert_eq!(
                                                    measurement_value,
                                                    &measurements[0].value[..]
                                                );
                                            } else {
                                                panic!("CCA_SW_COMP_MEASUREMENT_VALUE incorrect");
                                            }
                                        }
                                        _ => panic!("Invalid SW COMP tag value: {}", tag),
                                    }
                                }
                            }
                        }
                    }
                }
                _ => panic!("Invalid tag value: {}", tag),
            }
            mark_occurence(&mut occurence_vec, tag);
        }

        assert!(verify_occurence(&occurence_vec));
    }
}
