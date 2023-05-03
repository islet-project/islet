use crate::error::Error;
use crate::report::Report;

pub fn attest(user_data: &[u8]) -> Result<Report, Error> {
    // Encode user_data to challenge claim in the realm token
    // TODO:
    //   Get report via RSI
    Ok(Report {
        buffer: crate::mock::REPORT.to_vec(),
        user_data: user_data.to_vec(), // Hold user_data temporarily
    })
}
