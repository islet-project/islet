use super::attr;

use attr::mair_idx;
use vmsa::address::PhysAddr;
use vmsa::error::Error;
use vmsa::page_table::{self, Level};
use vmsa::RawGPA;

use armv9a::{define_bitfield, define_bits, define_mask};

define_bits!(
    PTDesc,
    Reserved[58 - 55],
    UXN[54 - 54],
    PXN[53 - 53],
    ADDR_BLK_L1[47 - 30],      // block descriptor; level 1
    ADDR_BLK_L2[47 - 21],      // block descriptor; level 2
    ADDR_TBL_OR_PAGE[47 - 12], // table descriptor(level 0-2) || page descriptor(level3)
    AF[10 - 10],               // access flag
    SH[9 - 8],                 // pte_shareable
    AP[7 - 6],                 // pte_access_perm
    NS[5 - 5],                 // security bit
    INDX[4 - 2],               // the index into the Memory Attribute Indirection Register MAIR_ELn
    TYPE[1 - 1],
    VALID[0 - 0]
);

#[derive(Clone, Copy)]
pub struct Entry(PTDesc);
impl page_table::Entry for Entry {
    type Inner = PTDesc;

    fn new() -> Self {
        Self(PTDesc::new(0))
    }

    fn is_valid(&self) -> bool {
        self.0.get_masked_value(PTDesc::VALID) != 0
    }

    fn clear(&mut self) {
        self.0 = PTDesc::new(0);
    }

    fn pte(&self) -> u64 {
        self.0.get()
    }

    fn mut_pte(&mut self) -> &mut Self::Inner {
        self.0.get_mut()
    }

    fn address(&self, level: usize) -> Option<PhysAddr> {
        match self.is_valid() {
            true => match self.0.get_masked_value(PTDesc::TYPE) {
                attr::page_type::TABLE_OR_PAGE => {
                    Some(PhysAddr::from(self.0.get_masked(PTDesc::ADDR_TBL_OR_PAGE)))
                }
                attr::page_type::BLOCK => match level {
                    1 => Some(PhysAddr::from(self.0.get_masked(PTDesc::ADDR_BLK_L1))),
                    2 => Some(PhysAddr::from(self.0.get_masked(PTDesc::ADDR_BLK_L2))),
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
                .set_masked_value(PTDesc::SH, attr::shareable::INNER)
                .set_bits(PTDesc::AF | PTDesc::VALID);
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
            armv9a::bits_in_reg(PTDesc::INDX, mair_idx::RMM_MEM)
                | armv9a::bits_in_reg(PTDesc::TYPE, attr::page_type::TABLE_OR_PAGE),
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
            true => match self.0.get_masked_value(PTDesc::TYPE) {
                attr::page_type::TABLE_OR_PAGE => true,
                _ => false,
            },
            false => false,
        }
    }
}
