use super::page_table::entry::Entry;
use super::page_table::{attr, L1Table};
use crate::config::{PlatformMemoryLayout, PAGE_SIZE, RMM_SHARED_BUFFER_START};
use crate::mm::page::BasePageSize;
use crate::mm::page_table::entry::PTDesc;

use vmsa::address::{PhysAddr, VirtAddr};
use vmsa::page::Page;
use vmsa::page_table::PageTable as RootPageTable;
use vmsa::page_table::{DefaultMemAlloc, Level, PageTableMethods};

use alloc::boxed::Box;
use armv9a::bits_in_reg;
use core::cell::RefCell;
use core::ffi::c_void;
use core::fmt;
use core::pin::Pin;

pub struct PageTable<'a> {
    inner: Pin<Box<RefCell<Inner<'a>>>>,
}

impl<'a> PageTable<'a> {
    pub fn new(layout: PlatformMemoryLayout) -> Self {
        let inner = Pin::new(Box::new(RefCell::new(Inner::new())));
        #[cfg(not(any(miri, test)))]
        inner.borrow_mut().fill(layout);
        Self { inner }
    }

    pub fn map(&self, addr: usize, secure: bool) -> bool {
        self.inner.borrow_mut().set_pages_for_rmi(addr, secure)
    }

    pub fn unmap(&self, addr: usize) -> bool {
        self.inner.borrow_mut().unset_pages_for_rmi(addr)
    }

    pub fn base(&self) -> u64 {
        self.inner.borrow_mut().get_base_address() as u64
    }
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
