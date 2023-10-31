#![deny(warnings)]
#![feature(vec_into_raw_parts)]
#![warn(rust_2018_idioms)]

pub mod attester;
pub mod c_api;
/// cbindgen:ignore
pub mod config;
pub mod error;
pub mod prelude;
pub mod report;
pub mod sealing;
pub mod verifier;

#[cfg(target_arch = "x86_64")]
mod mock;
mod parser;

#[cfg(test)]
mod tests {
    use super::prelude::*;

    #[test]
    fn attest_verify() {
        let user_data = b"User data";
        let report = attest(user_data).unwrap();
        let claims = verify(&report).unwrap();

        if let Some(ClaimData::Bstr(data)) = parse(&claims, config::STR_USER_DATA) {
            assert_eq!(&data[..user_data.len()], user_data);
        } else {
            assert!(false, "Claims parsing error.");
        }

        if let Some(ClaimData::Text(data)) = parse(&claims, config::STR_PLAT_PROFILE) {
            assert_eq!(data, "http://arm.com/CCA-SSD/1.0.0");
        } else {
            assert!(false, "Claims parsing error.");
        }
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
