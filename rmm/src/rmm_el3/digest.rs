use alloc::vec::Vec;
use sha2::{Digest, Sha256, Sha384, Sha512};

/// Supported public dak hash algorithms.
#[allow(dead_code)]
#[derive(Debug)]
enum HashAlgo {
    Sha256,
    Sha384,
    Sha512,
}

fn calculate_hash(data: Vec<u8>, algo: HashAlgo) -> Vec<u8> {
    match algo {
        HashAlgo::Sha256 => {
            let mut hasher = Sha256::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
        HashAlgo::Sha384 => {
            let mut hasher = Sha384::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
        HashAlgo::Sha512 => {
            let mut hasher = Sha512::new();
            hasher.update(data);
            hasher.finalize().to_vec()
        }
    }
}

pub(super) fn get_realm_public_key_hash(key: Vec<u8>) -> Vec<u8> {
    let priv_dak = p384::SecretKey::from_slice(&key).unwrap();

    calculate_hash(
        priv_dak.public_key().to_sec1_bytes().to_vec(),
        HashAlgo::Sha256,
    )
}
