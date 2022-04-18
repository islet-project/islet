use monitor::realm::mm::address::PhysAddr;

use super::super::translation_granule_4k::RawPTE;
use super::pte;

#[derive(Clone, Copy)]
pub struct Entry(RawPTE);

impl Entry {
    pub fn new() -> Self {
        Self(RawPTE::new(0))
    }

    pub fn is_valid(&self) -> bool {
        self.0.get_masked_value(RawPTE::VALID) != 0
    }

    pub fn clear(&mut self) {
        self.0 = RawPTE::new(0);
    }

    pub fn get_page_addr(&self, level: usize) -> Option<PhysAddr> {
        match self.is_valid() {
            true => match self.0.get_masked_value(RawPTE::TYPE) {
                pte::page_type::TABLE_OR_PAGE => {
                    Some(PhysAddr::from(self.0.get_masked(RawPTE::ADDR_TBL_OR_PAGE)))
                }
                pte::page_type::BLOCK => match level {
                    1 => Some(PhysAddr::from(self.0.get_masked(RawPTE::ADDR_BLK_L1))),
                    2 => Some(PhysAddr::from(self.0.get_masked(RawPTE::ADDR_BLK_L2))),
                    _ => None,
                },
                _ => None,
            },
            false => None,
        }
    }

    /// Mark this as a valid (present) entry and set address translation and flags.
    //
    /// # Arguments
    //
    /// * `paddr` - The physical memory address this entry shall translate to
    /// * `flags` - Flags from RawPTE (note that the VALID, and AF, and SH flags are set automatically)
    pub fn set_pte(&mut self, paddr: PhysAddr, flags: u64) {
        self.0
            .set(paddr.as_u64() | flags)
            .set_masked_value(RawPTE::SH, pte::shareable::INNER)
            .set_bits(RawPTE::AF)
            .set_bits(RawPTE::VALID);

        unsafe {
            llvm_asm! {"
            dsb ishst
            dc civac, $0
            dsb ish
            isb
            " : : "r"(&self.0 as *const _ as usize)};
        }
    }
}
