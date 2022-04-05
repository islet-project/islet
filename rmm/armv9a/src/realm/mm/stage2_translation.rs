use super::page::{get_page_range, BasePageSize};
use super::page_table::{L1Table, PageTable, PageTableMethods};
use super::pgtlb_allocator;
use crate::config::PAGE_SIZE;
use crate::helper::bits_in_reg;
use crate::helper::VTTBR_EL2;

use core::fmt;

use monitor::realm::mm::address::{GuestPhysAddr, PhysAddr};
use monitor::realm::mm::IPATranslation;

// initial lookup starts at level 1 with 2 page tables concatenated
pub const NUM_ROOT_PAGE: usize = 2;
pub const ROOT_PGTLB_ALIGNMENT: usize = PAGE_SIZE * NUM_ROOT_PAGE;

pub struct Stage2Translation<'a> {
    // We will set the translation granule with 4KB.
    // To reduce the level of page lookup, initial lookup will start from L1.
    // We allocate two single page table initial lookup table, addresing up 1TB.
    root_pgtlb: &'a mut PageTable<L1Table>,
}

impl<'a> Stage2Translation<'a> {
    pub fn new() -> Self {
        let root_pgtlb = unsafe {
            &mut *(pgtlb_allocator::allocate_tables(NUM_ROOT_PAGE, ROOT_PGTLB_ALIGNMENT).unwrap())
        };

        Self { root_pgtlb }
    }
}

impl<'a> IPATranslation for Stage2Translation<'a> {
    fn get_vttbr(&self, vmid: usize) -> u64 {
        bits_in_reg(VTTBR_EL2::VMID, vmid as u64)
            | bits_in_reg(VTTBR_EL2::BADDR, self.root_pgtlb as *const _ as u64)
    }

    fn set_pages(&mut self, guest: GuestPhysAddr, phys: PhysAddr, size: usize, flags: usize) {
        let pages = get_page_range::<BasePageSize>(guest, size / PAGE_SIZE);

        self.root_pgtlb
            .map_multiple_pages(pages, phys, flags as u64);
    }
    fn unset_pages(&mut self, _guest: GuestPhysAddr, _size: usize) {
        //TODO implement
    }
}

impl<'a> fmt::Debug for Stage2Translation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stage2Translation").finish()
    }
}
