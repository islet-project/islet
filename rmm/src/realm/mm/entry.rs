use crate::config::PAGE_SIZE;
use crate::realm::mm::attribute::{desc_type, memattr, page_type, shareable};
use crate::realm::mm::stage2_tte::S2TTE;
use crate::realm::mm::table_level::L3Table;
use vmsa::address::PhysAddr;
use vmsa::error::Error;
use vmsa::page_table::{self, Level};
use vmsa::RawGPA;

#[derive(Clone, Copy)]
pub struct Entry(S2TTE);

impl From<usize> for S2TTE {
    fn from(val: usize) -> Self {
        Self(val as u64)
    }
}

impl page_table::Entry for Entry {
    type Inner = S2TTE;

    fn new() -> Self {
        Self(S2TTE::new(0))
    }

    fn is_valid(&self) -> bool {
        self.0.get_masked_value(S2TTE::VALID) != 0
    }

    fn clear(&mut self) {
        self.0 = S2TTE::new(0);
    }

    fn pte(&self) -> u64 {
        self.0.get()
    }

    fn mut_pte(&mut self) -> &mut Self::Inner {
        self.0.get_mut()
    }

    fn address(&self, level: usize) -> Option<PhysAddr> {
        match self.is_valid() {
            true => match self.0.get_masked_value(S2TTE::TYPE) {
                page_type::TABLE_OR_PAGE => {
                    Some(PhysAddr::from(self.0.get_masked(S2TTE::ADDR_TBL_OR_PAGE)))
                }
                page_type::BLOCK => match level {
                    1 => Some(PhysAddr::from(self.0.get_masked(S2TTE::ADDR_BLK_L1))),
                    2 => Some(PhysAddr::from(self.0.get_masked(S2TTE::ADDR_BLK_L2))),
                    _ => None,
                },
                _ => None,
            },
            false => None,
        }
    }

    fn set(&mut self, addr: PhysAddr, flags: u64) -> Result<(), Error> {
        self.0.set(addr.as_u64() | flags);

        #[cfg(not(any(miri, test)))]
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

    fn point_to_subtable(&mut self, _index: usize, addr: PhysAddr) -> Result<(), Error> {
        let mut flags = S2TTE::new(0);
        flags
            .set_masked_value(S2TTE::DESC_TYPE, desc_type::L012_TABLE)
            .set_masked_value(S2TTE::MEMATTR, memattr::NORMAL_FWB)
            .set_masked_value(S2TTE::SH, shareable::INNER);
        self.set(addr, flags.get())
    }

    fn index<L: Level>(addr: usize) -> usize {
        match L::THIS_LEVEL {
            0 => RawGPA::from(addr).get_masked_value(RawGPA::L0Index) as usize,
            1 => {
                if L::TABLE_SIZE > PAGE_SIZE {
                    // We know that refering one direct parent table is enough
                    // because concatenation of the initial lookup table is upto 16.
                    let l0 = RawGPA::from(addr).get_masked_value(RawGPA::L0Index) as usize;
                    let l1 = RawGPA::from(addr).get_masked_value(RawGPA::L1Index) as usize;
                    // assuming L3Table is a single page-sized
                    l0 * L3Table::NUM_ENTRIES + l1
                } else {
                    RawGPA::from(addr).get_masked_value(RawGPA::L1Index) as usize
                }
            }
            2 => {
                if L::TABLE_SIZE > PAGE_SIZE {
                    let l1 = RawGPA::from(addr).get_masked_value(RawGPA::L1Index) as usize;
                    let l2 = RawGPA::from(addr).get_masked_value(RawGPA::L2Index) as usize;
                    l1 * L3Table::NUM_ENTRIES + l2
                } else {
                    RawGPA::from(addr).get_masked_value(RawGPA::L2Index) as usize
                }
            }
            3 => RawGPA::from(addr).get_masked_value(RawGPA::L3Index) as usize,
            _ => panic!(),
        }
    }

    fn points_to_table_or_page(&self) -> bool {
        match self.is_valid() {
            true => match self.0.get_masked_value(S2TTE::TYPE) {
                page_type::TABLE_OR_PAGE => true,
                page_type::BLOCK => false,
                _ => false,
            },
            false => false,
        }
    }
}
