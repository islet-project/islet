use crate::claim::{
    platform::{Label, SWComponent0, SWComponent1},
    Claim,
};
use crate::error::Error;

use minicbor::Decoder;

pub struct Parser<'a> {
    pub decoder: Decoder<'a>,
}

impl<'a> Parser<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self {
            decoder: Decoder::new(buffer),
        }
    }

    pub fn label(&mut self, expected: u16) -> Result<u16, Error> {
        let provided = self.decoder.u16()?;
        if expected != provided {
            println!("Expected: {}, provided: {}", expected, provided);
            Err(Error::Format)
        } else {
            Ok(expected)
        }
    }

    pub fn string_claim(&mut self, label: Label) -> Result<Claim<String>, Error> {
        let label = label as u16;
        let mut parse = || {
            Ok::<Claim<String>, Error>(Claim {
                label: self.label(label)?,
                value: self.decoder.str()?.to_string(),
            })
        };
        parse().or(Err(Error::PlatformToken(label)))
    }

    pub fn bytes_claim<const N: usize>(&mut self, label: Label) -> Result<Claim<[u8; N]>, Error> {
        let label = label as u16;
        let mut parse = || {
            Ok::<Claim<[u8; N]>, Error>(Claim {
                label: self.label(label)?,
                value: self.decoder.bytes()?.try_into().or(Err(Error::Format))?,
            })
        };
        parse().or(Err(Error::PlatformToken(label)))
    }

    pub fn u16_claim(&mut self, label: Label) -> Result<Claim<u16>, Error> {
        let label = label as u16;
        let mut parse = || {
            Ok::<Claim<u16>, Error>(Claim {
                label: self.label(label)?,
                value: self.decoder.u16()?,
            })
        };
        parse().or(Err(Error::PlatformToken(label)))
    }

    pub fn sw_components(
        &mut self,
        label: Label,
    ) -> Result<Claim<(SWComponent0, SWComponent1, SWComponent1, SWComponent1)>, Error> {
        let label = self.label(label as u16)?;
        let decoder = &mut self.decoder;
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

        Ok(Claim {
            label,
            value: (sw_comp0, sw_comp1, sw_comp2, sw_comp3),
        })
    }
}
