extern crate alloc;

use rmm_core::realm::vmem::IPATranslation;

use super::address::{GuestPhysAddr, PhysAddr};
use super::page_table::{get_page_range, L1Table, PageTable};
use super::page_table_entry::{BasePageSize, LargePageSize, PageTableEntryFlags};
use super::pgtlb_allocator;
use crate::aarch64::reg_bitvalue::bits_in_reg;
use crate::aarch64::VTTBR_EL2;
use crate::config::PAGE_SIZE;
use core::{fmt, mem};

// initial lookup starts at level 1 with 2 page tables concatenated
pub const NUM_ROOT_PAGE: usize = 2;
pub const ROOT_PGTLB_ALIGNMENT: usize = PAGE_SIZE * NUM_ROOT_PAGE;

pub struct Stage2Translation<'a> {
    // We will set the translation granule with 4KB.
    // To reduce the level of page lookup, initial lookup will start from L1.
    // We allocate two single page table initial lookup table, addresing up 1TB.
    //root_pgtlb: Result<*mut PageTable<L1Table>, ()>,
    root_pgtlb: &'a mut PageTable<L1Table>,
    vttbr_el2: u64,
}

impl<'a> Stage2Translation<'a> {
    pub fn new(vmid: u64) -> Self {
        let _root_pgtlb = pgtlb_allocator::allocate_tables(NUM_ROOT_PAGE, ROOT_PGTLB_ALIGNMENT);
        // FIXME: temporary for mile stone 1
        fill_stage2_table(&_root_pgtlb);
        Self {
            root_pgtlb: unsafe { mem::transmute(_root_pgtlb.unwrap()) },
            vttbr_el2: get_vttbr(vmid, &_root_pgtlb),
        }
    }

    /*
    fn get_root_pgtlb(&self) -> &mut PageTable<L1Table> {
         //unsafe { mem::transmute(self.root_pgtlb.unwrap()) }
         self.root_pgtlb
    }
    */
}

impl<'a> IPATranslation for Stage2Translation<'a> {
    fn set_mmu(&mut self) {
        unsafe {
            VTTBR_EL2.set(self.vttbr_el2);
        }
    }
}

impl<'a> fmt::Debug for Stage2Translation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stage2Translation")
            //.field("root_pgtlb", &self.root_pgtlb)
            .field("vttbr_el2", &self.vttbr_el2)
            .finish()
    }
}

fn get_vttbr(vmid: u64, pgtlb: &Result<*mut PageTable<L1Table>, ()>) -> u64 {
    bits_in_reg(VTTBR_EL2::VMID, vmid) | bits_in_reg(VTTBR_EL2::BADDR, pgtlb.unwrap() as u64)
}

fn fill_stage2_table(pgtlb: &Result<*mut PageTable<L1Table>, ()>) {
    let root: &mut PageTable<L1Table> = unsafe { mem::transmute(pgtlb.unwrap()) };
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

// To start VM with reserved memory during MS1
/*
pub fn create_stage2_table(vmid: u64) -> Stage2Translation<'static> {
    let s2_trans_tbl = Stage2Translation::new(vmid);

    s2_trans_tbl
}
*/
