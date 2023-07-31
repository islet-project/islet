use super::entry::Entry;
use super::{GranuleState, L0Table, GRANULE_SIZE};
use crate::const_assert_eq;
use crate::mm::address::PhysAddr;
use crate::mm::error::Error;
use crate::mm::page::{Page, PageSize};
use crate::mm::page_table::{PageTable, PageTableMethods};

// TODO: move this FVP-specific address info
const FVP_DRAM0_REGION: core::ops::Range<usize> = core::ops::Range {
    start: 0x8000_0000,
    end: 0x8000_0000 + 0x7C00_0000 - 1,
};
const FVP_DRAM1_REGION: core::ops::Range<usize> = core::ops::Range {
    start: 0x8_8000_0000,
    end: 0x8_8000_0000 + 0x8000_0000 - 1,
};
pub const DRAM_SIZE: usize = 0x7C00_0000 + 0x8000_0000;

pub const L0_TABLE_ENTRY_SIZE_RANGE: usize = 1024 * 1024 * 4; // 4mb
pub const L1_TABLE_ENTRY_SIZE_RANGE: usize = GRANULE_SIZE;

const_assert_eq!(L0_TABLE_ENTRY_SIZE_RANGE % L1_TABLE_ENTRY_SIZE_RANGE, 0);
const_assert_eq!(DRAM_SIZE % L0_TABLE_ENTRY_SIZE_RANGE, 0);
const_assert_eq!(
    ((DRAM_SIZE / L0_TABLE_ENTRY_SIZE_RANGE) <= L0Table::NUM_ENTRIES),
    true
);

pub struct GranuleStatusTable<'a> {
    root_pgtlb: &'a mut PageTable<PhysAddr, L0Table, Entry, { L0Table::NUM_ENTRIES }>,
}

impl<'a> GranuleStatusTable<'a> {
    pub fn new() -> Self {
        let root_pgtlb = unsafe {
            &mut *PageTable::<PhysAddr, L0Table, Entry, { L0Table::NUM_ENTRIES }>::new(1).unwrap()
        };
        Self { root_pgtlb }
    }

    fn validate_addr(&self, addr: usize) -> Result<(), Error> {
        if addr % GRANULE_SIZE != 0 {
            return Err(Error::MmInvalidAddr);
        }
        if !FVP_DRAM0_REGION.contains(&addr) && !FVP_DRAM1_REGION.contains(&addr) {
            return Err(Error::MmInvalidAddr);
        }
        Ok(())
    }

    pub fn set_granule(&mut self, addr: usize, state: u64) -> Result<(), Error> {
        self.validate_addr(addr)?;
        let pa1 = Page::<GranuleSize, PhysAddr>::including_address(PhysAddr::from(addr));
        let pa2 = Page::<GranuleSize, PhysAddr>::including_address(PhysAddr::from(addr));
        self.root_pgtlb.set_page(pa1, pa2, state)
    }
}

#[derive(Clone, Copy)]
pub enum GranuleSize {}
impl PageSize for GranuleSize {
    const SIZE: usize = GRANULE_SIZE;
    const MAP_TABLE_LEVEL: usize = 1;
    const MAP_EXTRA_FLAG: u64 = GranuleState::Undelegated;
}

pub static mut GRANULE_STATUS_TABLE: Option<GranuleStatusTable<'static>> = None;

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
