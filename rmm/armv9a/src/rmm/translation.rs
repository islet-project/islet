use super::address::VirtAddr;
use super::page::RmmBasePageSize;
use super::page_table::entry::{Entry, PTDesc};
use super::page_table::{attr, L1Table};
use crate::config::LARGE_PAGE_SIZE;
use crate::helper;

use core::ffi::c_void;
use core::fmt;
use lazy_static::lazy_static;
use monitor::mm::address::PhysAddr;
use monitor::mm::page::Page;
use monitor::mm::page_table::{PageTable, PageTableMethods};
use spin::mutex::Mutex;

extern "C" {
    static __RMM_BASE: u64;
}

lazy_static! {
    static ref RMM_PAGE_TABLE: Mutex<RmmPageTable<'static>> = Mutex::new(RmmPageTable::new());
}

fn get_page_table() -> u64 {
    let mut page_table = RMM_PAGE_TABLE.lock();
    page_table.fill();
    page_table.get_base_address() as u64
}

// initial lookup starts at level 1 with 2 page tables concatenated
pub const NUM_ROOT_PAGE: usize = 1;
pub const ALIGN_ROOT_PAGE: usize = 2;

pub struct RmmPageTable<'a> {
    // We will set the translation granule with 4KB.
    // To reduce the level of page lookup, initial lookup will start from L1.
    root_pgtlb: &'a mut PageTable<VirtAddr, L1Table, Entry>,
    dirty: bool,
}

impl<'a> RmmPageTable<'a> {
    pub fn new() -> Self {
        let root_pgtlb = unsafe {
            &mut *PageTable::<VirtAddr, L1Table, Entry>::new_with_align(
                NUM_ROOT_PAGE,
                ALIGN_ROOT_PAGE,
            )
            .unwrap()
        };

        Self {
            root_pgtlb,
            dirty: false,
        }
    }

    pub fn fill(&mut self) {
        let device_flags = helper::bits_in_reg(PTDesc::AP, attr::permission::RW);
        unsafe {
            let base_address = &__RMM_BASE as *const u64 as u64;
            self.set_pages(
                VirtAddr::from(base_address),
                PhysAddr::from(base_address),
                LARGE_PAGE_SIZE * 16,
                device_flags,
            );
        }
    }

    fn get_base_address(&self) -> *const c_void {
        self.root_pgtlb as *const _ as *const c_void
    }

    fn set_pages(&mut self, va: VirtAddr, phys: PhysAddr, size: usize, flags: u64) {
        let virtaddr = Page::<RmmBasePageSize, VirtAddr>::range_with_size(va, size);
        let phyaddr = Page::<RmmBasePageSize, PhysAddr>::range_with_size(phys, size);

        self.root_pgtlb.set_pages(virtaddr, phyaddr, flags);

        //TODO Set dirty only if pages are updated, not added
        self.dirty = true;
    }
}

impl<'a> fmt::Debug for RmmPageTable<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(Self)).finish()
    }
}

impl<'a> Drop for RmmPageTable<'a> {
    fn drop(&mut self) {
        info!("drop RmmPageTable");
        self.root_pgtlb.drop();
    }
}

pub fn set_register_mm() {
    debug!("initlize mmu registers");

    // set the ttlb base address, this is where the memory address translation
    // table walk starts
    let _ttlb_base = get_page_table();
}
