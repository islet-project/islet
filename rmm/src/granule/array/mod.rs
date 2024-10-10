pub mod entry;

use self::entry::Entry;
use self::entry::Granule;
use crate::rmi::error::Error;

pub const GRANULE_SIZE: usize = 4096;
pub const GRANULE_SHIFT: usize = 12;
pub const GRANULE_MASK: usize = !((1 << GRANULE_SHIFT) - 1);

// TODO: move this FVP-specific address info
#[cfg(not(kani))]
pub(super) const FVP_DRAM0_REGION: core::ops::Range<usize> = core::ops::Range {
    start: 0x8000_0000,
    end: 0x8000_0000 + 0x7C00_0000,
};
#[cfg(not(kani))]
pub(super) const FVP_DRAM1_REGION: core::ops::Range<usize> = core::ops::Range {
    start: 0x8_8000_0000,
    end: 0x8_8000_0000 + 0x8000_0000,
};

#[cfg(not(kani))]
pub(super) const FVP_DRAM1_IDX: usize =
    (FVP_DRAM0_REGION.end - FVP_DRAM0_REGION.start) / GRANULE_SIZE;

#[cfg(kani)]
pub const GRANULE_MEM_SIZE: usize = GRANULE_SIZE * GRANULE_STATUS_TABLE_SIZE;
#[cfg(kani)]
// We model the Host memory as a pre-allocated memory region which
// can avoid a false positive related to invalid memory accesses
// in model checking. Also, instead of using the same starting
// address (e.g., 0x8000_0000), we use a mock region filled with
// non-deterministic contents. It helps to address an issue related
// to the backend CBMC's pointer encoding, as 0x8000_0000 cannot be
// distinguished from null pointer in CBMC.
pub const GRANULE_REGION: [u8; GRANULE_MEM_SIZE] = [0; GRANULE_MEM_SIZE];

#[cfg(not(kani))]
pub fn validate_addr(addr: usize) -> bool {
    if addr % GRANULE_SIZE != 0 {
        // if the address is out of range.
        warn!("address need to be aligned 0x{:X}", addr);
        return false;
    }
    if !(FVP_DRAM0_REGION.contains(&addr) || FVP_DRAM1_REGION.contains(&addr)) {
        // if the address is out of range.
        warn!("address is strange 0x{:X}", addr);
        return false;
    }
    true
}
#[cfg(kani)]
// DIFF: check against GRANULE_REGION
pub fn validate_addr(addr: usize) -> bool {
    if addr % GRANULE_SIZE != 0 {
        // if the address is out of range.
        warn!("address need to be aligned 0x{:X}", addr);
        return false;
    }
    let g_start = GRANULE_REGION.as_ptr() as usize;
    let g_end = g_start + GRANULE_MEM_SIZE;
    (addr >= g_start && addr < g_end)
}

#[cfg(not(kani))]
pub fn granule_addr_to_index(addr: usize) -> usize {
    if FVP_DRAM0_REGION.contains(&addr) {
        return (addr - FVP_DRAM0_REGION.start) / GRANULE_SIZE;
    }
    if FVP_DRAM1_REGION.contains(&addr) {
        return ((addr - FVP_DRAM1_REGION.start) / GRANULE_SIZE) + FVP_DRAM1_IDX;
    }
    usize::MAX
}
#[cfg(kani)]
// DIFF: calculate index using GRANULE_REGION
pub fn granule_addr_to_index(addr: usize) -> usize {
    let g_start = GRANULE_REGION.as_ptr() as usize;
    let g_end = g_start + GRANULE_MEM_SIZE;
    if addr >= g_start && addr < g_end {
        return (addr - g_start) / GRANULE_SIZE;
    }
    usize::MAX
}

pub fn is_granule_aligned(addr: usize) -> bool {
    addr % GRANULE_SIZE == 0
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GranuleState {
    pub inner: u8,
}

#[allow(non_upper_case_globals)]
impl GranuleState {
    pub const Undelegated: u8 = 0;
    pub const Delegated: u8 = 1;
    pub const RD: u8 = 2;
    pub const Rec: u8 = 3;
    pub const RecAux: u8 = 4;
    pub const Data: u8 = 5;
    pub const RTT: u8 = 6;

    pub fn new(state: u8) -> Self {
        Self { inner: state }
    }
}

pub fn set_granule(granule: &mut Granule, state: u8) -> Result<(), Error> {
    granule.set_state(state)
}

lazy_static! {
    pub static ref GRANULE_STATUS_TABLE: GranuleStatusTable = GranuleStatusTable::new();
}

#[cfg(not(kani))]
pub const GRANULE_STATUS_TABLE_SIZE: usize = 0xfc000; // == RMM_MAX_GRANULES
#[cfg(kani)]
pub const GRANULE_STATUS_TABLE_SIZE: usize = 6;

pub struct GranuleStatusTable {
    pub entries: [Entry; GRANULE_STATUS_TABLE_SIZE],
}

impl GranuleStatusTable {
    pub fn new() -> Self {
        Self {
            entries: core::array::from_fn(|_| Entry::new()),
        }
    }

    #[cfg(kani)]
    pub fn is_valid(&self) -> bool {
        self.entries
            .iter()
            .fold(true, |acc, x| acc && x.lock().unwrap().is_valid())
    }
}

#[macro_export]
macro_rules! get_granule {
    ($addr:expr) => {{
        use crate::granule::array::{GRANULE_STATUS_TABLE, GRANULE_STATUS_TABLE_SIZE};
        use crate::granule::{granule_addr_to_index, validate_addr};
        use crate::rmi::error::Error;
        if !validate_addr($addr) {
            Err(Error::RmiErrorInput)
        } else {
            let idx = granule_addr_to_index($addr);
            if idx >= GRANULE_STATUS_TABLE_SIZE {
                Err(Error::RmiErrorInput)
            } else {
                let gst = &GRANULE_STATUS_TABLE;
                match gst.entries[idx].lock() {
                    Ok(guard) => Ok(guard),
                    Err(e) => Err(e),
                }
            }
        }
    }};
}

#[macro_export]
macro_rules! get_granule_if {
    ($addr:expr, $state:expr) => {{
        get_granule!($addr).and_then(|guard| {
            if guard.state() != $state {
                use crate::rmi::error::Error;
                Err(Error::RmiErrorInput)
            } else {
                Ok(guard)
            }
        })
    }};
}

pub fn is_not_in_realm(addr: usize) -> bool {
    match get_granule_if!(addr, GranuleState::Undelegated) {
        Ok(_) => true,
        _ => false,
    }
}
