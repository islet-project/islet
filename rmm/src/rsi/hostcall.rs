use crate::granule::{GranuleState, GRANULE_SIZE};
use crate::rmi::error::Error;
use crate::{const_assert_eq, get_granule, get_granule_if};

pub const HOST_CALL_NR_GPRS: usize = 7;
const PADDING: [usize; 2] = [6, 4032];

#[repr(C)]
pub struct HostCall {
    imm: u16,
    padding0: [u8; PADDING[0]],
    gprs: [u64; HOST_CALL_NR_GPRS],
    padding1: [u8; PADDING[1]],
}

// The width of the RsiHostCall structure is 4096 (0x1000) bytes in RMM Spec bet0.
// The width is changed to 256 (0x100) bytes at RMM Spec eac5.
const_assert_eq!(core::mem::size_of::<HostCall>(), GRANULE_SIZE);

impl HostCall {
    pub unsafe fn parse_mut<'a>(addr: usize) -> &'a mut Self {
        &mut *(addr as *mut Self)
    }

    pub fn set_gpr(&mut self, idx: usize, val: u64) -> Result<(), Error> {
        if idx >= HOST_CALL_NR_GPRS {
            error!("out of index: {}", idx);
            return Err(Error::RmiErrorInput);
        }
        self.gprs[idx] = val;
        Ok(())
    }

    pub fn imm(&self) -> u16 {
        self.imm
    }
}

impl core::fmt::Debug for HostCall {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("rsi::HostCall")
            .field("imm", &format_args!("{:#X}", &self.imm))
            .field("gprs", &self.gprs)
            .finish()
    }
}

impl safe_abstraction::RawPtr for HostCall {
    fn is_within_range(&self) -> bool {
        let align_down = self.addr() & !(GRANULE_SIZE - 1);
        get_granule_if!(align_down, GranuleState::Data).is_ok()
    }
}
