pub mod entry;

use self::entry::Entry;
use self::entry::Granule;
use crate::rmi::error::Error;

pub const GRANULE_SIZE: usize = 4096;
pub const GRANULE_SHIFT: usize = 12;
pub const GRANULE_MASK: usize = !((1 << GRANULE_SHIFT) - 1);

// TODO: move this FVP-specific address info
pub(super) const FVP_DRAM0_REGION: core::ops::Range<usize> = core::ops::Range {
    start: 0x8000_0000,
    end: 0x8000_0000 + 0x7C00_0000,
};
pub(super) const FVP_DRAM1_REGION: core::ops::Range<usize> = core::ops::Range {
    start: 0x8_8000_0000,
    end: 0x8_8000_0000 + 0x8000_0000,
};

pub(super) const FVP_DRAM1_IDX: usize =
    (FVP_DRAM0_REGION.end - FVP_DRAM0_REGION.start) / GRANULE_SIZE;

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

pub fn granule_addr_to_index(addr: usize) -> usize {
    if FVP_DRAM0_REGION.contains(&addr) {
        return (addr - FVP_DRAM0_REGION.start) / GRANULE_SIZE;
    }
    if FVP_DRAM1_REGION.contains(&addr) {
        return ((addr - FVP_DRAM1_REGION.start) / GRANULE_SIZE) + FVP_DRAM1_IDX;
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

pub fn create_granule_status_table() {
    unsafe {
        if GRANULE_STATUS_TABLE.is_none() {
            GRANULE_STATUS_TABLE = Some(GranuleStatusTable::new());
        }
    }
}

pub fn set_granule(granule: &mut Granule, state: u8) -> Result<(), Error> {
    granule.set_state(state)
}

pub static mut GRANULE_STATUS_TABLE: Option<GranuleStatusTable> = None;
const GRANULE_STATUS_TABLE_SIZE: usize = 0xfc000; // == RMM_MAX_GRANULES

pub struct GranuleStatusTable {
    pub entries: [Entry; GRANULE_STATUS_TABLE_SIZE],
}

impl GranuleStatusTable {
    pub fn new() -> Self {
        Self {
            entries: core::array::from_fn(|_| Entry::new()),
        }
    }
}

#[macro_export]
macro_rules! get_granule {
    ($addr:expr) => {{
        use crate::granule::array::GRANULE_STATUS_TABLE;
        use crate::granule::{granule_addr_to_index, validate_addr};
        use crate::rmi::error::Error;
        if !validate_addr($addr) {
            Err(Error::RmiErrorInput)
        } else {
            let idx = granule_addr_to_index($addr);
            if idx == usize::MAX {
                Err(Error::RmiErrorInput)
            } else if let Some(gst) = unsafe { &mut GRANULE_STATUS_TABLE } {
                match gst.entries[idx].lock() {
                    Ok(guard) => Ok(guard),
                    Err(e) => Err(e),
                }
            } else {
                Err(Error::RmiErrorInput)
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
