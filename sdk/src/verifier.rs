use crate::error::Error;
use crate::parser::Parser;
use crate::token::Platform;

use minicbor::data::Tag;
use minicbor::Decoder;

use crate::claim::platform::Label;

const TAG_CCA_TOKEN: u64 = 399;
const TAG_COSE_SIGN1: u64 = 18;

const TOKEN_COUNT: u64 = 2;
const TOKEN_PLAT: u16 = 44234;
const TOKEN_REALM: u16 = 44241;

pub fn verify(raw_report: &[u8]) -> Result<(), Error> {
    let (platform, _realm) = cca_token(raw_report)?;
    let (payload, _signature) = plat_token(platform)?;

    let mut parser = Parser::new(payload);
    if 9 != parser.decoder.map()?.ok_or(Error::Decoding)? {
        return Err(Error::Format);
    }

    let plat_token = Platform {
        profile: parser.string_claim(Label::Profile)?,
        challenge: parser.bytes_claim::<32>(Label::Challenge)?,
        implementation_id: parser.bytes_claim::<64>(Label::ImplementationId)?,
        instance_id: parser.bytes_claim::<33>(Label::InstanceId)?,
        config: parser.bytes_claim::<33>(Label::Config)?,
        lifecycle: parser.u16_claim(Label::Lifecycle)?,
        hash_algo: parser.string_claim(Label::HashAlgo)?,
        sw_components: parser.sw_components(Label::SWComponents)?,
        verification_service: parser.string_claim(Label::VerificationService)?,
    };

    println!("{:?}", plat_token);

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
