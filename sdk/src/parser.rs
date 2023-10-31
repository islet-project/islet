use crate::config;
use cca_token::{dumper::print_token, AttestationClaims, ClaimData};

pub fn parse<'a>(claims: &'a AttestationClaims, title: &'static str) -> Option<&'a ClaimData> {
    let title = support_user_data(title);
    claims.data(title)
}

pub fn print_claims(claims: &AttestationClaims) {
    print_token(&claims);
}

// The requirement of Certifier
fn support_user_data(title: &'static str) -> &'static str {
    if title == config::STR_USER_DATA {
        config::STR_REALM_CHALLENGE
    } else {
        title
    }
}
