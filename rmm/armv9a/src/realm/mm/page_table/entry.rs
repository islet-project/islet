use monitor::mm::address::PhysAddr;
use monitor::mm::page_table;

use super::super::translation_granule_4k::RawPTE;
use super::pte;

#[derive(Clone, Copy)]
pub struct Entry(RawPTE);

impl page_table::Entry for Entry {
    fn new() -> Self {
        Self(RawPTE::new(0))
    }

    fn is_valid(&self) -> bool {
        self.0.get_masked_value(RawPTE::VALID) != 0
    }

    fn clear(&mut self) {
        self.0 = RawPTE::new(0);
    }

    fn address(&self, level: usize) -> Option<PhysAddr> {
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

    fn set(&mut self, addr: PhysAddr, flags: u64) {
        self.0
            .set(addr.as_u64() | flags)
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
