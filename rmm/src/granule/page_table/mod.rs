pub mod entry;
pub mod translation;

use self::entry::{Entry, Inner};
use self::translation::{
    GranuleStatusTable, GRANULE_STATUS_TABLE, L0_TABLE_ENTRY_SIZE_RANGE, L1_TABLE_ENTRY_SIZE_RANGE,
};
use crate::rmi::error::Error as RmiError;

use vmsa::address::PhysAddr;
use vmsa::error::Error;
use vmsa::page_table::{HasSubtable, Level};

pub const GRANULE_SIZE: usize = 4096;
pub const GRANULE_SHIFT: usize = 12;
pub const GRANULE_MASK: usize = !((1 << GRANULE_SHIFT) - 1);

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

/// Safety / Usage: "granule transaction" a set of APIs that define how to access "granule" and contents inside it.
///
/// - Goal
///   - The high-level goal of these APIs is to enforce that read/write from/to "granule" is only allowed after getting a proper granule lock.
///
/// - Usage: take the example of REALM_DESTROY.
///   (1) `let mut g = get_granule_if!(addr, GranuleStatus::RD);` --> getting a granule while holding a lock. (precisely, it gets EntryGuard<entry::Inner>)
///   (2) ... do something under `g` ...
///   (3) `set_granule(&mut g, GranuleStatus::Delegated)` --> do a state transition when everything goes ok.
///   - this can be seen as a typical transaction that defines a way to access granule. (granule transaction)
///   - to open a transaction, you can use either `get_granule` or `get_granule_if` or `set_state_and_get_granule`.
///   - to modify a granule and close a transaction, `set_granule` can be used.
///
/// - Safety:
///   - In the above usage, `set_granule()` takes &entry::Inner as input, which can only be acquired by `get_granule_if!`.
///     This means that you can access granule only after `get_granule_if!`, and this is ensured by this API design.
///   - TODO(Optional): currently, handling a granule transaction is in charge of developers.
///           e.g., (1) they should invoke `set_granule()` at the right moment, (2) `EntryGuard` still lives after `set_granule()`.
///           we might be able to free them from it via Rust features.
///   - TODO(Must): currently, these APIs do not involve in mapping some address into RMM page table.
///           callers must map some physical address, if needed, before/after using these APIs.
///           we need to define and use a secure interface to map some address and do set_granule().
///
/// - Note:
///   - why is it using macros, not functions?
///     writing a function that returns `EntryGuard<>` requires cloning EntryGuard. To get around this, macros are currently used.
///     we might be able to figure out a better way to define `get_granule_*!` macros.

/// get_granule!(addr: a physical address)
/// - when success, returns `EntryGuard<entry::Inner>` allowing an access to "Granule".
#[macro_export]
macro_rules! get_granule {
    ($addr:expr) => {{
        {
            use crate::granule::translation::{GranuleSize, GRANULE_STATUS_TABLE};
            use crate::granule::validate_addr;
            use vmsa::address::PhysAddr;
            use vmsa::error::Error as MmError;
            use vmsa::page::Page;
            use vmsa::page_table::{self, PageTableMethods};

            if !validate_addr($addr) {
                Err(MmError::MmInvalidAddr)
            } else {
                if let Some(gst) = unsafe { &mut GRANULE_STATUS_TABLE } {
                    let pa =
                        Page::<GranuleSize, PhysAddr>::including_address(PhysAddr::from($addr));
                    match gst
                        .root_pgtlb
                        .entry(pa, 1, false, |e| page_table::Entry::lock(e))
                    {
                        Ok(guard) => match guard {
                            (Some(g), _level) => Ok(g),
                            _ => Err(MmError::MmNoEntry),
                        },
                        Err(e) => Err(e),
                    }
                } else {
                    Err(MmError::MmErrorOthers)
                }
            }
        }
    }};
}

/// get_granule_if!(addr: a physical address, state: a granule state you expect it to be)
/// - when success, returns `EntryGuard<entry::Inner>` allowing an access to "Granule".
#[macro_export]
macro_rules! get_granule_if {
    ($addr:expr, $state:expr) => {{
        get_granule!($addr).and_then(|guard| {
            if guard.state() != $state {
                use vmsa::error::Error as MmError;
                Err(MmError::MmStateError)
            } else {
                Ok(guard)
            }
        })
    }};
}

/// set_state_and_get_granule!(addr: a physical address, state: a granule state you want to set)
/// - when success, returns `EntryGuard<entry::Inner>` allowing an access to "Granule".
#[macro_export]
macro_rules! set_state_and_get_granule {
    ($addr:expr, $state:expr) => {{
        {
            use crate::granule::set_granule_raw;
            set_granule_raw($addr, $state).and_then(|_| get_granule!($addr))
        }
    }};
}

fn make_move_mut_reference<T>(_: T) {}

// Notice: do not try to make a cycle in parent-child relationship
//   e.g., Rd (parent) -> Rec (child) -> Rd (parent)
pub fn set_granule_with_parent(
    parent: Inner,
    child: &mut Inner,
    state: u64,
) -> Result<(), RmiError> {
    let addr = child.addr();
    let prev = child.state();

    match child.set_state(PhysAddr::from(addr), state) {
        Ok(_) => {
            match child.set_parent(parent) {
                Ok(_) => Ok(()),
                Err(e) => {
                    // In this case, state has already been changed.
                    // So, we should get its state back for a complete transaction management.
                    to_rmi_result(child.set_state(PhysAddr::from(addr), prev))?;
                    to_rmi_result(Err(e))
                }
            }
        }
        Err(e) => to_rmi_result(Err(e)),
    }
}

pub fn set_granule(granule: &mut Inner, state: u64) -> Result<(), RmiError> {
    let addr = granule.addr();
    to_rmi_result(granule.set_state(PhysAddr::from(addr), state))?;
    make_move_mut_reference(granule);
    Ok(())
}

pub fn check_granule_parent(parent: &Inner, child: &Inner) -> Result<(), RmiError> {
    to_rmi_result(child.check_parent(parent))
}

/// This is the only function that doesn't take `Inner` as input, instead it takes a raw address.
/// Notice: do not directly call this function outside this file.
pub fn set_granule_raw(addr: usize, state: u64) -> Result<(), Error> {
    if let Some(gst) = unsafe { &mut GRANULE_STATUS_TABLE } {
        gst.set_granule(addr, state)
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

// TODO: we can use "constructors" for this kind of initialization. (we can define macros for that)
pub fn create_granule_status_table() {
    unsafe {
        if GRANULE_STATUS_TABLE.is_none() {
            GRANULE_STATUS_TABLE = Some(GranuleStatusTable::new());
        }
    }
}

pub fn to_rmi_result(res: Result<(), Error>) -> Result<(), RmiError> {
    match res {
        Ok(_) => Ok(()),
        Err(e) => Err(RmiError::from(e)),
    }
}

pub fn is_not_in_realm(addr: usize) -> bool {
    match get_granule_if!(addr, GranuleState::Undelegated) {
        Ok(_) | Err(Error::MmNoEntry) => true,
        _ => false,
    }
}

pub fn is_granule_aligned(addr: usize) -> bool {
    addr % GRANULE_SIZE == 0
}

#[cfg(test)]
mod test {
    use crate::granule::translation::{GranuleStatusTable, GRANULE_STATUS_TABLE};
    use crate::granule::{set_granule, GranuleState};
    use crate::set_state_and_get_granule;
    use vmsa::error::Error;

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

        let test_fn = |addr: usize| -> Result<(), Error> {
            let mut granule = set_state_and_get_granule!(addr, GranuleState::Delegated)?;
            assert!(set_granule(&mut granule, GranuleState::Undelegated).is_ok());
            Ok(())
        };
        assert!(test_fn(TEST_ADDR).is_ok());
    }

    #[test]
    fn test_find_granule_with_wrong_addr() {
        recreate_granule_status_table();

        let test_fn = |addr: usize| -> Result<(), Error> {
            let _ = set_state_and_get_granule!(addr, GranuleState::Delegated)?;
            Ok(())
        };
        assert!(test_fn(TEST_WRONG_ADDR).is_err());
    }

    #[test]
    fn test_validate_state() {
        recreate_granule_status_table();

        let test_fn = |addr: usize| -> Result<(), Error> {
            let mut granule = set_state_and_get_granule!(addr, GranuleState::Delegated)?;
            assert!(set_granule(&mut granule, GranuleState::RTT).is_ok());
            assert!(set_granule(&mut granule, GranuleState::Delegated).is_ok());
            assert!(set_granule(&mut granule, GranuleState::Undelegated).is_ok());
            Ok(())
        };
        assert!(test_fn(TEST_ADDR).is_ok());
    }

    #[test]
    fn test_validate_wrong_state() {
        recreate_granule_status_table();

        let test_fn = |addr: usize| -> Result<(), Error> {
            let mut granule = set_state_and_get_granule!(addr, GranuleState::Delegated)?;
            assert!(set_granule(&mut granule, GranuleState::Delegated).is_err());
            assert!(set_granule(&mut granule, GranuleState::Undelegated).is_ok());
            Ok(())
        };
        assert!(test_fn(TEST_ADDR).is_ok());
    }

    #[test]
    fn test_get_granule_state() {
        recreate_granule_status_table();

        let test_fn = |addr: usize| -> Result<(), Error> {
            let granule = set_state_and_get_granule!(addr, GranuleState::Delegated)?;
            assert!(granule.state() == GranuleState::Delegated);
            Ok(())
        };
        assert!(test_fn(TEST_ADDR).is_ok());
    }
}
