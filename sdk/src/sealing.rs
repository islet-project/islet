use crate::error::Error;

use rand::rngs::ThreadRng;
use rsa::pkcs1::DecodeRsaPrivateKey;
use rsa::{Pkcs1v15Encrypt, RsaPrivateKey};

pub const DEBUG_KEY: &[u8] = include_bytes!("../debug/rsa2048-priv.der");

pub fn seal(plaintext: &[u8]) -> Result<Vec<u8>, Error> {
    let pri_key = RsaPrivateKey::from_pkcs1_der(DEBUG_KEY).or(Err(Error::SealingKey))?;
    let pub_key = pri_key.to_public_key();
    let mut rng: ThreadRng = Default::default();
    let sealed = pub_key
        .encrypt(&mut rng, Pkcs1v15Encrypt, &plaintext[..])
        .or(Err(Error::Sealing))?;
    Ok(sealed)
}

pub fn unseal(sealed: &[u8]) -> Result<Vec<u8>, Error> {
    let pri_key = RsaPrivateKey::from_pkcs1_der(DEBUG_KEY).or(Err(Error::SealingKey))?;
    let dec_plaintext = pri_key
        .decrypt(Pkcs1v15Encrypt, &sealed)
        .or(Err(Error::Sealing))?;
    Ok(dec_plaintext)
}
