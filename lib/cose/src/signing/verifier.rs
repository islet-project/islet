use super::Algorithm;

use std::str::FromStr;

pub struct Verifier {
    algorithm: Algorithm,
    key_public_raw: Vec<u8>,
}

impl Verifier {
    pub(crate) fn new(algorithm: Algorithm, key_public: &[u8]) -> Self {
        Self {
            algorithm,
            key_public_raw: key_public.to_vec(),
        }
    }

    pub fn verify(&self, sig: &[u8], data: &[u8]) -> Result<(), &'static str> {
        println!("== Signature Verification:");
        let sig_hex = hex::encode(sig);
        println!(
            "Sign Algo\t = [{:?}]\nPublic Key\t = [{:?}]\nData\t\t = [{:?}]\nSignature\t = [{:?}]",
            self.algorithm,
            hex::encode(&self.key_public_raw),
            hex::encode(data),
            sig_hex
        );
        match self.algorithm {
            Algorithm::ES256 => {
                use p256::ecdsa::signature::Verifier;
                let key = p256::ecdsa::VerifyingKey::from_sec1_bytes(&self.key_public_raw)
                    .or(Err("Invalid Key"))?;
                let sig =
                    p256::ecdsa::Signature::from_str(&sig_hex).or(Err("Invalid Signature"))?;
                key.verify(data, &sig).or(Err("Failed to veirify"))?;
            }
            Algorithm::ES384 => {
                use p384::ecdsa::signature::Verifier;
                let key = p384::ecdsa::VerifyingKey::from_sec1_bytes(&self.key_public_raw)
                    .or(Err("Invalid Key"))?;
                let sig =
                    p384::ecdsa::Signature::from_str(&sig_hex).or(Err("Invalid Signature"))?;
                key.verify(data, &sig).or(Err("Failed to veirify"))?;
            }
            Algorithm::ES512 => {
                // p521 from  cannot do ecdsa
                return Err("ES512 is not supported.");
            }
        }
        println!("== End of Signature Verification\n");
        Ok(())
    }
}
