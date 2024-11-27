use crate::report::Report;
use crate::AttestationClaims;

use rust_rsi::{verify_token, TokenError};

pub fn verify(report: &Report) -> Result<AttestationClaims, TokenError> {
    let claims = verify_token(&report.buffer, None)?;

    cfg_if::cfg_if! {
        if #[cfg(target_arch = "x86_64")] {
            Ok(AttestationClaims {
                origin: claims,
                user_data: report.user_data.clone()
            })
        } else {
            Ok(claims)
        }
    }
}
