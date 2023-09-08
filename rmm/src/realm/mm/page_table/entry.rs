use super::pte;
use crate::mm::address::PhysAddr;
use crate::mm::error::Error;
use crate::mm::page_table::{self, Level};
use crate::mm::RawGPA;
use crate::realm::mm::translation_granule_4k::RawPTE;

use armv9a::bits_in_reg;

#[derive(Clone, Copy)]
pub struct Entry(RawPTE);

impl page_table::Entry for Entry {
    type Inner = RawPTE;

    fn new() -> Self {
        Self(RawPTE::new(0))
    }

    fn is_valid(&self) -> bool {
        self.0.get_masked_value(RawPTE::VALID) != 0
    }

    fn clear(&mut self) {
        self.0 = RawPTE::new(0);
    }

    fn pte(&self) -> u64 {
        self.0.get()
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

    fn set(&mut self, addr: PhysAddr, flags: u64, is_raw: bool) -> Result<(), Error> {
        if is_raw {
            self.0.set(addr.as_u64() | flags);
        } else {
            self.0
                .set(addr.as_u64() | flags)
                .set_masked_value(RawPTE::SH, pte::shareable::INNER)
                .set_bits(RawPTE::AF)
                .set_bits(RawPTE::VALID);
        }

        unsafe {
            core::arch::asm!(
                "dsb ishst",
                "dc civac, {}",
                "dsb ish",
                "isb",
                in(reg) &self.0 as *const _ as usize,
            );
        }
        Ok(())
    }

    fn set_with_page_table_flags(&mut self, addr: PhysAddr) -> Result<(), Error> {
        self.set(
            addr,
            bits_in_reg(RawPTE::ATTR, pte::attribute::NORMAL)
                | bits_in_reg(RawPTE::TYPE, pte::page_type::TABLE_OR_PAGE),
            false,
        )
    }

    fn index<L: Level>(addr: usize) -> usize {
        match L::THIS_LEVEL {
            0 => RawGPA::from(addr).get_masked_value(RawGPA::L0Index) as usize,
            1 => RawGPA::from(addr).get_masked_value(RawGPA::L1Index) as usize,
            2 => RawGPA::from(addr).get_masked_value(RawGPA::L2Index) as usize,
            3 => RawGPA::from(addr).get_masked_value(RawGPA::L3Index) as usize,
            _ => panic!(),
        }
    }

    fn points_to_table_or_page(&self) -> bool {
        match self.is_valid() {
            true => match self.0.get_masked_value(RawPTE::TYPE) {
                pte::page_type::TABLE_OR_PAGE => true,
                pte::page_type::BLOCK => false,
                _ => false,
            },
            false => false,
        }
    }
}
