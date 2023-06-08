mod verifier;

use self::verifier::Verifier;

use ciborium::{de, value::Value};
use coset::{AsCborValue, CoseSign1};

pub fn verify(object: &[u8], key: &[u8], aad: &[u8]) -> Result<(), &'static str> {
    let cose = parse(object)?;
    if cose.protected.header.alg.is_none() {
        return Err("Invalid algorithm");
    }
    let alg = cose
        .protected
        .header
        .alg
        .as_ref()
        .unwrap()
        .clone()
        .try_into()
        .or(Err("Failed to get algorithm."))?;
    let verifier = Verifier::new(alg, &key);
    cose.verify_signature(aad, |sig, data| verifier.verify(sig, data))
}

#[derive(Debug)]
pub(crate) enum Algorithm {
    // sha256 + secp256r1/prime256v1/P-256
    ES256,
    // sha384 + secp384r1/P-384
    ES384,
    // sha512 + secp521r1/P-521
    ES512,
}

impl TryFrom<coset::Algorithm> for Algorithm {
    type Error = &'static str;

    fn try_from(alg: coset::Algorithm) -> Result<Self, Self::Error> {
        match alg {
            coset::Algorithm::Assigned(coset::iana::Algorithm::ES256) => Ok(Algorithm::ES256),
            coset::Algorithm::Assigned(coset::iana::Algorithm::ES384) => Ok(Algorithm::ES384),
            coset::Algorithm::Assigned(coset::iana::Algorithm::ES512) => Ok(Algorithm::ES512),
            _ => Err("Unknowun algorithm"),
        }
    }
}

fn parse(object: &[u8]) -> Result<CoseSign1, &'static str> {
    const TAG_COSE_SIGN1: u64 = 18;
    if let Value::Tag(tag, data) = de::from_reader(object).or(Err("Invalid Format"))? {
        if tag != TAG_COSE_SIGN1 {
            return Err("Invalid Tag");
        }
        Ok(CoseSign1::from_cbor_value(*data).or(Err("Invalid Format"))?)
    } else {
        return Err("Invalid Format");
    }
}
