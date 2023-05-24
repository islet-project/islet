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
    println!("Getting attestation operation on aarch64.");

    const LEN: usize = rsi_el0::CHALLENGE_LEN as usize;
    if user_data.len() > LEN {
        println!("Length of user_data cannot over CHALLENGE_LEN[{}]", LEN);
        return Err(Error::InvalidArgument);
    }

    let mut challenge: [u8; LEN] = [0; LEN];
    challenge[..user_data.len()].clone_from_slice(&user_data);

    match rsi_el0::attestation_token(&challenge) {
        Ok(token) => Ok(Report { buffer: token }),
        Err(error) => {
            println!("Failed to get an attestation report. {:?}", error);
            Err(Error::AttestationReport)
        }
    }
}

pub fn attest(user_data: &[u8]) -> Result<Report, Error> {
    // Encode user_data to challenge claim in the realm token
    cfg_if::cfg_if! {
        if #[cfg(target_arch="x86_64")] {
            attest_x86_64(user_data)
        } else {
            attest_aarch64(user_data)
        }
    }
}
