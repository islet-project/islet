#![deny(warnings)]
#![feature(vec_into_raw_parts)]
#![warn(rust_2018_idioms)]

pub mod attester;
pub mod c_api;
pub mod claim;
pub mod error;
pub mod report;
pub mod sealing;
pub mod verifier;

/// cbindgen:ignore
mod config;
mod mock;
mod parser;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attest_verify() {
        let user_data = b"User data";
        let report = attester::attest(user_data).unwrap();
        let claims = verifier::verify(&report).unwrap();
        println!("Debug: {:?}", claims);

        if let claim::Value::Bytes(value) = &claims.value(config::STR_USER_DATA).unwrap() {
            assert_eq!(user_data, &value[..user_data.len()]);
        } else {
            assert!(false, "Wrong user data");
        }

        if let claim::Value::String(value) = &claims.value(config::STR_PLAT_PROFILE).unwrap() {
            assert_eq!(value.as_str(), "http://arm.com/CCA-SSD/1.0.0");
        } else {
            assert!(false, "Wrong platform profile");
        }
    }

    #[test]
    fn claim_not_supported_yet() {
        let user_data = b"User data";
        let report = attester::attest(user_data).unwrap();
        let claims = verifier::verify(&report).unwrap();
        assert!(claims.value(config::STR_PLAT_SW_COMPONENTS).is_none());
    }

    #[test]
    fn sealing() {
        use super::sealing::{seal, unseal};
        let usr_data = b"User data";
        let enc_data = seal(usr_data).unwrap();
        let dec_data = unseal(&enc_data).unwrap();
        assert_eq!(usr_data, &dec_data[..]);
    }
}
