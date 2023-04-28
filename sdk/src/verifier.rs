use crate::error::Error;
use crate::token::{Platform, SWComponent0, SWComponent1};

use minicbor::data::Tag;
use minicbor::Decoder;

const TAG_CCA_TOKEN: u64 = 399;
const TAG_COSE_SIGN1: u64 = 18;

const TOKEN_COUNT: u64 = 2;
const TOKEN_PLAT: u16 = 44234;
const TOKEN_REALM: u16 = 44241;

pub fn verify(raw_report: &[u8]) -> Result<(), Error> {
    let (platform, _realm) = cca_token(raw_report)?;
    let (payload, _signature) = plat_token(platform)?;

    let mut decoder = Decoder::new(payload);

    if 9 != decoder.map()?.ok_or(Error::Decoding)? {
        return Err(Error::Format);
    }

    let plat_token = Platform {
        profile: (decoder.u16()?, decoder.str()?.to_string()),
        challenge: (
            decoder.u16()?,
            decoder.bytes()?.try_into().or(Err(Error::Format))?,
        ),
        implementation_id: (
            decoder.u16()?,
            decoder.bytes()?.try_into().or(Err(Error::Format))?,
        ),
        instance_id: (
            decoder.u16()?,
            decoder.bytes()?.try_into().or(Err(Error::Format))?,
        ),
        config: (
            decoder.u16()?,
            decoder.bytes()?.try_into().or(Err(Error::Format))?,
        ),
        lifecycle: (decoder.u16()?, decoder.u16()?),
        hash_algo: (decoder.u16()?, decoder.str()?.to_string()),
        sw_components: sw_components(&mut decoder)?,
        verification_service: (decoder.u16()?, decoder.str()?.to_string()),
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

fn sw_components(
    decoder: &mut Decoder,
) -> Result<(u16, SWComponent0, SWComponent1, SWComponent1, SWComponent1), Error> {
    let label = decoder.u16()?;
    assert_eq!(4, decoder.array().unwrap().unwrap());
    assert_eq!(5, decoder.map().unwrap().unwrap());

    let sw_comp0 = SWComponent0 {
        name: (decoder.u16()?, decoder.str()?.to_string()),
        measurement: (
            decoder.u16()?,
            decoder.bytes()?.try_into().or(Err(Error::Format))?,
        ),
        version: (decoder.u16()?, decoder.str()?.to_string()),
        signer_id: (
            decoder.u16()?,
            decoder.bytes()?.try_into().or(Err(Error::Format))?,
        ),
        hash_algo: (decoder.u16()?, decoder.str()?.to_string()),
    };

    assert_eq!(4, decoder.map().unwrap().unwrap());
    let sw_comp1 = SWComponent1 {
        name: (decoder.u16()?, decoder.str()?.to_string()),
        measurement: (
            decoder.u16()?,
            decoder.bytes()?.try_into().or(Err(Error::Format))?,
        ),
        version: (decoder.u16()?, decoder.str()?.to_string()),
        signer_id: (
            decoder.u16()?,
            decoder.bytes()?.try_into().or(Err(Error::Format))?,
        ),
    };

    assert_eq!(4, decoder.map().unwrap().unwrap());
    let sw_comp2 = SWComponent1 {
        name: (decoder.u16()?, decoder.str()?.to_string()),
        measurement: (
            decoder.u16()?,
            decoder.bytes()?.try_into().or(Err(Error::Format))?,
        ),
        version: (decoder.u16()?, decoder.str()?.to_string()),
        signer_id: (
            decoder.u16()?,
            decoder.bytes()?.try_into().or(Err(Error::Format))?,
        ),
    };

    assert_eq!(4, decoder.map().unwrap().unwrap());
    let sw_comp3 = SWComponent1 {
        name: (decoder.u16()?, decoder.str()?.to_string()),
        measurement: (
            decoder.u16()?,
            decoder.bytes()?.try_into().or(Err(Error::Format))?,
        ),
        version: (decoder.u16()?, decoder.str()?.to_string()),
        signer_id: (
            decoder.u16()?,
            decoder.bytes()?.try_into().or(Err(Error::Format))?,
        ),
    };

    Ok((label, sw_comp0, sw_comp1, sw_comp2, sw_comp3))
}
