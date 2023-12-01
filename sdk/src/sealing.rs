use crate::error::Error;

use openssl::rsa::Padding;
use openssl::rsa::Rsa;

pub const DEBUG_KEY: &[u8] = include_bytes!("../debug/rsa2048-priv.der");

pub fn seal(plaintext: &[u8]) -> Result<Vec<u8>, Error> {
    let pri_key = Rsa::private_key_from_der(DEBUG_KEY).or(Err(Error::SealingKey))?;
    let padding = Padding::PKCS1;
    let len = core::cmp::max(plaintext.len(), pri_key.size() as usize);
    let mut sealed = vec![0 as u8; len];
    let _ = pri_key
        .public_encrypt(plaintext, &mut sealed, padding)
        .or(Err(Error::Sealing))?;
    Ok(sealed)
}

pub fn unseal(sealed: &[u8]) -> Result<Vec<u8>, Error> {
    let pri_key = Rsa::private_key_from_der(DEBUG_KEY).or(Err(Error::SealingKey))?;
    let padding = Padding::PKCS1;
    let len = core::cmp::max(sealed.len(), pri_key.size() as usize);
    let mut dec_plaintext = vec![0 as u8; len];
    let _ = pri_key
        .private_decrypt(sealed, &mut dec_plaintext, padding)
        .or(Err(Error::SealingKey))?;
    Ok(dec_plaintext)
}
