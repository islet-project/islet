use crate::claim::{self, Claim, Claims};
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
    let user_data = &mut realm_tok.challenge.value;
    user_data.fill(0);
    user_data[..report.user_data.len()].copy_from_slice(&report.user_data);

    Ok(Claims {
        realm_sig,
        realm_tok,
        plat_sig,
        plat_tok,
    })
}

fn plat_token(encoded: &[u8]) -> Result<(claim::PlatformSignature, claim::PlatformToken), Error> {
    use crate::claim::platform::Label;
    let (payload, signature) = cose_sign1(encoded).or(Err(Error::CoseSign))?;
    let signature: claim::PlatformSignature = Claim {
        label: 0,
        value: signature.try_into().or(Err(Error::PlatformSignature))?,
    };

    let mut parser = Parser::new(payload);
    let token = claim::PlatformToken {
        claims_len: parser.decoder.map()?.ok_or(Error::Decoding)?,
        profile: parser.string_claim(Label::Profile as u16)?,
        challenge: parser.bytes_claim::<32>(Label::Challenge as u16)?,
        implementation_id: parser.bytes_claim::<64>(Label::ImplementationId as u16)?,
        instance_id: parser.bytes_claim::<33>(Label::InstanceId as u16)?,
        config: parser.bytes_claim::<33>(Label::Config as u16)?,
        lifecycle: parser.u16_claim(Label::Lifecycle as u16)?,
        hash_algo: parser.string_claim(Label::HashAlgo as u16)?,
        sw_components: parser.sw_components_claim(Label::SWComponents as u16)?,
        verification_service: parser.string_claim(Label::VerificationService as u16)?,
    };

    Ok((signature, token))
}

fn realm_token(encoded: &[u8]) -> Result<(claim::RealmSignature, claim::RealmToken), Error> {
    use crate::claim::realm::Label;
    let (payload, signature) = cose_sign1(encoded).or(Err(Error::CoseSign))?;
    let signature: claim::RealmSignature = Claim {
        label: 0,
        value: signature.try_into().or(Err(Error::RealmSignature))?,
    };

    let mut parser = Parser::new(payload);
    let token = claim::RealmToken {
        claims_len: parser.decoder.map()?.ok_or(Error::Decoding)?,
        challenge: parser.bytes_claim::<64>(Label::Challenge as u16)?,
        rpv: parser.bytes_claim::<64>(Label::RPV as u16)?,
        public_key: parser.bytes_claim::<97>(Label::PublicKey as u16)?,
        public_key_hash_algo: parser.string_claim(Label::PublicKeyHashAlgo as u16)?,
        hash_algo: parser.string_claim(Label::HashAlgo as u16)?,
        rim: parser.bytes_claim::<32>(Label::RIM as u16)?,
        rem: parser.rem_claim::<32>(Label::REM as u16)?,
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
