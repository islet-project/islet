use super::page::{get_page_range, BasePageSize};
use super::page_table::{entry::Entry, L1Table, PageTable, PageTableMethods};
use crate::config::PAGE_SIZE;

use core::ffi::c_void;
use core::fmt;

use monitor::realm::mm::address::{GuestPhysAddr, PhysAddr};
use monitor::realm::mm::IPATranslation;

// initial lookup starts at level 1 with 2 page tables concatenated
pub const NUM_ROOT_PAGE: usize = 2;

pub struct Stage2Translation<'a> {
    // We will set the translation granule with 4KB.
    // To reduce the level of page lookup, initial lookup will start from L1.
    // We allocate two single page table initial lookup table, addresing up 1TB.
    root_pgtlb: &'a mut PageTable<L1Table, Entry>,
    dirty: bool,
}

impl<'a> Stage2Translation<'a> {
    pub fn new() -> Self {
        let root_pgtlb = unsafe { &mut *PageTable::<L1Table, Entry>::new(NUM_ROOT_PAGE).unwrap() };

        Self {
            root_pgtlb,
            dirty: false,
        }
    }
}

impl<'a> IPATranslation for Stage2Translation<'a> {
    fn get_base_address(&self) -> *const c_void {
        self.root_pgtlb as *const _ as *const c_void
    }

    fn set_pages(&mut self, guest: GuestPhysAddr, phys: PhysAddr, size: usize, flags: usize) {
        let pages = get_page_range::<BasePageSize>(guest, size / PAGE_SIZE);

        self.root_pgtlb
            .map_multiple_pages(pages, phys, flags as u64);

        //TODO Set dirty only if pages are updated, not added
        self.dirty = true;
    }
    fn unset_pages(&mut self, _guest: GuestPhysAddr, _size: usize) {
        //TODO implement
    }

    fn clean(&mut self) {
        if self.dirty {
            unsafe {
                llvm_asm! {
                    "
                    dsb ishst
                    tlbi vmalle1
                    dsb ish
                    isb
                    "
                }
            }

            self.dirty = false;
        }
    }
}

impl<'a> fmt::Debug for Stage2Translation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stage2Translation").finish()
    }
}
