use crate::error::Error;

use openssl::rand::rand_bytes;
use openssl::symm::{decrypt_aead, encrypt_aead, Cipher};
use serde::{Deserialize, Serialize};
use zeroize::Zeroizing;

// These seal and unseal functions are implemented similarly as in the VMWare's Certifier Framework.
// Here are the main assumptions:
// - The requested symmetric Sealing Key is bound to the platform, firmware and the Realm Initial Measurement (RIM).
//   Thus, any change of the platform, firmware, or a realm image will result in a different key.
// - AES-256-GCM is used as an encryption algorithm; the AAD is not set
// - The plaintext is encrypted and put into a sealed data structure. This strucure is comprised of
//   the header and the ciphertext, the header contains the IV and the authentication TAG.
//   The whole structure is serialized using serde and bincode crates to produce binary object, that then can
//   be saved in a file on the host side.

// We take VHUK_M (Measurement based Virtual Hardware Unique Key) and RIM
// as a key material during the sealing key derivation process
#[cfg(target_arch = "aarch64")]
const UNIQUE_SEALING_KEY: u64 =
    rust_rsi::RSI_SEALING_KEY_FLAGS_KEY | rust_rsi::RSI_SEALING_KEY_FLAGS_RIM;

const AES_GCM_256_IV_LEN: usize = 12;
const AES_GCM_256_TAG_LEN: usize = 16;
const SEALING_KEY_LEN: usize = 32;

// An embedded sealing key used on the simulated platform
#[cfg(target_arch = "x86_64")]
const SEALING_KEY: [u8; SEALING_KEY_LEN] = [
    0x63, 0x10, 0xc1, 0xf0, 0x53, 0xd5, 0x52, 0x40, 0x29, 0xfa, 0x7f, 0x7d, 0xcd, 0x9e, 0x28, 0x2c,
    0x4a, 0x93, 0x9d, 0x55, 0xb9, 0x89, 0x15, 0x44, 0x45, 0xa3, 0x86, 0x1e, 0x1f, 0xa1, 0xe2, 0xce,
];

#[derive(Serialize, Deserialize)]
struct Header {
    tag: [u8; AES_GCM_256_TAG_LEN],
    iv: [u8; AES_GCM_256_IV_LEN],
}

impl Header {
    fn new() -> Result<Self, Error> {
        let mut instance = Self {
            tag: [0u8; AES_GCM_256_TAG_LEN],
            iv: [0u8; AES_GCM_256_IV_LEN],
        };
        rand_bytes(&mut instance.iv).map_err(|_| Error::Sealing)?;
        Ok(instance)
    }
}

#[derive(Serialize, Deserialize)]
struct SealedData {
    header: Header,
    ciphertext: Vec<u8>,
}

fn sealing_key() -> Result<[u8; SEALING_KEY_LEN], Error> {
    cfg_if::cfg_if! {
        // Return the embedded sealing key for simulated platform
        if #[cfg(target_arch="x86_64")] {
            Ok(SEALING_KEY)
        } else {
            rust_rsi::sealing_key(UNIQUE_SEALING_KEY, 0).or(Err(Error::SealingKey))
        }
    }
}

pub fn seal(plaintext: &[u8]) -> Result<Vec<u8>, Error> {
    let mut header = Header::new()?;
    let cipher = Cipher::aes_256_gcm();
    let sealing_key = Zeroizing::new(sealing_key().map_err(|_| Error::SealingKey)?);

    let enc_res = encrypt_aead(
        cipher,
        sealing_key.as_ref(),
        Some(&header.iv),
        &[],
        plaintext,
        &mut header.tag,
    );

    let sealed_data = SealedData {
        header: header,
        ciphertext: enc_res.map_err(|_| Error::Sealing)?,
    };

    bincode::serialize(&sealed_data).map_err(|_| Error::Sealing)
}

pub fn unseal(sealed: &[u8]) -> Result<Vec<u8>, Error> {
    let sealed_data: SealedData = bincode::deserialize(sealed).map_err(|_| Error::Sealing)?;
    let cipher = Cipher::aes_256_gcm();
    let sealing_key = Zeroizing::new(sealing_key().map_err(|_| Error::SealingKey)?);

    let dec_res = decrypt_aead(
        cipher,
        sealing_key.as_ref(),
        Some(&sealed_data.header.iv),
        &[],
        &sealed_data.ciphertext,
        &sealed_data.header.tag,
    );

    dec_res.map_err(|_| Error::Sealing)
}
