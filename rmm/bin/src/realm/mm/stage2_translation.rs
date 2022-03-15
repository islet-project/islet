extern crate alloc;

use super::address::{GuestPhysAddr, PhysAddr};
use super::page_table::{get_page_range, L1Table, PageTable};
use super::page_table_entry::{BasePageSize, LargePageSize, PageTableEntryFlags};
use super::pgtlb_allocator;
use crate::config::PAGE_SIZE;
use alloc::boxed::Box;
use core::mem;

// initial lookup starts at level 1 with 2 page tables concatenated
pub const NUM_ROOT_PAGE: usize = 2;
pub const ROOT_PGTLB_ALIGNMENT: usize = PAGE_SIZE * NUM_ROOT_PAGE;

pub struct Stage2Translation {
    // We will set the translation granule with 4KB.
    // Each VM is exepected to use less than 8GB.
    // To reduce the level of page lookup, initial lookup will start from L1.
    // We allocate a single page able which is still able to address 512GB.
    root_pgtlb: Result<*mut PageTable<L1Table>, ()>,
    //vttbr0_el2: u64,
    //vtcr_el2: u64,
    //mair_el2: u64,
    //tcr_el2: u64,
    //sctlr_el2: u64,
    //vstcr_el2: u64,
    //hcr_el2: u64,
}

impl Stage2Translation {
    fn new() -> Self {
        Self {
            root_pgtlb: pgtlb_allocator::allocate_tables(NUM_ROOT_PAGE, ROOT_PGTLB_ALIGNMENT),
            // TODO: set the initial values
            //vttbr0_el2: 0,
            // TODO: VTCR_EL2 SL0=1, T0SZ: 25, granule
            //vtcr_el2: 0,
            //mair_el2: 0,
            //tcr_el2: 0,
            //sctlr_el2: 0,
            //vstcr_el2: 0,
            //hcr_el2: 0,
        }
    }

    fn get_root_pgtlb<T>(&self) -> &mut PageTable<T> {
        unsafe { mem::transmute(self.root_pgtlb.unwrap()) }
    }
}

pub fn create_stage2_table() -> Box<Stage2Translation> {
    let s2_trans_tbl = Box::new(Stage2Translation::new());

    s2_trans_tbl
}

// To start VM with reserved memory during MS1
fn create_static_stage2_table() {
    let stg2_tlb = create_stage2_table();
    let root = stg2_tlb.get_root_pgtlb::<L1Table>();
    let flags: PageTableEntryFlags = PageTableEntryFlags::MEMATTR_NORMAL
        | PageTableEntryFlags::S2AP_RW
        | PageTableEntryFlags::VALID;
    let pages1 = get_page_range::<BasePageSize>(GuestPhysAddr(0x88100000), 0x200 - 0x66);
    let block_2m = get_page_range::<LargePageSize>(GuestPhysAddr(0x88200000), 1);
    let pages2 = get_page_range::<BasePageSize>(GuestPhysAddr(0x88400000), 0x300 - 0x266);
    assert!(root.map_pages(pages1, PhysAddr(0x88066000), flags).is_ok());
    assert!(root
        .map_pages(block_2m, PhysAddr(0x88200000), flags)
        .is_ok());
    assert!(root.map_pages(pages2, PhysAddr(0x88400000), flags).is_ok());
}
