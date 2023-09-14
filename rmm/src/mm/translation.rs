use super::page_table::entry::Entry;
use super::page_table::{attr, L1Table};
use crate::asm::dcache_flush;
use crate::config::PAGE_SIZE;
use crate::mm::page::BasePageSize;
use crate::mm::page_table::entry::PTDesc;

use paging::address::{PhysAddr, VirtAddr};
use paging::page::Page;
use paging::page_table::PageTable as RootPageTable;
use paging::page_table::{Level, PageTableMethods};

use armv9a::bits_in_reg;
use core::ffi::c_void;
use core::fmt;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

extern "C" {
    static __RMM_BASE__: u64;
    static __RW_START__: u64;
    static __RW_END__: u64;
}

pub struct PageTable {
    page_table: &'static Mutex<Inner<'static>>,
}

impl PageTable {
    pub fn get_ref() -> Self {
        Self {
            page_table: &RMM_PAGE_TABLE,
        }
    }

    pub fn map(&self, addr: usize, secure: bool) -> bool {
        self.page_table.lock().set_pages_for_rmi(addr, secure)
    }

    pub fn unmap(&self, addr: usize) -> bool {
        self.page_table.lock().unset_pages_for_rmi(addr)
    }
}

lazy_static! {
    static ref RMM_PAGE_TABLE: Mutex<Inner<'static>> = Mutex::new(Inner::new());
}

pub fn get_page_table() -> u64 {
    let mut page_table = RMM_PAGE_TABLE.lock();
    page_table.fill();
    page_table.get_base_address() as u64
}

// initial lookup starts at level 1 with 2 page tables concatenated
pub const NUM_ROOT_PAGE: usize = 1;
pub const ALIGN_ROOT_PAGE: usize = 2;

struct Inner<'a> {
    // We will set the translation granule with 4KB.
    // To reduce the level of page lookup, initial lookup will start from L1.
    root_pgtlb:
        &'a mut RootPageTable<VirtAddr, L1Table, Entry, { <L1Table as Level>::NUM_ENTRIES }>,
    dirty: bool,
}

impl<'a> Inner<'a> {
    pub fn new() -> Self {
        let root_pgtlb = unsafe {
            &mut *RootPageTable::<VirtAddr, L1Table, Entry, { <L1Table as Level>::NUM_ENTRIES }>::new_with_align(
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

    fn fill(&mut self) {
        if self.dirty == true {
            return;
        }

        let ro_flags = bits_in_reg(PTDesc::AP, attr::permission::RO);
        let rw_flags = bits_in_reg(PTDesc::AP, attr::permission::RW);
        let rmm_flags = bits_in_reg(PTDesc::INDX, attr::mair_idx::RW_DATA);

        unsafe {
            let base_address = &__RMM_BASE__ as *const u64 as u64;
            let rw_start = &__RW_START__ as *const u64 as u64;
            let ro_size = rw_start - base_address;
            let rw_size = &__RW_END__ as *const u64 as u64 - rw_start;
            let uart_phys = 0x1c0c0000 as u64;
            self.set_pages(
                VirtAddr::from(base_address),
                PhysAddr::from(base_address),
                ro_size as usize,
                ro_flags | rmm_flags,
            );
            self.set_pages(
                VirtAddr::from(rw_start),
                PhysAddr::from(rw_start),
                rw_size as usize,
                rw_flags,
            );
            // UART
            self.set_pages(
                VirtAddr::from(uart_phys),
                PhysAddr::from(uart_phys),
                1,
                rw_flags | rmm_flags,
            );
        }
        //TODO Set dirty only if pages are updated, not added
        self.dirty = true;
    }

    fn get_base_address(&self) -> *const c_void {
        self.root_pgtlb as *const _ as *const c_void
    }

    fn set_pages(&mut self, va: VirtAddr, phys: PhysAddr, size: usize, flags: u64) {
        let virtaddr = Page::<BasePageSize, VirtAddr>::range_with_size(va, size);
        let phyaddr = Page::<BasePageSize, PhysAddr>::range_with_size(phys, size);

        if self
            .root_pgtlb
            .set_pages(virtaddr, phyaddr, flags, false)
            .is_err()
        {
            warn!("set_pages error");
        }
    }

    fn unset_page(&mut self, addr: usize) {
        let va = VirtAddr::from(addr);
        let page = Page::<BasePageSize, VirtAddr>::including_address(va);
        self.root_pgtlb.unset_page(page);
    }

    fn set_pages_for_rmi(&mut self, addr: usize, secure: bool) -> bool {
        if addr == 0 {
            warn!("map address is empty");
            return false;
        }

        let rw_flags = bits_in_reg(PTDesc::AP, attr::permission::RW);
        let memattr_flags = bits_in_reg(PTDesc::INDX, attr::mair_idx::RMM_MEM);
        let sh_flags = bits_in_reg(PTDesc::SH, attr::shareable::INNER);
        let secure_flags = bits_in_reg(PTDesc::NS, !secure as u64);
        let va = VirtAddr::from(addr);
        let phys = PhysAddr::from(addr);

        self.set_pages(
            va,
            phys,
            PAGE_SIZE,
            rw_flags | memattr_flags | secure_flags | sh_flags,
        );

        dcache_flush(addr, PAGE_SIZE);
        true
    }

    fn unset_pages_for_rmi(&mut self, addr: usize) -> bool {
        if addr == 0 {
            warn!("map address is empty");
            return false;
        }

        self.unset_page(addr);
        true
    }
}

impl<'a> fmt::Debug for Inner<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(Self)).finish()
    }
}

impl<'a> Drop for Inner<'a> {
    fn drop(&mut self) {
        info!("drop PageTable");
        self.root_pgtlb.drop();
    }
}
