use crate::host::Accessor as HostAccessor;
use crate::rmi::error::Error;
use crate::rmi::{HASH_ALGO_SHA256, HASH_ALGO_SHA512};
use crate::rmm::granule::GRANULE_SIZE;

const PADDING: [usize; 5] = [248, 767, 960, 6, 2020];

#[repr(C)]
pub struct Params {
    pub features_0: u64,
    padding0: [u8; PADDING[0]],
    pub hash_algo: u8,
    padding1: [u8; PADDING[1]],
    pub rpv: [u8; 64],
    padding2: [u8; PADDING[2]],
    pub vmid: u16,
    padding3: [u8; PADDING[3]],
    pub rtt_base: u64,
    pub rtt_level_start: i64,
    pub rtt_num_start: u32,
    padding4: [u8; PADDING[4]],
}

const_assert_eq!(core::mem::size_of::<Params>(), GRANULE_SIZE);

impl Params {
    pub fn validate(&self) -> Result<(), Error> {
        match self.hash_algo {
            HASH_ALGO_SHA256 | HASH_ALGO_SHA512 => Ok(()),
            _ => Err(Error::RmiErrorInput),
        }
    }
}

impl Default for Params {
    fn default() -> Self {
        Self {
            features_0: 0,
            padding0: [0; PADDING[0]],
            hash_algo: 0,
            padding1: [0; PADDING[1]],
            rpv: [0; 64],
            padding2: [0; PADDING[2]],
            vmid: 0,
            padding3: [0; PADDING[3]],
            rtt_base: 0,
            rtt_level_start: 0,
            rtt_num_start: 0,
            padding4: [0; PADDING[4]],
        }
    }
}

impl core::fmt::Debug for Params {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Params")
            .field("features_0", &format_args!("{:#X}", &self.features_0))
            .field("hash_algo", &self.hash_algo)
            .field("rpv", &self.rpv)
            .field("vmid", &self.vmid)
            .field("rtt_base", &format_args!("{:#X}", &self.rtt_base))
            .field("rtt_level_start", &self.rtt_level_start)
            .field("rtt_num_start", &self.rtt_num_start)
            .finish()
    }
}

impl HostAccessor for Params {}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn spec_params() {
        assert_eq!(core::mem::size_of::<Params>(), GRANULE_SIZE);

        assert_eq!(offset_of!(Params, features_0), 0x0);
        assert_eq!(offset_of!(Params, hash_algo), 0x100);
        assert_eq!(offset_of!(Params, rpv), 0x400);
        assert_eq!(offset_of!(Params, vmid), 0x800);
        assert_eq!(offset_of!(Params, rtt_base), 0x808);
        assert_eq!(offset_of!(Params, rtt_level_start), 0x810);
        assert_eq!(offset_of!(Params, rtt_num_start), 0x818);
    }
}
