use crate::error::Error;
use crate::report::Report;

pub fn attest() -> Result<Report, Error> {
    // TODO:
    //   Get report via RSI with parameters (challenge, user_data)
    Ok(Report {
        buffer: crate::mock::REPORT.to_vec(),
    })
}
