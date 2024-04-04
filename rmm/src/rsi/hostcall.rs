use crate::const_assert_eq;
use crate::granule::GRANULE_SIZE;
use crate::rmi::error::Error;

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

impl safe_abstraction::raw_ptr::RawPtr for HostCall {}

impl safe_abstraction::raw_ptr::SafetyChecked for HostCall {}

impl safe_abstraction::raw_ptr::SafetyAssured for HostCall {
    fn is_initialized(&self) -> bool {
        // The initialization of this memory is guaranteed
        // according to the RMM Specification A2.2.4 Granule Wiping.
        // This instance belongs to a Data Granule and has been initialized.
        true
    }

    fn verify_ownership(&self) -> bool {
        // The instance's ownership is guaranteed while being processed by the RMM.
        // While the Realm holds RW permissions for the instance,
        // it cannot exercise these permissions from the moment an SMC request is made
        // until the request is completed. Even in multi-core environments,
        // the designated areas are protected by Stage 2 Table,
        // ensuring that there are no adverse effects on RMM's memory safety.
        true
    }
}
