use super::entry;
use super::{validate_addr, GranuleState, L0Table, L1Table, GRANULE_SIZE};
use crate::config;
use crate::const_assert_eq;

use alloc::collections::BTreeMap;
use core::ptr::addr_of_mut;
use vmsa::address::PhysAddr;
use vmsa::error::Error;
use vmsa::page::{Page, PageSize};
use vmsa::page_table::{DefaultMemAlloc, Level, PageTable, PageTableMethods};

pub const L0_TABLE_ENTRY_SIZE_RANGE: usize = 1024 * 1024 * 4; // 4mb
pub const L1_TABLE_ENTRY_SIZE_RANGE: usize = GRANULE_SIZE;

const_assert_eq!(L0_TABLE_ENTRY_SIZE_RANGE % L1_TABLE_ENTRY_SIZE_RANGE, 0);

type L0PageTable = PageTable<PhysAddr, L0Table, entry::Entry, { <L0Table as Level>::NUM_ENTRIES }>;
pub type L1PageTable =
    PageTable<PhysAddr, L1Table, entry::Entry, { <L1Table as Level>::NUM_ENTRIES }>;

pub struct GranuleStatusTable<'a, 'b> {
    pub root_pgtbl: &'a mut L0PageTable,
    l1_tables: BTreeMap<usize, &'b mut L1PageTable>,
    // TODO: replace this BTreeMap with a more efficient structure.
    //    to do so, we need to do refactoring on how we manage entries in PageTable.
    //    e.g., moving storage (`entries: [E; N]`) out of PageTable, and each impl (GST, RMM, RTT) is in charge of handling that.
}

impl GranuleStatusTable<'_, '_> {
    pub fn new() -> Self {
        Self {
            root_pgtbl: L0PageTable::new_in(&DefaultMemAlloc {}).unwrap(),
            l1_tables: BTreeMap::new(),
        }
    }

    fn add_l1_table(&mut self, index: usize, addr: usize) {
        self.l1_tables
            .insert(index, unsafe { &mut *(addr as *mut L1PageTable) });
    }

    pub fn set_granule(&mut self, addr: usize, state: u64) -> Result<(), Error> {
        if !validate_addr(addr) {
            return Err(Error::MmInvalidAddr);
        }
        let pa1 = Page::<GranuleSize, PhysAddr>::including_address(PhysAddr::from(addr));
        let pa2 = Page::<GranuleSize, PhysAddr>::including_address(PhysAddr::from(addr));
        self.root_pgtbl.set_page(pa1, pa2, state)
    }
}

#[derive(Clone, Copy)]
pub enum GranuleSize {}
impl PageSize for GranuleSize {
    const SIZE: usize = GRANULE_SIZE;
    const MAP_TABLE_LEVEL: usize = 1;
    const MAP_EXTRA_FLAG: u64 = GranuleState::Undelegated;
}

pub static mut GRANULE_STATUS_TABLE: Option<GranuleStatusTable<'_, '_>> = None;

pub fn add_l1_table(index: usize, addr: usize) -> Result<usize, Error> {
    if let Some(gst) = unsafe { &mut *addr_of_mut!(GRANULE_STATUS_TABLE) } {
        gst.add_l1_table(index, addr);
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
    if let Some(gst) = unsafe { &mut *addr_of_mut!(GRANULE_STATUS_TABLE) } {
        if let Some(t) = gst.l1_tables.get_mut(&index) {
            Ok(*t as *mut _ as usize)
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

    let mut base_idx = 0;
    let regions = config::NS_DRAM_REGIONS.lock();
    for range in regions.iter() {
        if range.contains(&phys) {
            return Ok((phys - range.start) / GRANULE_SIZE + base_idx);
        }
        base_idx += (range.end - range.start) / GRANULE_SIZE;
    }
    warn!("address is strange 0x{:X}", phys);
    Err(Error::MmInvalidAddr)
}
