use super::page_table::entry::Entry;
use super::page_table::{attr, L1Table};
#[cfg(not(any(miri, test, fuzzing)))]
use crate::config::PAGE_BITS;
use crate::config::{PlatformMemoryLayout, PAGE_SIZE, RMM_STACK_GUARD_SIZE, RMM_STACK_SIZE};
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

/// Bit shift to align a virtual address to a 4 KiB page for `TLBI VAE2IS, Xt` (`VA[55:12]`).
#[cfg(not(any(miri, test, fuzzing)))]
const TLBI_ADDR_SHIFT: usize = PAGE_BITS;

/// Number of VA bits (`VA[55:12]` -> bits `[43:0]`) in the `TLBI VAE2IS, Xt` register operand.
#[cfg(not(any(miri, test, fuzzing)))]
const TLBI_ADDR_BITS: usize = 44;

/// Mask for the `VA[55:12]` field (bits `[43:0]`, `0x0FFF_FFFF_FFFF`) of `TLBI VAE2IS, Xt`,
/// keeping the upper `TTL` (`[47:44]`) and `RES0` (`[63:48]`) fields zeroed.
#[cfg(not(any(miri, test, fuzzing)))]
const TLBI_ADDR_MASK: usize = (1usize << TLBI_ADDR_BITS) - 1;

/// Encode a virtual address into the register operand format for `TLBI VAE2IS, Xt`.
#[cfg(not(any(miri, test, fuzzing)))]
#[inline(always)]
const fn tlbi_va(addr: usize) -> usize {
    (addr >> TLBI_ADDR_SHIFT) & TLBI_ADDR_MASK
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

    /// Map multiple discontiguous physical granules to a contiguous virtual address range.
    /// This is used for data structures that span multiple granules, such as SVE registers
    /// with vector length > 14 (requiring more than 4KB).
    ///
    /// Returns the starting virtual address of the contiguous mapping, or None on failure.
    pub fn map_multi_granule(&self, phys_addrs: &[PhysAddr], secure: bool) -> Option<VirtAddr> {
        if phys_addrs.is_empty() {
            return None;
        }

        // Mock the mapping in the host mock builds (unit tests and fuzzing):
        // they run in user space where the RMM translation tables are plain
        // data, so a VA range taken from the multi-granule allocator is not
        // backed by any memory and dereferencing it faults. The mock granules
        // of one structure are laid out contiguously (see
        // test_utils::alloc_granule), so the identity VA of the first granule
        // is a valid contiguous mapping.
        if cfg!(any(test, fuzzing)) {
            return Some(VirtAddr::from(phys_addrs[0].as_usize()));
        }

        // Hold the VA allocator lock across allocation and page-table mapping
        // (lock order: VA_MULTI_GRANULE_ALLOCATOR -> RMM_PAGE_TABLE).
        let mut allocator = super::va_range_allocator::VA_MULTI_GRANULE_ALLOCATOR.lock();

        // Allocate a contiguous virtual address range
        let va_start = match allocator.alloc(phys_addrs.len()) {
            Some(va) => va,
            None => {
                error!(
                    "Failed to allocate multi-granule VA range for {} pages",
                    phys_addrs.len()
                );
                return None;
            }
        };

        // Map each physical page to its corresponding virtual page
        let success = self
            .page_table
            .lock()
            .set_pages_for_multi_granule(va_start, phys_addrs, secure);

        if !success {
            // Clean up the mapping allocation
            allocator.remove(va_start);
            return None;
        }

        Some(va_start)
    }

    /// Unmap a multi-granule mapping.
    /// This removes the page table entries and frees the virtual address range.
    pub fn unmap_multi_granule(&self, va_start: VirtAddr) -> bool {
        // Counterpart of the mocked mapping created by map_multi_granule():
        // nothing was allocated in unit tests, so there is nothing to release.
        if cfg!(any(test, fuzzing)) {
            return true;
        }

        // Hold the VA allocator lock across removal and page-table unmapping
        // (lock order: VA_MULTI_GRANULE_ALLOCATOR -> RMM_PAGE_TABLE) so that
        // a concurrent alloc() cannot reuse `va_start` before its PTEs and TLB
        // entries are cleared, and a concurrent double-unmap is rejected.
        let mut allocator = super::va_range_allocator::VA_MULTI_GRANULE_ALLOCATOR.lock();

        let mapping = match allocator.remove(va_start) {
            Some(m) => m,
            None => {
                warn!(
                    "No multi-granule mapping found at {:#X}",
                    va_start.as_usize()
                );
                return false;
            }
        };

        // Remove page table entries
        self.page_table
            .lock()
            .unset_pages_for_multi_granule(va_start, mapping.num_pages);

        true
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

impl Inner<'_> {
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
        let xn_flags = bits_in_reg(PTDesc::UXN, 1) | bits_in_reg(PTDesc::PXN, 1);
        let rmm_flags = bits_in_reg(PTDesc::INDX, attr::mair_idx::RMM_MEM);
        let device_flags = bits_in_reg(PTDesc::INDX, attr::mair_idx::DEVICE_MEM);
        let base_address = layout.rmm_base;
        let rw_start = layout.rw_start;
        let ro_size = rw_start - base_address;
        let rw_size = layout.rw_end - rw_start;
        let uart_phys = layout.uart_phys;
        let el3_shared_page = layout.el3_shared_buf;
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
            rw_flags | rmm_flags | xn_flags,
        );

        for i in 0..crate::config::NUM_OF_CPU {
            // The stack guard is the first page of each per-CPU stack region and it is
            // intentionally left unmapped, so that a stack overflow faults instead of
            // corrupting the memory below the stack. Only the stack area is mapped.
            let mapped_stack_base = layout.stack_base
                + ((RMM_STACK_GUARD_SIZE + RMM_STACK_SIZE) * i) as u64
                + RMM_STACK_GUARD_SIZE as u64;
            self.set_pages(
                VirtAddr::from(mapped_stack_base),
                PhysAddr::from(mapped_stack_base),
                RMM_STACK_SIZE,
                rw_flags | rmm_flags | xn_flags,
            );
        }
        // UART
        // The Device memory type already forbids instruction fetches, but mark the
        // mapping execute-never as well so the protection lives in the PTE itself
        // and every writable mapping in the RMM EL2 page table is uniformly XN.
        self.set_pages(
            VirtAddr::from(uart_phys),
            PhysAddr::from(uart_phys),
            PAGE_SIZE,
            rw_flags | device_flags | xn_flags,
        );
        self.set_pages(
            VirtAddr::from(el3_shared_page),
            PhysAddr::from(el3_shared_page),
            PAGE_SIZE,
            rw_flags | rmm_flags | xn_flags,
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

        #[cfg(not(any(miri, test, fuzzing)))]
        unsafe {
            core::arch::asm!(
                "dsb ishst",
                "tlbi vae2is, {}",
                "dsb ish",
                "isb",
                in(reg) tlbi_va(addr),
            );
        }
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

    /// Map multiple discontiguous physical pages to a contiguous virtual address range.
    /// Each physical page is mapped to successive virtual pages starting from va_start.
    fn set_pages_for_multi_granule(
        &mut self,
        va_start: VirtAddr,
        phys_addrs: &[PhysAddr],
        secure: bool,
    ) -> bool {
        if phys_addrs.is_empty() {
            warn!("set_pages_for_multi_granule: empty phys_addrs");
            return false;
        }

        // Validate all physical addresses upfront
        for (i, &phys) in phys_addrs.iter().enumerate() {
            if phys == PhysAddr::from(0usize) {
                warn!(
                    "set_pages_for_multi_granule: zero physical address at index {}",
                    i
                );
                return false;
            }
        }

        let rw_flags = bits_in_reg(PTDesc::AP, attr::permission::RW);
        let memattr_flags = bits_in_reg(PTDesc::INDX, attr::mair_idx::RMM_MEM);
        let sh_flags = bits_in_reg(PTDesc::SH, attr::shareable::INNER);
        let secure_flags = bits_in_reg(PTDesc::NS, !secure as u64);
        let xn_flags = bits_in_reg(PTDesc::UXN, 1) | bits_in_reg(PTDesc::PXN, 1);
        let valid_flags = bits_in_reg(PTDesc::VALID, 1);

        let flags = rw_flags | memattr_flags | secure_flags | sh_flags | xn_flags | valid_flags;

        // All addresses are valid, proceed with mapping
        for (i, &phys) in phys_addrs.iter().enumerate() {
            let va = va_start + VirtAddr::from(i * PAGE_SIZE);
            self.set_pages(va, phys, PAGE_SIZE, flags);
        }

        true
    }

    /// Unmap pages from a multi-granule mapping.
    fn unset_pages_for_multi_granule(&mut self, va_start: VirtAddr, num_pages: usize) {
        if num_pages == 0 {
            return;
        }

        for i in 0..num_pages {
            let va = va_start + VirtAddr::from(i * PAGE_SIZE);
            let page = Page::<BasePageSize, VirtAddr>::including_address(va);
            self.root_pgtbl.unset_page(page);
        }

        #[cfg(not(any(miri, test, fuzzing)))]
        unsafe {
            core::arch::asm!("dsb ishst");
            for i in 0..num_pages {
                let va = va_start.as_usize() + i * PAGE_SIZE;
                core::arch::asm!("tlbi vae2is, {}", in(reg) tlbi_va(va));
            }
            core::arch::asm!("dsb ish", "isb");
        }
    }
}

impl fmt::Debug for Inner<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct(stringify!(Self)).finish()
    }
}

impl Drop for Inner<'_> {
    fn drop(&mut self) {
        info!("drop PageTable");
        self.root_pgtbl.drop();
    }
}
