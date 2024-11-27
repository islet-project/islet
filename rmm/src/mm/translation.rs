use super::page_table::entry::Entry;
use super::page_table::{attr, L1Table};
use crate::config::{
    PlatformMemoryLayout, PAGE_SIZE, RMM_SHARED_BUFFER_START, RMM_STACK_GUARD_SIZE, RMM_STACK_SIZE,
};
use crate::mm::page::BasePageSize;
use crate::mm::page_table::entry::PTDesc;

use vmsa::address::{PhysAddr, VirtAddr};
use vmsa::page::Page;
use vmsa::page_table::PageTable as RootPageTable;
use vmsa::page_table::{DefaultMemAlloc, Level, PageTableMethods};

use armv9a::bits_in_reg;
use core::ffi::c_void;
use core::fmt;
use lazy_static::lazy_static;
use spin::mutex::Mutex;

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

pub fn init_page_table(layout: PlatformMemoryLayout) {
    let mut page_table = RMM_PAGE_TABLE.lock();
    page_table.fill(layout);
}

pub fn get_page_table() -> u64 {
    RMM_PAGE_TABLE.lock().get_base_address() as u64
}

pub fn drop_page_table() {
    RMM_PAGE_TABLE.lock().root_pgtbl.drop();
}

struct Inner<'a> {
    // We will set the translation granule with 4KB.
    // To reduce the level of page lookup, initial lookup will start from L1.
    root_pgtbl:
        &'a mut RootPageTable<VirtAddr, L1Table, Entry, { <L1Table as Level>::NUM_ENTRIES }>,
    dirty: bool,
}

impl<'a> Inner<'a> {
    pub fn new() -> Self {
        let root_pgtbl =
            RootPageTable::<VirtAddr, L1Table, Entry, { <L1Table as Level>::NUM_ENTRIES }>::new_in(
                &DefaultMemAlloc {},
            )
            .unwrap();

        Self {
            root_pgtbl,
            dirty: false,
        }
    }

    fn fill(&mut self, layout: PlatformMemoryLayout) {
        if self.dirty {
            return;
        }

        let ro_flags = bits_in_reg(PTDesc::AP, attr::permission::RO);
        let rw_flags = bits_in_reg(PTDesc::AP, attr::permission::RW);
        let rmm_flags = bits_in_reg(PTDesc::INDX, attr::mair_idx::RMM_MEM);
        let device_flags = bits_in_reg(PTDesc::INDX, attr::mair_idx::DEVICE_MEM);
        let base_address = layout.rmm_base;
        let rw_start = layout.rw_start;
        let ro_size = rw_start - base_address;
        let rw_size = layout.rw_end - rw_start;
        let uart_phys = layout.uart_phys;
        let shared_start = RMM_SHARED_BUFFER_START;
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
            rw_flags | rmm_flags,
        );
        let per_cpu = RMM_STACK_GUARD_SIZE + RMM_STACK_SIZE;
        for i in 0..crate::config::NUM_OF_CPU {
            let stack_base = layout.stack_base + (per_cpu * i) as u64;
            self.set_pages(
                VirtAddr::from(stack_base),
                PhysAddr::from(stack_base),
                RMM_STACK_SIZE,
                rw_flags | rmm_flags,
            );
        }
        // UART
        self.set_pages(
            VirtAddr::from(uart_phys),
            PhysAddr::from(uart_phys),
            PAGE_SIZE,
            rw_flags | device_flags,
        );
        self.set_pages(
            VirtAddr::from(shared_start),
            PhysAddr::from(shared_start),
            PAGE_SIZE,
            rw_flags | rmm_flags,
        );

        //TODO Set dirty only if pages are updated, not added
        self.dirty = true;
    }

    fn get_base_address(&self) -> *const c_void {
        self.root_pgtbl as *const _ as *const c_void
    }

    fn set_pages(&mut self, va: VirtAddr, phys: PhysAddr, size: usize, flags: u64) {
        let virtaddr = Page::<BasePageSize, VirtAddr>::range_with_size(va, size);
        let phyaddr = Page::<BasePageSize, PhysAddr>::range_with_size(phys, size);

        if self.root_pgtbl.set_pages(virtaddr, phyaddr, flags).is_err() {
            warn!("set_pages error");
        }
    }

    fn unset_page(&mut self, addr: usize) {
        let va = VirtAddr::from(addr);
        let page = Page::<BasePageSize, VirtAddr>::including_address(va);
        self.root_pgtbl.unset_page(page);
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
        let xn_flags = bits_in_reg(PTDesc::UXN, 1) | bits_in_reg(PTDesc::PXN, 1);
        let valid_flags = bits_in_reg(PTDesc::VALID, 1);

        let va = VirtAddr::from(addr);
        let phys = PhysAddr::from(addr);

        self.set_pages(
            va,
            phys,
            PAGE_SIZE,
            rw_flags | memattr_flags | secure_flags | sh_flags | xn_flags | valid_flags,
        );

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
        self.root_pgtbl.drop();
    }
}
