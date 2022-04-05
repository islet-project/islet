use monitor::realm::vmem::IPATranslation;

use super::address::{GuestPhysAddr, PhysAddr};
use super::page::{get_page_range, BasePageSize, Page};
use super::page_table::{L1Table, PageTable, PageTableMethods};
use super::page_table_entry::{pte_access_perm, pte_mem_attr};
use super::pgtlb_allocator;
use super::translation_granule_4k::RawPTE;
use crate::config::PAGE_SIZE;
use crate::helper::bits_in_reg;
use crate::helper::VTTBR_EL2;
use core::fmt;

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

        fill_stage2_table(root_pgtlb);

        Self { root_pgtlb }
    }
}

impl<'a> IPATranslation for Stage2Translation<'a> {
    fn get_vttbr(&self, vmid: usize) -> u64 {
        bits_in_reg(VTTBR_EL2::VMID, vmid as u64)
            | bits_in_reg(VTTBR_EL2::BADDR, self.root_pgtlb as *const _ as u64)
    }
}

impl<'a> fmt::Debug for Stage2Translation<'a> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Stage2Translation").finish()
    }
}

fn fill_stage2_table(root: &mut PageTable<L1Table>) {
    // page for vm
    let flags = bits_in_reg(RawPTE::ATTR, pte_mem_attr::NORMAL)
        | bits_in_reg(RawPTE::S2AP, pte_access_perm::RW);
    let pages1 =
        get_page_range::<BasePageSize>(GuestPhysAddr::from(0x8806_c000 as usize), 0x200 - 0x6c);

    root.map_multiple_pages(pages1, PhysAddr::from(0x8806_c000 as usize), flags);

    // page for uart
    let device_flags = bits_in_reg(RawPTE::ATTR, pte_mem_attr::DEVICE_NGNRE)
        | bits_in_reg(RawPTE::S2AP, pte_access_perm::RW);

    let uart_page =
        Page::<BasePageSize>::including_address(GuestPhysAddr::from(0x1c0a_0000 as usize));
    root.map_page(uart_page, PhysAddr::from(0x1c0a_0000), device_flags);
}
