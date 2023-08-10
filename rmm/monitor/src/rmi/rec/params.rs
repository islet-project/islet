use crate::host::Accessor as HostAccessor;
use crate::rmm::granule::GRANULE_SIZE;

#[repr(C)]
#[derive(Debug)]
pub struct Params {
    pub flags: u64,
    padding0: [u8; 248],
    pub mpidr: u64,
    padding1: [u8; 248],
    pub pc: u64,
    padding2: [u8; 248],
    pub gprs: [u64; 8],
    padding3: [u8; 1216],
    pub num_aux: u64,
    pub aux: [u64; 16],
    padding4: [u8; 1912],
}

impl Default for Params {
    fn default() -> Self {
        Self {
            flags: 0,
            padding0: [0; 248],
            mpidr: 0,
            padding1: [0; 248],
            pc: 0,
            padding2: [0; 248],
            gprs: [0; 8],
            padding3: [0; 1216],
            num_aux: 0,
            aux: [0; 16],
            padding4: [0; 1912],
        }
    }
}

const_assert_eq!(core::mem::size_of::<Params>(), GRANULE_SIZE);

impl HostAccessor for Params {}

#[cfg(test)]
pub mod test {
    use super::*;

    #[test]
    fn spec_params() {
        assert_eq!(core::mem::size_of::<Params>(), GRANULE_SIZE);

        assert_eq!(offset_of!(Params, flags), 0x0);
        assert_eq!(offset_of!(Params, mpidr), 0x100);
        assert_eq!(offset_of!(Params, pc), 0x200);
        assert_eq!(offset_of!(Params, gprs), 0x300);
        assert_eq!(offset_of!(Params, num_aux), 0x800);
        assert_eq!(offset_of!(Params, aux), 0x808);
    }
}
