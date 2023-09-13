use crate::const_assert_eq;
use crate::granule::GRANULE_SIZE;
use crate::host::Accessor as HostAccessor;

const PADDING: [usize; 5] = [248, 248, 248, 1216, 1912];

#[repr(C)]
pub struct Params {
    pub flags: u64,
    padding0: [u8; PADDING[0]],
    pub mpidr: u64,
    padding1: [u8; PADDING[1]],
    pub pc: u64,
    padding2: [u8; PADDING[2]],
    pub gprs: [u64; 8],
    padding3: [u8; PADDING[3]],
    pub num_aux: u64,
    pub aux: [u64; 16],
    padding4: [u8; PADDING[4]],
}

const_assert_eq!(core::mem::size_of::<Params>(), GRANULE_SIZE);

impl Default for Params {
    fn default() -> Self {
        Self {
            flags: 0,
            padding0: [0; PADDING[0]],
            mpidr: 0,
            padding1: [0; PADDING[1]],
            pc: 0,
            padding2: [0; PADDING[2]],
            gprs: [0; 8],
            padding3: [0; PADDING[3]],
            num_aux: 0,
            aux: [0; 16],
            padding4: [0; PADDING[4]],
        }
    }
}
impl core::fmt::Debug for Params {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("Params")
            .field("flags", &format_args!("{:#X}", &self.flags))
            .field("mpidr", &format_args!("{:#X}", &self.mpidr))
            .field("pc", &format_args!("{:#X}", &self.pc))
            .field("gprs", &format_args!("{:#X?}", &self.gprs))
            .field("num_aux", &self.num_aux)
            .field("aux", &self.aux)
            .finish()
    }
}

impl HostAccessor for Params {}

#[cfg(test)]
pub mod test {
    use super::*;
    use crate::offset_of;

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
