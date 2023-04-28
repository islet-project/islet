pub mod error;
mod mock;
pub mod token;
pub mod verifier;

pub fn attest() -> Result<Vec<u8>, crate::error::Error> {
    Ok(mock::REPORT.to_vec())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn attest_verify() {
        let report = attest().unwrap();
        assert_eq!(report.len(), mock::REPORT_LEN);
        verifier::verify(&report).unwrap();
    }
}
