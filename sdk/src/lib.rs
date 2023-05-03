#![deny(warnings)]
#![warn(rust_2018_idioms)]

pub mod attester;
pub mod claim;
pub mod error;
pub mod report;
pub mod verifier;

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
        assert_eq!(report.buffer.len(), mock::REPORT_LEN);
        let claims = verifier::verify(&report).unwrap();
        println!("{:#?}", claims);
        assert_eq!(
            user_data,
            &claims.realm_tok.challenge.value[..user_data.len()]
        );
    }
}
