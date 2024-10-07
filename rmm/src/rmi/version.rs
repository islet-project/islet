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

        let lower = encode_version();
        let higher = lower;

        ret[1] = lower;
        ret[2] = higher;

        let (req_major, req_minor) = decode_version(req);

        if req_major != rmi::ABI_MAJOR_VERSION || req_minor != rmi::ABI_MINOR_VERSION {
            warn!(
                "Wrong unsupported version requested ({}, {})",
                req_major, req_minor
            );
            return Err(Error::RmiErrorInput);
        }

        trace!("RMI_ABI_VERSION: {:#X?} {:#X?}", lower, higher);

        Ok(())
    });
}

#[cfg(test)]
mod test {
    use super::encode_version;
    use crate::rmi::{ABI_MAJOR_VERSION, ABI_MINOR_VERSION, SUCCESS, VERSION};
    use crate::test_utils::*;

    // Source: https://github.com/ARM-software/cca-rmm-acs
    // Test Case: cmd_rmi_version_host
    #[test]
    fn rmi_version() {
        let ret = rmi::<VERSION>(&[encode_version()]);

        assert_eq!(ret[0], SUCCESS);

        // Must Be Zero fields
        assert_eq!(extract_bits(ret[1], 31, 63), 0);
        assert_eq!(extract_bits(ret[2], 31, 63), 0);

        // Version Check
        assert_eq!(extract_bits(ret[1], 0, 15), ABI_MINOR_VERSION);
        assert_eq!(extract_bits(ret[1], 16, 30), ABI_MAJOR_VERSION);
    }
}
