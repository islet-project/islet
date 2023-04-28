use crate::error::Error;

pub fn attest() -> Result<Vec<u8>, Error> {
    // TODO:
    //   Get report via RSI with parameters (challenge, user_data)
    Ok(crate::mock::REPORT.to_vec())
}
