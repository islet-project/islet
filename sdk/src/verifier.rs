use crate::report::Report;

use cca_token::{verifier::verify_token, AttestationClaims as Claims, TokenError};

#[cfg(target_arch = "x86_64")]
fn replace_user_data(claims: &mut Claims, user_data: Vec<u8>) {
    let claim = claims
        .claim_mut(crate::config::STR_REALM_CHALLENGE)
        .expect("CCA Token should include Realm challenge.");
    claim.data = cca_token::ClaimData::Bstr(user_data);
}

pub fn verify(report: &Report) -> Result<Claims, TokenError> {
    let claims = verify_token(&report.buffer)?;

    cfg_if::cfg_if! {
        if #[cfg(target_arch="x86_64")] {
          let mut claims = claims;
          replace_user_data(&mut claims, report.user_data.clone());
        }
    }

    Ok(claims)
}
