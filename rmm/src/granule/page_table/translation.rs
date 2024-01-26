use super::entry;
use super::{validate_addr, GranuleState, L0Table, L1Table, GRANULE_SIZE};
use super::{FVP_DRAM0_REGION, FVP_DRAM1_REGION};
use crate::const_assert_eq;
use vmsa::address::PhysAddr;
use vmsa::error::Error;
use vmsa::page::{Page, PageSize};
use vmsa::page_table::{Level, PageTable, PageTableMethods};

use alloc::collections::BTreeMap;

pub const DRAM_SIZE: usize = 0x7C00_0000 + 0x8000_0000;

pub const L0_TABLE_ENTRY_SIZE_RANGE: usize = 1024 * 1024 * 4; // 4mb
pub const L1_TABLE_ENTRY_SIZE_RANGE: usize = GRANULE_SIZE;

const_assert_eq!(L0_TABLE_ENTRY_SIZE_RANGE % L1_TABLE_ENTRY_SIZE_RANGE, 0);
const_assert_eq!(DRAM_SIZE % L0_TABLE_ENTRY_SIZE_RANGE, 0);
const_assert_eq!(
    ((DRAM_SIZE / L0_TABLE_ENTRY_SIZE_RANGE) <= L0Table::NUM_ENTRIES),
    true
);

type L0PageTable = PageTable<PhysAddr, L0Table, entry::Entry, { <L0Table as Level>::NUM_ENTRIES }>;
pub type L1PageTable =
    PageTable<PhysAddr, L1Table, entry::Entry, { <L1Table as Level>::NUM_ENTRIES }>;

pub struct GranuleStatusTable {
    pub root_pgtlb: L0PageTable,
    l1_tables: BTreeMap<usize, L1PageTable>,
    // TODO: replace this BTreeMap with a more efficient structure.
    //    to do so, we need to do refactoring on how we manage entries in PageTable.
    //    e.g., moving storage (`entries: [E; N]`) out of PageTable, and each impl (GST, RMM, RTT) is in charge of handling that.
}

impl GranuleStatusTable {
    pub fn new() -> Self {
        Self {
            root_pgtlb: L0PageTable::new(),
            l1_tables: BTreeMap::new(),
        }
    }

    fn add_l1_table(&mut self, index: usize) {
        self.l1_tables.insert(index, L1PageTable::new());
    }

    pub fn set_granule(&mut self, addr: usize, state: u64) -> Result<(), Error> {
        if !validate_addr(addr) {
            return Err(Error::MmInvalidAddr);
        }
        let pa1 = Page::<GranuleSize, PhysAddr>::including_address(PhysAddr::from(addr));
        let pa2 = Page::<GranuleSize, PhysAddr>::including_address(PhysAddr::from(addr));
        self.root_pgtlb.set_page(pa1, pa2, state, false)
    }
}

#[derive(Clone, Copy)]
pub enum GranuleSize {}
impl PageSize for GranuleSize {
    const SIZE: usize = GRANULE_SIZE;
    const MAP_TABLE_LEVEL: usize = 1;
    const MAP_EXTRA_FLAG: u64 = GranuleState::Undelegated;
}

pub static mut GRANULE_STATUS_TABLE: Option<GranuleStatusTable> = None;

pub fn add_l1_table(index: usize) -> Result<usize, Error> {
    if let Some(gst) = unsafe { &mut GRANULE_STATUS_TABLE } {
        gst.add_l1_table(index);
        if let Some(t) = gst.l1_tables.get(&index) {
            Ok(t as *const _ as usize)
        } else {
            Err(Error::MmErrorOthers)
        }
    } else {
        Err(Error::MmErrorOthers)
    }
}

pub fn get_l1_table_addr(index: usize) -> Result<usize, Error> {
    if let Some(gst) = unsafe { &mut GRANULE_STATUS_TABLE } {
        if let Some(t) = gst.l1_tables.get_mut(&index) {
            Ok(t as *mut _ as usize)
        } else {
            Err(Error::MmErrorOthers)
        }
    } else {
        Err(Error::MmErrorOthers)
    }
}

pub fn addr_to_idx(phys: usize) -> Result<usize, Error> {
    if phys % GRANULE_SIZE != 0 {
        warn!("address need to be aligned 0x{:X}", phys);
        return Err(Error::MmInvalidAddr);
    }

    if FVP_DRAM0_REGION.contains(&phys) {
        Ok((phys - FVP_DRAM0_REGION.start) / GRANULE_SIZE)
    } else if FVP_DRAM1_REGION.contains(&phys) {
        let num_dram0 = (FVP_DRAM0_REGION.end - FVP_DRAM0_REGION.start + 1) / GRANULE_SIZE;
        Ok(((phys - FVP_DRAM1_REGION.start) / GRANULE_SIZE) + num_dram0)
    } else {
        warn!("address is strange 0x{:X}", phys);
        Err(Error::MmInvalidAddr)
    }
}
