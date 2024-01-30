use crate::event::Mainloop;
use crate::listen;
use crate::rmi::{self, error::Error};

extern crate alloc;

pub fn decode_version(version: usize) -> (usize, usize) {
    let major = (version & 0x7fff0000) >> 16;
    let minor = version & 0xffff;

    (major, minor)
}

fn encode_version() -> usize {
    (rmi::ABI_MAJOR_VERSION << 16) | rmi::ABI_MINOR_VERSION
}

pub fn set_event_handler(mainloop: &mut Mainloop) {
    listen!(mainloop, rmi::VERSION, |arg, ret, _| {
        let req = arg[0];

        let (req_major, req_minor) = decode_version(req);

        if req_major != rmi::ABI_MAJOR_VERSION || req_minor != rmi::ABI_MINOR_VERSION {
            warn!(
                "Wrong unsupported version requested ({}, {})",
                req_major, req_minor
            );
            return Err(Error::RmiErrorInput);
        }

        let lower = encode_version();
        let higher = lower;

        ret[1] = lower;
        ret[2] = higher;

        trace!("RMI_ABI_VERSION: {:#X?} {:#X?}", lower, higher);

        Ok(())
    });
}
