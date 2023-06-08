use crate::claim::{self, Claim, Claims, Value};
use crate::config::*;
use crate::error::Error;
use crate::parser::Parser;
use crate::report::Report;

use minicbor::data::Tag;
use minicbor::Decoder;

pub fn verify(report: &Report) -> Result<Claims, Error> {
    let (platform, realm) = cca_token(&report.buffer).or(Err(Error::CCAToken))?;
    let (plat_sig, plat_tok, sw_comps) = plat_token(platform)?;
    let (realm_sig, realm_tok) = realm_token(&realm.clone())?;
    let claims = Claims {
        realm_sig,
        realm_tok,
        plat_sig,
        plat_tok,
        sw_comps,
    };

    cfg_if::cfg_if! {
        if #[cfg(target_arch="x86_64")] {
            let mut claims = claims;
            // Replace user_data temporarily
            if let Value::Bytes(user_data) = &mut claims.get_mut(STR_REALM_CHALLENGE).unwrap().value {
                user_data.fill(0);
                user_data[..report.user_data.len()].copy_from_slice(&report.user_data);
            }
        }
    }

    match claims.value(STR_REALM_PUB_KEY).ok_or(Error::Claims)? {
        Value::Bytes(key) => {
            println!("Verify Realm Signature.");
            cose::signing::verify(realm, key, b"").or(Err(Error::RealmSignature))?;
            Ok(claims)
        }
        _ => Err(Error::Claims),
    }
}

fn plat_token(
    encoded: &[u8],
) -> Result<
    (
        claim::PlatformSignature,
        claim::PlatformToken,
        claim::PlatformSWComponents,
    ),
    Error,
> {
    let (payload, signature) = cose_sign1(encoded).or(Err(Error::CoseSign))?;
    let signature = Claim {
        label: TAG_UNASSIGINED,
        title: STR_PLAT_SIGNATURE.to_string(),
        value: Value::Bytes(signature.to_vec()),
    };

    let mut parser = Parser::new(payload);

    const SW_COMP_COUNT: usize = 1;
    if CLAIM_COUNT_PLATFORM_TOKEN + SW_COMP_COUNT
        != parser.decoder.map()?.ok_or(Error::Decoding)? as usize
    {
        return Err(Error::ClaimCount);
    }

    let profile = parser.string(STR_PLAT_PROFILE)?;
    let challenge = parser.bytes::<32>(STR_PLAT_CHALLENGE)?;
    let implementation_id = parser.bytes::<64>(STR_PLAT_IMPLEMENTATION_ID)?;
    let instance_id = parser.bytes::<33>(STR_PLAT_INSTANCE_ID)?;
    let config = parser.bytes::<33>(STR_PLAT_CONFIGURATION)?;
    let lifecycle = parser.u16(STR_PLAT_SECURITY_LIFECYCLE)?;
    let hash_algo_id = parser.string(STR_PLAT_HASH_ALGO_ID)?;
    let sw_components = parser.sw_components(STR_PLAT_SW_COMPONENTS)?;
    let verification_service = parser.string(STR_PLAT_VERIFICATION_SERVICE)?;

    let token = [
        profile,
        challenge,
        implementation_id,
        instance_id,
        config,
        lifecycle,
        hash_algo_id,
        verification_service,
    ];

    Ok((signature, token, sw_components))
}

fn realm_token(encoded: &[u8]) -> Result<(claim::RealmSignature, claim::RealmToken), Error> {
    let (payload, signature) = cose_sign1(encoded).or(Err(Error::CoseSign))?;
    let signature = Claim {
        label: TAG_UNASSIGINED,
        title: STR_REALM_SIGNATURE.to_string(),
        value: Value::Bytes(signature.to_vec()),
    };

    let mut parser = Parser::new(payload);
    if CLAIM_COUNT_REALM_TOKEN != parser.decoder.map()?.ok_or(Error::Decoding)? as usize {
        return Err(Error::ClaimCount);
    }

    let token = [
        parser.bytes::<64>(STR_REALM_CHALLENGE)?,
        parser.bytes::<64>(STR_REALM_PERSONALIZATION_VALUE)?,
        parser.bytes::<97>(STR_REALM_PUB_KEY)?,
        parser.string(STR_REALM_PUB_KEY_HASH_ALGO_ID)?,
        parser.string(STR_REALM_HASH_ALGO_ID)?,
        parser.bytes::<32>(STR_REALM_INITIAL_MEASUREMENT)?,
        parser.rem::<32>(STR_REALM_EXTENTIBLE_MEASUREMENTS)?,
    ];

    Ok((signature, token))
}

fn cca_token(report: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    let mut decoder = Decoder::new(&report);
    if decoder.tag()? != Tag::Unassigned(TAG_CCA_TOKEN) {
        return Err(Error::CCAToken);
    }

    if TOKEN_COUNT != decoder.map()?.ok_or(Error::Decoding)? {
        return Err(Error::Format);
    }

    if TOKEN_PLAT != decoder.u16()? {
        return Err(Error::Format);
    }

    let plat_token = decoder.bytes()?;

    if TOKEN_REALM != decoder.u16()? {
        return Err(Error::Format);
    }

    let realm_token = decoder.bytes()?;
    Ok((plat_token, realm_token))
}

fn cose_sign1(token: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    let mut decoder = Decoder::new(token);
    if Tag::Unassigned(TAG_COSE_SIGN1) != decoder.tag()? {
        return Err(Error::Format);
    }

    if 4 != decoder.array()?.ok_or(Error::Decoding)? {
        return Err(Error::Format);
    }

    decoder.skip()?;
    decoder.skip()?;

    let payload = decoder.bytes()?;
    let signature = decoder.bytes()?;
    Ok((payload, signature))
}
