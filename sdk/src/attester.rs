use crate::error::Error;
use crate::report::Report;

#[cfg(target_arch = "x86_64")]
fn attest_x86_64(user_data: &[u8]) -> Result<Report, Error> {
    println!("Simulated attestation operation on x86_64.");
    Ok(Report {
        buffer: crate::mock::REPORT.to_vec(),
        user_data: user_data.to_vec(), // Hold user_data temporarily
    })
}

#[cfg(target_arch = "aarch64")]
fn attest_aarch64(user_data: &[u8]) -> Result<Report, Error> {
    // TODO: Get attestation token via ioctl
    println!("Simulated attestation operation on aarch64.");
    Ok(Report {
        buffer: crate::mock::REPORT.to_vec(),
        user_data: user_data.to_vec(), // Hold user_data temporarily
    })
}

pub fn attest(user_data: &[u8]) -> Result<Report, Error> {
    // Encode user_data to challenge claim in the realm token
    // TODO:
    //   Get report via RSI
    cfg_if::cfg_if! {
        if #[cfg(target_arch="x86_64")] {
            attest_x86_64(user_data)
        } else {
            attest_aarch64(user_data)
        }
    }
}
