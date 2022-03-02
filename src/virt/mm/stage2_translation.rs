extern crate alloc;

use super::page_table::{L1Table, PageTable};
use super::pgtlb_allocator;
use alloc::boxed::Box;

// initial lookup starts at level 1 with 2 page tables concatenated
pub const ROOT_PGTLB_ALIGNMENT: usize = 13;

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
            root_pgtlb: pgtlb_allocator::allocate_tables(1, ROOT_PGTLB_ALIGNMENT),
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
}

pub fn create_stage2_table() -> Box<Stage2Translation> {
    let s2_trans_tbl = Box::new(Stage2Translation::new());

    s2_trans_tbl
}

/*
#[cfg(test)]
pub mod test {
    extern crate alloc;
    use realm_management_monitor::virt::mm::{PageTable, Stage2Translation};

    #[test]
    fn test_stage2_table() {
        stg2_tlb = create_stage2_table();
        assert!(stg2_tlb);
        let page1 = Page<BasePageSize>{
            gpa = 0x,
        }
        stg2_tlb.map_page
    }
}
*/
