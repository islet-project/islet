use crate::claim::{self, Claim, Claims, Value};
use crate::config::*;
use crate::error::Error;
use crate::parser::Parser;
use crate::report::Report;

use minicbor::data::Tag;
use minicbor::Decoder;

pub fn verify(report: &Report) -> Result<Claims, Error> {
    let (platform, realm) = cca_token(&report.buffer).or(Err(Error::CCAToken))?;
    let (plat_sig, plat_tok) = plat_token(platform)?;
    let (realm_sig, realm_tok) = realm_token(realm)?;

    // Replace user_data temporarily
    let mut realm_tok = realm_tok;
    if let Value::Bytes(user_data) = &mut realm_tok.challenge.value {
        user_data.fill(0);
        user_data[..report.user_data.len()].copy_from_slice(&report.user_data);
    }

    Ok(Claims {
        realm_sig,
        realm_tok,
        plat_sig,
        plat_tok,
    })
}

fn plat_token(encoded: &[u8]) -> Result<(claim::PlatformSignature, claim::PlatformToken), Error> {
    let (payload, signature) = cose_sign1(encoded).or(Err(Error::CoseSign))?;
    let signature = Claim {
        label: TAG_UNASSIGINED,
        title: STR_PLAT_SIGNATURE,
        value: Value::Bytes(signature.to_vec()),
    };

    let mut parser = Parser::new(payload);
    let token = claim::PlatformToken {
        claims_len: parser.decoder.map()?.ok_or(Error::Decoding)?,
        profile: parser.string(STR_PLAT_PROFILE)?,
        challenge: parser.bytes::<32>(STR_PLAT_CHALLENGE)?,
        implementation_id: parser.bytes::<64>(STR_PLAT_IMPLEMENTATION_ID)?,
        instance_id: parser.bytes::<33>(STR_PLAT_INSTANCE_ID)?,
        config: parser.bytes::<33>(STR_PLAT_CONFIGURATION)?,
        lifecycle: parser.u16(STR_PLAT_SECURITY_LIFECYCLE)?,
        hash_algo_id: parser.string(STR_PLAT_HASH_ALGO_ID)?,
        sw_components: parser.sw_components(STR_PLAT_SW_COMPONENTS)?,
        verification_service: parser.string(STR_PLAT_VERIFICATION_SERVICE)?,
    };

    Ok((signature, token))
}

fn realm_token(encoded: &[u8]) -> Result<(claim::RealmSignature, claim::RealmToken), Error> {
    let (payload, signature) = cose_sign1(encoded).or(Err(Error::CoseSign))?;
    let signature = Claim {
        label: TAG_UNASSIGINED,
        title: STR_REALM_SIGNATURE,
        value: Value::Bytes(signature.to_vec()),
    };

    let mut parser = Parser::new(payload);
    let token = claim::RealmToken {
        claims_len: parser.decoder.map()?.ok_or(Error::Decoding)?,
        challenge: parser.bytes::<64>(STR_REALM_CHALLENGE)?,
        rpv: parser.bytes::<64>(STR_REALM_PERSONALIZATION_VALUE)?,
        public_key: parser.bytes::<97>(STR_REALM_PUB_KEY)?,
        public_key_hash_algo: parser.string(STR_REALM_PUB_KEY_HASH_ALGO_ID)?,
        hash_algo: parser.string(STR_REALM_HASH_ALGO_ID)?,
        rim: parser.bytes::<32>(STR_REALM_INITIAL_MEASUREMENT)?,
        rem: parser.rem::<32>(STR_REALM_EXTENTIBLE_MEASUREMENTS)?,
    };

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
