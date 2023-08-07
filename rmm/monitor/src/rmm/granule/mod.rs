pub mod entry;
pub mod translation;

use crate::mm::error::Error;
use crate::mm::page_table::{HasSubtable, Level};

use entry::Entry;
use translation::{
    GranuleStatusTable, GRANULE_STATUS_TABLE, L0_TABLE_ENTRY_SIZE_RANGE, L1_TABLE_ENTRY_SIZE_RANGE,
};

pub const GRANULE_SIZE: usize = 4096;

/// The Level 0 Table
/// Each entry (L1table) covers 4mb. This is a configurable number.
pub enum L0Table {}
impl Level for L0Table {
    const THIS_LEVEL: usize = 0;
    const TABLE_SIZE: usize = Self::NUM_ENTRIES * core::mem::size_of::<Entry>();
    const TABLE_ALIGN: usize = 64;
    const NUM_ENTRIES: usize = L1Table::NUM_ENTRIES;
    // Note/TODO: why using "L1Table::NUM_ENTRIES"?
    //     currently, PageTable simply assumes that every level table has the same NUM_ENTRIES.
    //     it gets problematic in which for example L0Table has 1000 entries while L1table has 1024 entries.
    //     when trying to access L1table[1010], it causes out-of-bound access because the array is created by the size of 1000.
    //     for a workaround, we need to use the largest NUM_ENTRIES.
}
impl HasSubtable for L0Table {
    type NextLevel = L1Table;
}

/// The Level 1 Table
/// Each entry covers PAGE_SIZE (4kb).
pub enum L1Table {}
impl Level for L1Table {
    const THIS_LEVEL: usize = 1;
    const TABLE_SIZE: usize = Self::NUM_ENTRIES * core::mem::size_of::<Entry>();
    const TABLE_ALIGN: usize = 64;
    const NUM_ENTRIES: usize = (L0_TABLE_ENTRY_SIZE_RANGE / L1_TABLE_ENTRY_SIZE_RANGE);
}

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct GranuleState {
    pub inner: u64,
}

#[allow(non_upper_case_globals)]
impl GranuleState {
    pub const Undelegated: u64 = 0;
    pub const Delegated: u64 = 1;
    pub const RD: u64 = 2;
    pub const Rec: u64 = 3;
    pub const RecAux: u64 = 4;
    pub const Data: u64 = 5;
    pub const RTT: u64 = 6;

    pub fn new(state: u64) -> Self {
        Self { inner: state }
    }
}

// TODO: we can use "constructors" for this kind of initialization. (we can define macros for that)
pub fn create_granule_status_table() {
    unsafe {
        if GRANULE_STATUS_TABLE.is_none() {
            GRANULE_STATUS_TABLE = Some(GranuleStatusTable::new());
        }
    }
}

// Safety/TODO:
//  - currently, GranuleStatusTable does not involve in mapping some address into RMM page table.
//  - callers must map some physical address, if needed, prior to calling this function.
//  - TODO: we need to define and use a secure interface to map some address and do set_granule().
pub fn set_granule(addr: usize, state: u64) -> Result<(), Error> {
    if let Some(gst) = unsafe { &mut GRANULE_STATUS_TABLE } {
        gst.set_granule(addr, state)
    } else {
        Err(Error::MmErrorOthers)
    }
}

// TODO: this function will be modified soon to be more secure in terms of concurrency.
pub fn get_granule_state(addr: usize) -> Result<u64, Error> {
    if let Some(gst) = unsafe { &mut GRANULE_STATUS_TABLE } {
        gst.get_granule_state(addr)
    } else {
        Err(Error::MmErrorOthers)
    }
}

// TODO: move this FVP-specific address info
const FVP_DRAM0_REGION: core::ops::Range<usize> = core::ops::Range {
    start: 0x8000_0000,
    end: 0x8000_0000 + 0x7C00_0000 - 1,
};
const FVP_DRAM1_REGION: core::ops::Range<usize> = core::ops::Range {
    start: 0x8_8000_0000,
    end: 0x8_8000_0000 + 0x8000_0000 - 1,
};

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

#[cfg(test)]
mod test {
    use crate::mm::error::Error;
    use crate::rmm::granule;
    use crate::rmm::granule::translation::{GranuleStatusTable, GRANULE_STATUS_TABLE};
    use crate::rmm::granule::GranuleState;

    const TEST_ADDR: usize = 0x880c_0000;
    const TEST_WRONG_ADDR: usize = 0x7900_0000;

    fn recreate_granule_status_table() {
        unsafe {
            if GRANULE_STATUS_TABLE.is_none() {
                GRANULE_STATUS_TABLE = Some(GranuleStatusTable::new());
            } else {
                GRANULE_STATUS_TABLE.take();
                GRANULE_STATUS_TABLE = Some(GranuleStatusTable::new());
            }
        }
    }

    #[test]
    fn test_add_granule() {
        recreate_granule_status_table();
        assert!(granule::set_granule(TEST_ADDR, GranuleState::Delegated).is_ok());
        // restore state
        assert!(granule::set_granule(TEST_ADDR, GranuleState::Undelegated).is_ok());
    }

    #[test]
    fn test_find_granule_with_wrong_addr() {
        recreate_granule_status_table();
        assert!(
            granule::set_granule(TEST_WRONG_ADDR, GranuleState::Delegated)
                == Err(Error::MmInvalidAddr)
        );
    }

    #[test]
    fn test_validate_state() {
        recreate_granule_status_table();
        assert!(granule::set_granule(TEST_ADDR, GranuleState::Delegated).is_ok());
        // RTT state don't use the map
        assert!(granule::set_granule(TEST_ADDR, GranuleState::RTT).is_ok());
        // restore state
        assert!(granule::set_granule(TEST_ADDR, GranuleState::Delegated).is_ok());
        assert!(granule::set_granule(TEST_ADDR, GranuleState::Undelegated).is_ok());
    }

    #[test]
    fn test_validate_wrong_state() {
        recreate_granule_status_table();
        assert!(granule::set_granule(TEST_ADDR, GranuleState::Delegated).is_ok());
        assert!(
            granule::set_granule(TEST_ADDR, GranuleState::Delegated) == Err(Error::MmStateError)
        );

        // restore state
        assert!(granule::set_granule(TEST_ADDR, GranuleState::Undelegated).is_ok());
    }

    #[test]
    fn test_get_granule_state() {
        recreate_granule_status_table();
        assert!(granule::set_granule(TEST_ADDR, GranuleState::Delegated).is_ok());

        let state = granule::get_granule_state(TEST_ADDR);
        assert!(state.is_ok());
        assert!(state.unwrap() == GranuleState::Delegated);
    }
}
