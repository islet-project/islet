use core::ffi::CStr;
use p384::{
    ecdsa::{signature::Verifier, Signature, VerifyingKey},
    elliptic_curve::generic_array::GenericArray,
    EncodedPoint,
};

use super::{HASH_ALGO_SHA256, HASH_ALGO_SHA512};
use crate::granule::GRANULE_SIZE;
use crate::measurement::{Measurement, MEASUREMENTS_SLOT_MAX_SIZE};
use crate::rmi::error::Error;

const FMT_VERSION: usize = 1;
pub const REALM_ID_SIZE: usize = 128;
pub const P384_PUBLIC_KEY_SIZE: usize = 96;
const P384_SIGNATURE_SIZE: usize = P384_PUBLIC_KEY_SIZE;

#[allow(dead_code)]
const P385_SIGNATURE_POINT_SIZE: usize = P384_SIGNATURE_SIZE / 2;
#[allow(dead_code)]
const SHA_384_HASH_SIZE: usize = 48;

const METADATA_HASH_SHA_256: usize = 0x01;
const METADATA_HASH_SHA_512: usize = 0x02;

const REALM_METADATA_HEADER_SIZE: usize = 0x150;
#[allow(dead_code)]
const REALM_METADATA_SIGNED_SIZE: usize = 0x1B0;
const REALM_METADATA_UNUSED_SIZE: usize = 0xE50;

#[derive(Debug, Clone, Copy)]
#[repr(C)]
pub struct IsletRealmMetadata {
    fmt_version: usize,
    realm_id: [u8; REALM_ID_SIZE],
    rim: [u8; MEASUREMENTS_SLOT_MAX_SIZE],
    hash_algo: usize,
    svn: usize,
    version_major: usize,
    version_minor: usize,
    version_patch: usize,
    public_key: [u8; P384_PUBLIC_KEY_SIZE],
    signature: [u8; P384_SIGNATURE_SIZE],
    _unused: [u8; REALM_METADATA_UNUSED_SIZE],
}

const _: () = assert!(core::mem::size_of::<IsletRealmMetadata>() == GRANULE_SIZE);
const _: () = assert!(core::mem::size_of::<IsletRealmMetadata>() >= REALM_METADATA_SIGNED_SIZE);

const _: () = assert!(core::mem::offset_of!(IsletRealmMetadata, fmt_version) == 0x00);
const _: () = assert!(core::mem::offset_of!(IsletRealmMetadata, realm_id) == 0x08);
const _: () = assert!(core::mem::offset_of!(IsletRealmMetadata, rim) == 0x88);
const _: () = assert!(core::mem::offset_of!(IsletRealmMetadata, hash_algo) == 0xc8);
const _: () = assert!(core::mem::offset_of!(IsletRealmMetadata, svn) == 0xd0);
const _: () = assert!(core::mem::offset_of!(IsletRealmMetadata, version_major) == 0xd8);
const _: () = assert!(core::mem::offset_of!(IsletRealmMetadata, version_minor) == 0xe0);
const _: () = assert!(core::mem::offset_of!(IsletRealmMetadata, version_patch) == 0xe8);
const _: () = assert!(core::mem::offset_of!(IsletRealmMetadata, public_key) == 0xf0);

impl IsletRealmMetadata {
    fn realm_id_as_str(&self) -> Option<&str> {
        let Ok(cstr) = CStr::from_bytes_until_nul(&self.realm_id) else {
            return None;
        };
        let Ok(s) = cstr.to_str() else {
            return None;
        };
        Some(s)
    }

    pub fn dump(&self) {
        debug!("fmt_version: {:#010x}", self.fmt_version);
        debug!(
            "realm_id: {}",
            self.realm_id_as_str().unwrap_or("INVALID REALM ID")
        );
        debug!("rim: {}", hex::encode(self.rim));
        debug!("hash_algo: {:#010x}", self.hash_algo);
        debug!("svn: {:#010x}", self.svn);
        debug!("version_major: {:#010x}", self.version_major);
        debug!("version_minor: {:#010x}", self.version_minor);
        debug!("version_patch: {:#010x}", self.version_patch);
        debug!("public_key: {}", hex::encode(self.public_key));
        debug!("signature: {}", hex::encode(self.signature));
    }

    fn verifying_key(&self) -> core::result::Result<VerifyingKey, Error> {
        let point = EncodedPoint::from_untagged_bytes(GenericArray::from_slice(&self.public_key));
        VerifyingKey::from_encoded_point(&point).or(Err(Error::RmiErrorInput))
    }

    fn signature(&self) -> core::result::Result<Signature, Error> {
        Signature::from_slice(&self.signature).or(Err(Error::RmiErrorInput))
    }

    fn header_as_u8_slice(&self) -> &[u8] {
        let slice = unsafe {
            core::slice::from_raw_parts(
                (self as *const Self) as *const u8,
                core::mem::size_of::<Self>(),
            )
        };
        &slice[..REALM_METADATA_HEADER_SIZE]
    }

    pub fn verify_signature(&self) -> core::result::Result<(), Error> {
        let verifying_key = self.verifying_key()?;
        let signature = self.signature()?;
        let data = self.header_as_u8_slice();

        verifying_key
            .verify(data, &signature)
            .or(Err(Error::RmiErrorInput))
    }

    pub fn validate(&self) -> core::result::Result<(), Error> {
        if self.fmt_version != FMT_VERSION {
            error!(
                "Metadata format version {} is not supported!",
                self.fmt_version
            );
            Err(Error::RmiErrorInput)?
        }

        if self.svn == 0 {
            error!("SVN number should be greater than zero");
            Err(Error::RmiErrorInput)?
        }

        if ![METADATA_HASH_SHA_256, METADATA_HASH_SHA_512].contains(&self.hash_algo) {
            error!("Hash algorithm is invalid {}", self.hash_algo);
            Err(Error::RmiErrorInput)?
        }

        let is_printable_ascii = |&c| c >= b' ' && c <= b'~';

        if !self
            .realm_id
            .iter()
            .take_while(|&c| *c != b'\0')
            .all(is_printable_ascii)
        {
            error!("Realm id is invalid");
            Err(Error::RmiErrorInput)?
        }

        Ok(())
    }

    pub fn equal_rd_rim(&self, rim: &Measurement) -> bool {
        rim.as_slice() == self.rim
    }

    pub fn equal_rd_hash_algo(&self, hash_algo: u8) -> bool {
        let converted_algo = match hash_algo {
            HASH_ALGO_SHA256 => METADATA_HASH_SHA_256,
            HASH_ALGO_SHA512 => METADATA_HASH_SHA_512,
            _ => unreachable!(),
        };

        converted_algo == self.hash_algo
    }

    // for sealing key derivation

    pub fn svn(&self) -> usize {
        self.svn
    }

    pub fn public_key(&self) -> &[u8; P384_PUBLIC_KEY_SIZE] {
        &self.public_key
    }

    pub fn realm_id(&self) -> &[u8; REALM_ID_SIZE] {
        &self.realm_id
    }
}

// This should work but I don't like this traits
impl vmsa::guard::Content for IsletRealmMetadata {}
impl safe_abstraction::raw_ptr::RawPtr for IsletRealmMetadata {}
impl safe_abstraction::raw_ptr::SafetyChecked for IsletRealmMetadata {}
impl safe_abstraction::raw_ptr::SafetyAssured for IsletRealmMetadata {
    fn is_initialized(&self) -> bool {
        true
    }

    fn verify_ownership(&self) -> bool {
        true
    }
}
