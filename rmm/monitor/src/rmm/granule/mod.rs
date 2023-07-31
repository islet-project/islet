pub mod entry;
pub mod translation;

use crate::mm::page_table::{HasSubtable, Level};

use entry::Entry;
use translation::{
    GranuleStatusTable, GRANULE_STATUS_TABLE, L0_TABLE_ENTRY_SIZE_RANGE, L1_TABLE_ENTRY_SIZE_RANGE,
};

pub const GRANULE_SIZE: usize = 4096;
pub const RET_SUCCESS: usize = 0;

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
impl L0Table {
    pub const NUM_ENTRIES: usize = L1Table::NUM_ENTRIES;
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
impl L1Table {
    pub const NUM_ENTRIES: usize = (L0_TABLE_ENTRY_SIZE_RANGE / L1_TABLE_ENTRY_SIZE_RANGE);
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
pub fn set_granule(addr: usize, state: u64) -> usize {
    if let Some(gst) = unsafe { &mut GRANULE_STATUS_TABLE } {
        match gst.set_granule(addr, state) {
            Ok(_) => RET_SUCCESS,
            Err(e) => e.into(),
        }
    } else {
        1
    }
}

#[cfg(test)]
mod test {
    use crate::mm::error::Error;
    use crate::rmm::granule;
    use crate::rmm::granule::create_granule_status_table;
    use crate::rmm::granule::GranuleState;

    const TEST_ADDR: usize = 0x880c_0000;
    const TEST_WRONG_ADDR: usize = 0x7900_0000;

    #[test]
    fn test_add_granule() {
        create_granule_status_table();
        assert!(granule::set_granule(TEST_ADDR, GranuleState::Delegated) == granule::RET_SUCCESS);
        // restore state
        assert!(granule::set_granule(TEST_ADDR, GranuleState::Undelegated) == granule::RET_SUCCESS);
    }

    #[test]
    fn test_find_granule_with_wrong_addr() {
        create_granule_status_table();
        assert!(
            granule::set_granule(TEST_WRONG_ADDR, GranuleState::Delegated)
                == Error::MmInvalidAddr.into()
        );
    }

    #[test]
    fn test_validate_state() {
        create_granule_status_table();
        assert!(granule::set_granule(TEST_ADDR, GranuleState::Delegated) == granule::RET_SUCCESS);
        // RTT state don't use the map
        assert!(granule::set_granule(TEST_ADDR, GranuleState::RTT) == granule::RET_SUCCESS);
        // restore state
        assert!(granule::set_granule(TEST_ADDR, GranuleState::Delegated) == granule::RET_SUCCESS);
        assert!(granule::set_granule(TEST_ADDR, GranuleState::Undelegated) == granule::RET_SUCCESS);
    }

    #[test]
    fn test_validate_wrong_state() {
        create_granule_status_table();
        assert!(granule::set_granule(TEST_ADDR, GranuleState::Delegated) == granule::RET_SUCCESS);
        assert!(
            granule::set_granule(TEST_ADDR, GranuleState::Delegated) == Error::MmStateError.into()
        );

        // restore state
        assert!(granule::set_granule(TEST_ADDR, GranuleState::Undelegated) == granule::RET_SUCCESS);
    }
}
