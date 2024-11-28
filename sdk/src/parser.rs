use crate::error::Error;
use crate::AttestationClaims;
use rust_rsi::{print_token, PlatClaims, RealmClaims};

pub fn parse(claims: &AttestationClaims) -> Result<(RealmClaims, PlatClaims), Error> {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "x86_64")] {
            let mut realm_claims = RealmClaims::from_raw_claims(
                    &claims.origin.realm_claims.token_claims,
                    &claims.origin.realm_claims.measurement_claims)?;
            realm_claims.challenge = claims.user_data.clone();
            let plat_claims = PlatClaims::from_raw_claims(
                    &claims.origin.platform_claims.token_claims)?;
            Ok((realm_claims, plat_claims))
        } else {
            let realm_claims = RealmClaims::from_raw_claims(
                    &claims.realm_claims.token_claims,
                    &claims.realm_claims.measurement_claims)?;
            let plat_claims = PlatClaims::from_raw_claims(
                    &claims.platform_claims.token_claims)?;
            Ok((realm_claims, plat_claims))
        }
    }
}

pub fn print_claims(claims: &AttestationClaims) {
    cfg_if::cfg_if! {
        if #[cfg(target_arch = "x86_64")] {
            print_token(&claims.origin);
        } else {
            print_token(&claims);
        }
    }
}
