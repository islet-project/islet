#![deny(warnings)]
#![feature(vec_into_raw_parts)]
#![warn(rust_2018_idioms)]

pub mod attester;
pub mod c_api;
pub mod error;
pub mod prelude;
pub mod report;
pub mod sealing;
pub mod verifier;

#[cfg(target_arch = "x86_64")]
mod mock;
mod parser;

cfg_if::cfg_if! {
    if #[cfg(target_arch = "x86_64")] {
        pub struct AttestationClaims {
            pub origin: rust_rsi::AttestationClaims,
            pub user_data: Vec<u8>, // The requirement of Certifier: Simulated Version on x86
        }
    } else {
        pub use rust_rsi::AttestationClaims;
    }
}

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn attest_verify() {
        let user_data = b"User data";
        let report = attest(user_data).unwrap();
        let claims = verify(&report).unwrap();
        let (realm_claims, plat_claims) = parse(&claims).unwrap();
        assert_eq!(user_data, &realm_claims.challenge[..user_data.len()]);
        assert_eq!("http://arm.com/CCA-SSD/1.0.0", plat_claims.profile);
    }

    #[test]
    fn sealing() {
        use super::sealing::{seal, unseal};
        let plaintext = b"Plaintext";
        let sealed = seal(plaintext).unwrap();
        let unsealed = unseal(&sealed).unwrap();
        assert_eq!(plaintext, &unsealed[..]);
    }
}
