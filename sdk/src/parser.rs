use crate::claim::{
    platform::{SWComponent0, SWComponent1},
    Claim, Value,
};
use crate::config::to_label;
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

    pub fn label(&mut self, title: &'static str) -> Result<u16, Error> {
        let label = self.decoder.u16()?;
        if label != to_label(title) {
            println!("Expected: {}, Provided: {}", to_label(title), label);
            Err(Error::Claim(title))
        } else {
            Ok(label)
        }
    }

    pub fn string(&mut self, title: &'static str) -> Result<Claim, Error> {
        let mut parse = || {
            Ok::<Claim, Error>(Claim {
                label: self.label(title)?,
                title,
                value: Value::String(self.decoder.str()?.to_string()),
            })
        };
        parse().or(Err(Error::Claim(title)))
    }

    pub fn bytes<const N: usize>(&mut self, title: &'static str) -> Result<Claim, Error> {
        let mut parse = || {
            let label = self.label(title)?;
            let value = self.decoder.bytes()?;
            if value.len() != N {
                return Err(Error::Claim(title));
            }
            Ok::<Claim, Error>(Claim {
                label,
                title,
                value: Value::Bytes(value.to_vec()),
            })
        };
        parse().or(Err(Error::Claim(title)))
    }

    pub fn u16(&mut self, title: &'static str) -> Result<Claim, Error> {
        let mut parse = || {
            Ok::<Claim, Error>(Claim {
                label: self.label(title)?,
                title,
                value: Value::U16(self.decoder.u16()?),
            })
        };
        parse().or(Err(Error::Claim(title)))
    }

    pub fn rem<const N: usize>(&mut self, title: &'static str) -> Result<Claim, Error> {
        let mut parse = || {
            let label = self.label(title)?;
            let _ = self.decoder.array()?;
            let value = self.decoder.bytes()?;
            if value.len() != N {
                return Err(Error::Claim(title));
            }
            Ok::<Claim, Error>(Claim {
                label,
                title,
                value: Value::Bytes(value.to_vec()),
            })
        };
        parse().or(Err(Error::Claim(title)))
    }

    pub fn sw_components(
        &mut self,
        title: &'static str,
    ) -> Result<(SWComponent0, SWComponent1, SWComponent1, SWComponent1), Error> {
        let _ = self.label(title)?;
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

        Ok((sw_comp0, sw_comp1, sw_comp2, sw_comp3))
    }
}
