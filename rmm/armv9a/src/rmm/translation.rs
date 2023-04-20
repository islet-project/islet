use super::page::RmmBasePageSize;
use super::page_table::entry::{Entry, PTDesc};
use super::page_table::{attr, L1Table};
use crate::config::{LARGE_PAGE_SIZE, PAGE_SIZE};
use crate::helper;
use crate::helper::{bits_in_reg, regs};
use core::arch::asm;
use monitor::rmm::address::VirtAddr;

use core::ffi::c_void;
use core::fmt;
use lazy_static::lazy_static;
use monitor::mm::address::PhysAddr;
use monitor::mm::page::Page;
use monitor::mm::page_table::{PageTable, PageTableMethods};
use spin::mutex::Mutex;

extern "C" {
    static __RMM_BASE: u64;
    static __RW_START__: u64;
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
        if self.dirty == true {
            return;
        }

        let ro_flags = helper::bits_in_reg(PTDesc::AP, attr::permission::RO);
        let rw_flags = helper::bits_in_reg(PTDesc::AP, attr::permission::RW);
        unsafe {
            let base_address = &__RMM_BASE as *const u64 as u64;
            let rw_start = &__RW_START__ as *const u64 as u64;
            let uart_phys = 0x1c0c0000 as u64;
            self.set_pages(
                VirtAddr::from(base_address),
                PhysAddr::from(base_address),
                PAGE_SIZE * 19,
                ro_flags,
            );
            self.set_pages(
                VirtAddr::from(rw_start),
                PhysAddr::from(rw_start),
                (LARGE_PAGE_SIZE * 16) - (PAGE_SIZE * 19),
                rw_flags,
            );
            self.set_pages(
                VirtAddr::from(uart_phys),
                PhysAddr::from(uart_phys),
                1,
                rw_flags,
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
    unsafe {
        asm!("tlbi alle2is", "dsb ish", "isb",);
    }

    // /* Set attributes in the right indices of the MAIR. */
    let mair_el2 = bits_in_reg(regs::MAIR_EL2::Attr0, regs::mair_attr::NORMAL);

    /*
     * The size of the virtual address space is configured as 64 – T0SZ.
     * In this, 64 – 0x19 gives 39 bits of virtual address space.
     * This equates to 512GB (2^39), which means that the entire virtual address
     * space is covered by a single L1 table.
     * Therefore, our starting level of translation is level 1.
     */
    let mut tcr_el2 = bits_in_reg(regs::TCR_EL2::T0SZ, 0x19);

    // configure the tcr_el2 attributes
    tcr_el2 |= bits_in_reg(regs::TCR_EL2::PS, regs::tcr_paddr_size::PS_1T)
        | bits_in_reg(regs::TCR_EL2::TG0, regs::tcr_granule::G_4K)
        | bits_in_reg(regs::TCR_EL2::SH0, regs::tcr_shareable::INNER)
        | bits_in_reg(regs::TCR_EL2::ORGN0, regs::tcr_cacheable::WBWA)
        | bits_in_reg(regs::TCR_EL2::IRGN0, regs::tcr_cacheable::WBWA);

    // set the ttlb base address, this is where the memory address translation
    // table walk starts
    let ttlb_base = get_page_table();

    unsafe {
        // Invalidate the local I-cache so that any instructions fetched
        // speculatively are discarded.
        regs::MAIR_EL2.set(mair_el2);
        regs::TCR_EL2.set(tcr_el2);
        regs::TTBR0_EL2.set(ttlb_base);
        asm!("dsb ish", "isb",);
    }
}
