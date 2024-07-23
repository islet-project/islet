use armv9a::{define_bitfield, define_bits, define_mask};
use vmsa::address::PhysAddr;

use crate::realm::mm::address::GuestPhysAddr;
use crate::realm::mm::attribute::{desc_type, memattr, shareable};
use crate::realm::mm::rtt::{RTT_MIN_BLOCK_LEVEL, RTT_PAGE_LEVEL};
use crate::realm::rd::Rd;
use crate::rmi::error::Error;

pub const INVALID_UNPROTECTED: u64 = 0x0;

pub mod invalid_hipas {
    pub const UNASSIGNED: u64 = 0b00;
    pub const ASSIGNED: u64 = 0b01;
    pub const DESTROYED: u64 = 0b10;
}

pub mod invalid_ripas {
    pub const EMPTY: u64 = 0b0;
    pub const RAM: u64 = 0b1;
    pub const DESTROYED: u64 = 0b10;
}

pub fn mapping_size(level: usize) -> usize {
    match level {
        3 => 1 << S2TTE::ADDR_BLK_L3.trailing_zeros(), // 4096
        2 => 1 << S2TTE::ADDR_BLK_L2.trailing_zeros(),
        1 => 1 << S2TTE::ADDR_BLK_L1.trailing_zeros(),
        0 => 1 << S2TTE::ADDR_BLK_L0.trailing_zeros(),
        _ => unreachable!(),
    }
}

pub fn level_mask(level: usize) -> Option<u64> {
    match level {
        3 => Some(S2TTE::ADDR_BLK_L3),
        2 => Some(S2TTE::ADDR_BLK_L2),
        1 => Some(S2TTE::ADDR_BLK_L1),
        0 => Some(S2TTE::ADDR_BLK_L0),
        _ => None,
    }
}

define_bits!(
    S2TTE,
    NS[55 - 55], // DDI0615A: For a Block or Page descriptor fetched for stage 2 in the Realm Security state, bit 55 is the NS field. if set, it means output address is in NS PAS.
    XN[54 - 54],
    CONT[52 - 52],
    // https://armv8-ref.codingbelief.com/en/chapter_d4/d43_1_vmsav8-64_translation_table_descriptor_formats.html
    ADDR_BLK_L0[47 - 39],      // block descriptor; level 0 w/o concatenation
    ADDR_BLK_L1[47 - 30],      // block descriptor; level 1
    ADDR_BLK_L2[47 - 21],      // block descriptor; level 2
    ADDR_BLK_L3[47 - 12],      // page descriptor; level 3
    ADDR_TBL_OR_PAGE[47 - 12], // table descriptor(level 0-2) || page descriptor(level3)
    AF[10 - 10],
    SH[9 - 8],   // pte_shareable
    S2AP[7 - 6], // pte_access_perm
    INVALID_RIPAS[6 - 5],
    INVALID_HIPAS[4 - 2],
    MEMATTR[5 - 2], // pte_mem_attr
    DESC_TYPE[1 - 0],
    TYPE[1 - 1], // pte_type ; block(0) or table(1)
    VALID[0 - 0]
);

impl S2TTE {
    pub fn get_s2tte(
        rd: &Rd,
        ipa: usize,
        level: usize,
        error_code: Error,
    ) -> Result<(S2TTE, usize), Error> {
        let (s2tte, last_level) = rd
            .s2_table()
            .lock()
            .ipa_to_pte(GuestPhysAddr::from(ipa), level)
            .ok_or(error_code)?;

        Ok((S2TTE::from(s2tte as usize), last_level))
    }

    pub fn is_valid(&self, level: usize, is_ns: bool) -> bool {
        let ns = self.get_masked_value(S2TTE::NS);
        let ns_valid = if is_ns { ns == 1 } else { ns == 0 };
        ns_valid
            && ((level == RTT_PAGE_LEVEL
                && self.get_masked_value(S2TTE::DESC_TYPE) == desc_type::L3_PAGE)
                || (level == RTT_MIN_BLOCK_LEVEL
                    && self.get_masked_value(S2TTE::DESC_TYPE) == desc_type::L012_BLOCK))
    }

    pub fn is_host_ns_valid(&self, level: usize) -> bool {
        let tmp = S2TTE::new(!0);
        let addr_mask = match level {
            1 => tmp.get_masked(S2TTE::ADDR_BLK_L1),
            2 => tmp.get_masked(S2TTE::ADDR_BLK_L2),
            3 => tmp.get_masked(S2TTE::ADDR_BLK_L3),
            _ => return false,
        };
        let mask = addr_mask
            | tmp.get_masked(S2TTE::MEMATTR)
            | tmp.get_masked(S2TTE::S2AP)
            | tmp.get_masked(S2TTE::SH);

        if (self.get() & !mask) != 0 {
            return false;
        }

        if self.get_masked_value(S2TTE::MEMATTR) == memattr::FWB_RESERVED {
            return false;
        }

        if self.get_masked_value(S2TTE::SH) == shareable::RESERVED {
            return false;
        }

        true
    }

    pub fn is_unassigned(&self) -> bool {
        self.get_masked_value(S2TTE::DESC_TYPE) == desc_type::LX_INVALID
            && self.get_masked_value(S2TTE::INVALID_HIPAS) == invalid_hipas::UNASSIGNED
    }

    pub fn is_unassigned_empty(&self) -> bool {
        self.is_unassigned()
            && self.get_masked_value(S2TTE::NS) == 0
            && self.get_masked_value(S2TTE::INVALID_RIPAS) == invalid_ripas::EMPTY
    }

    pub fn is_unassigned_destroyed(&self) -> bool {
        self.is_unassigned()
            && self.get_masked_value(S2TTE::INVALID_RIPAS) == invalid_ripas::DESTROYED
    }

    pub fn is_unassigned_ram(&self) -> bool {
        self.is_unassigned() && self.get_masked_value(S2TTE::INVALID_RIPAS) == invalid_ripas::RAM
    }

    pub fn is_unassigned_ns(&self) -> bool {
        self.is_unassigned() && self.get_masked_value(S2TTE::NS) != 0
    }

    pub fn is_destroyed(&self) -> bool {
        self.get_masked_value(S2TTE::DESC_TYPE) == desc_type::LX_INVALID
            && self.get_masked_value(S2TTE::INVALID_HIPAS) == invalid_hipas::DESTROYED
    }

    pub fn is_assigned(&self) -> bool {
        self.get_masked_value(S2TTE::DESC_TYPE) == desc_type::LX_INVALID
            && self.get_masked_value(S2TTE::INVALID_HIPAS) == invalid_hipas::ASSIGNED
    }

    pub fn is_assigned_empty(&self) -> bool {
        self.is_assigned() && self.get_masked_value(S2TTE::INVALID_RIPAS) == invalid_ripas::EMPTY
    }

    pub fn is_assigned_destroyed(&self) -> bool {
        self.is_assigned()
            && self.get_masked_value(S2TTE::INVALID_RIPAS) == invalid_ripas::DESTROYED
    }

    pub fn is_assigned_ram(&self, level: usize) -> bool {
        if self.get_masked_value(S2TTE::NS) != 0 {
            return false;
        }
        let desc_type = self.get_masked_value(S2TTE::DESC_TYPE);
        if (level == RTT_PAGE_LEVEL && desc_type == desc_type::L3_PAGE)
            || (level == RTT_MIN_BLOCK_LEVEL && desc_type == desc_type::L012_BLOCK)
        {
            return true;
        }
        false
    }

    pub fn is_assigned_ns(&self, level: usize) -> bool {
        if self.get_masked_value(S2TTE::NS) == 0 {
            return false;
        }
        let desc_type = self.get_masked_value(S2TTE::DESC_TYPE);
        if (level == RTT_PAGE_LEVEL && desc_type == desc_type::L3_PAGE)
            || (level == RTT_MIN_BLOCK_LEVEL && desc_type == desc_type::L012_BLOCK)
        {
            return true;
        }
        false
    }

    // level should be the value returned in page table walking
    // (== the last level that has been reached)
    pub fn is_table(&self, level: usize) -> bool {
        (level < RTT_PAGE_LEVEL) && self.get_masked_value(S2TTE::DESC_TYPE) == desc_type::L012_TABLE
    }

    pub fn is_invalid_ripas(&self) -> bool {
        (self.get_masked_value(S2TTE::DESC_TYPE) != desc_type::LX_INVALID)
            && (self.get_ripas() != invalid_ripas::RAM)
    }

    pub fn addr_as_block(&self, level: usize) -> Option<PhysAddr> {
        match level {
            1 => Some(PhysAddr::from(self.get_masked(S2TTE::ADDR_BLK_L1))),
            2 => Some(PhysAddr::from(self.get_masked(S2TTE::ADDR_BLK_L2))),
            3 => Some(PhysAddr::from(self.get_masked(S2TTE::ADDR_BLK_L3))),
            _ => None,
        }
    }

    pub fn get_ripas(&self) -> u64 {
        self.get_masked_value(S2TTE::INVALID_RIPAS)
    }

    pub fn is_live(&self, _level: usize) -> bool {
        self.get_masked_value(S2TTE::DESC_TYPE) != desc_type::LX_INVALID
            || self.is_assigned_empty()
            || self.is_assigned_destroyed()
    }
}
