use crate::error::Error;
use crate::parser::Parser;
use crate::token;

use minicbor::data::Tag;
use minicbor::Decoder;

const TAG_CCA_TOKEN: u64 = 399;
const TAG_COSE_SIGN1: u64 = 18;

const TOKEN_COUNT: u64 = 2;
const TOKEN_PLAT: u16 = 44234;
const TOKEN_REALM: u16 = 44241;

pub fn verify_plat_token(encoded: &[u8]) -> Result<(), Error> {
    use crate::claim::platform::Label;
    let (payload, _signature) = plat_token(encoded)?;

    let mut parser = Parser::new(payload);
    let plat_token = token::Platform {
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

    println!("{:?}", plat_token);
    Ok(())
}

pub fn verify_realm_token(encoded: &[u8]) -> Result<(), Error> {
    use crate::claim::realm::Label;
    let (payload, _signature) = plat_token(encoded).unwrap();

    let mut parser = Parser::new(payload);
    let token = token::Realm {
        claims_len: parser.decoder.map()?.ok_or(Error::Decoding)?,
        challenge: parser.bytes_claim::<64>(Label::Challenge as u16)?,
        rpv: parser.bytes_claim::<64>(Label::RPV as u16)?,
        public_key: parser.bytes_claim::<97>(Label::PublicKey as u16)?,
        public_key_hash_algo: parser.string_claim(Label::PublicKeyHashAlgo as u16)?,
        hash_algo: parser.string_claim(Label::HashAlgo as u16)?,
        rim: parser.bytes_claim::<32>(Label::RIM as u16)?,
        rem: parser.rem_claim::<32>(Label::REM as u16)?,
    };

    println!("{:?}", token);
    Ok(())
}

pub fn verify(raw_report: &[u8]) -> Result<(), Error> {
    let (platform, realm) = cca_token(raw_report)?;
    verify_plat_token(platform)?;
    verify_realm_token(realm)?;

    Ok(())
}

fn cca_token(raw_report: &[u8]) -> Result<(&[u8], &[u8]), Error> {
    let mut decoder = Decoder::new(&raw_report);
    if decoder.tag()? != Tag::Unassigned(TAG_CCA_TOKEN) {
        return Err(Error::NotCCAToken);
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

fn plat_token(token: &[u8]) -> Result<(&[u8], &[u8]), Error> {
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
