#![no_std]

extern crate alloc;

use aes::cipher::generic_array::GenericArray;
use aes::cipher::{BlockEncrypt, KeyInit};
use aes::Aes256;
use alloc::{vec, vec::Vec};
use core::mem;
use elliptic_curve::ScalarPrimitive;
use p256::U256;
use p384::elliptic_curve::Curve;
use p384::U384;
use sha2::{Digest, Sha256};

/// Derives key material from a symmetric key, some label and input data.
/// Key material is a derived 256-bit symmetric key using counter-mode KDF complying
/// with NIST SP800-108, where the PRF is a combined sha256 hash and an ECB-mode
/// AES encryption.
///
/// # Arguments
///
/// * `input` - A binary data
/// * `input_key` - A 256-bit symmetric input derivation key
/// * `label` - Unique label describing seed purpose
///
pub fn generate_seed(input: &[u8], input_key: &[u8], label: &[u8]) -> Vec<u8> {
    let mut context = vec![0; input.len() + mem::size_of::<u32>() * 2];
    let mut state =
        vec![0; context.len() + label.len() + mem::size_of::<u8>() + mem::size_of::<u32>() * 2];
    let lcs: u32 = 3;
    let reprovisioning_bits: u32 = 0;
    let block_index: u32 = 1;
    let seed_output_length: u32 = 32;

    context[..input.len()].copy_from_slice(&input);
    context[input.len()..input.len() + mem::size_of::<u32>()].copy_from_slice(&lcs.to_ne_bytes());
    context[input.len() + mem::size_of::<u32>()..]
        .copy_from_slice(&reprovisioning_bits.to_ne_bytes());

    state[mem::size_of::<u32>()..mem::size_of::<u32>() + label.len()].copy_from_slice(&label);
    state[mem::size_of::<u32>() + label.len()
        ..mem::size_of::<u32>() + label.len() + mem::size_of::<u8>()]
        .copy_from_slice(&0u8.to_ne_bytes());
    state[mem::size_of::<u32>() + label.len() + mem::size_of::<u8>()
        ..mem::size_of::<u32>() + label.len() + mem::size_of::<u8>() + context.len()]
        .copy_from_slice(&context);
    state[mem::size_of::<u32>() + label.len() + mem::size_of::<u8>() + context.len()..]
        .copy_from_slice(&seed_output_length.to_ne_bytes());

    state[..mem::size_of::<u32>()].copy_from_slice(&block_index.to_ne_bytes());

    let mut hasher = Sha256::new();
    hasher.update(&state);
    let state_hash = hasher.finalize();

    let mut seed_buffer = state_hash.to_vec();

    let cipher = Aes256::new_from_slice(&input_key).unwrap();
    let mut block = GenericArray::from_mut_slice(&mut seed_buffer[..16]);

    cipher.encrypt_block(&mut block);

    block = GenericArray::from_mut_slice(&mut seed_buffer[16..]);
    cipher.encrypt_block(&mut block);

    seed_buffer
}

/// Derives a Secp256r1 public key using HKDF(Sha256) on a given key material.
///
/// # Arguments
///
/// * `seed` - input secret key
/// * `info` - info string used in expand step
///
pub fn derive_p256_key(seed: &[u8], info: Option<&[u8]>) -> p256::SecretKey {
    let n = p256::NistP256::ORDER;
    let bits = n.bits();
    let bytes = (bits + 7) / 8;
    let n_2 = n.saturating_sub(&U256::from_u32(2));

    // RSS never passes salt. And what passes named as salt in reality is an info.
    let hk = hkdf::Hkdf::<Sha256>::new(None, seed);

    let mut okm = vec![0; bytes];

    let mut k;

    loop {
        hk.expand(
            match info {
                Some(i) => i,
                None => &[],
            },
            &mut okm,
        )
        .expect("hkdf could not expand to 42 bytes");

        k = U256::from_be_slice(&okm);

        if k <= n_2 {
            break;
        }
    }

    let private_key_scalar = k.saturating_add(&U256::from_u32(1));

    p256::SecretKey::new(ScalarPrimitive::new(private_key_scalar).unwrap())
}

/// Derives a Secp384r1 public key using HKDF(Sha256) on a given key material.
///
/// # Arguments
///
/// * `seed` - input secret key
/// * `info` - info string used in expand step
///
pub fn derive_p384_key(seed: &[u8], info: Option<&[u8]>) -> p384::SecretKey {
    let n = p384::NistP384::ORDER;
    let bits = n.bits();
    let bytes = (bits + 7) / 8;
    let n_2 = n.saturating_sub(&U384::from_u32(2));

    // RSS never passes salt. And what passes named as salt in reality is an info.
    let hk = hkdf::Hkdf::<Sha256>::new(None, seed);

    let mut okm = vec![0; bytes];

    let mut k;

    loop {
        hk.expand(
            match info {
                Some(i) => i,
                None => &[],
            },
            &mut okm,
        )
        .expect("hkdf could not expand to 42 bytes");

        k = U384::from_be_slice(&okm);

        if k <= n_2 {
            break;
        }
    }

    let private_key_scalar = k.saturating_add(&U384::from_u32(1));

    p384::SecretKey::new(ScalarPrimitive::new(private_key_scalar).unwrap())
}
