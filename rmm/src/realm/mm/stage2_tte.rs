use core::slice::Iter;

use armv9a::{define_bitfield, define_bits, define_mask};
use vmsa::address::PhysAddr;
use vmsa::page_table::Entry;

use crate::realm::mm::address::GuestPhysAddr;
use crate::realm::mm::attribute::{desc_type, memattr, shareable};
use crate::realm::mm::entry;
use crate::realm::mm::rtt::{RTT_MIN_BLOCK_LEVEL, RTT_PAGE_LEVEL};
use crate::realm::rd::Rd;
use crate::rmi::error::Error;

pub const INVALID_UNPROTECTED: u64 = 0x0;

pub mod invalid_hipas {
    pub const UNASSIGNED: u64 = 0b0;
    pub const ASSIGNED: u64 = 0b1;
}

pub mod invalid_ripas {
    pub const EMPTY: u64 = 0b0;
    pub const RAM: u64 = 0b1;
    pub const DESTROYED: u64 = 0b10;
    pub const DEV: u64 = 0b11;
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
    INVALID_HIPAS[58 - 58], // Host IPA State (HIPAS)
    INVALID_RIPAS[57 - 56], // Realm IPA State (RIPAS)
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
    SH[9 - 8],      // pte_shareable
    S2AP[7 - 6],    // pte_access_perm
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
            && self.get_masked_value(S2TTE::NS) == 0
    }

    pub fn is_unassigned_empty(&self) -> bool {
        self.is_unassigned() && self.get_masked_value(S2TTE::INVALID_RIPAS) == invalid_ripas::EMPTY
    }

    pub fn is_unassigned_destroyed(&self) -> bool {
        self.is_unassigned()
            && self.get_masked_value(S2TTE::INVALID_RIPAS) == invalid_ripas::DESTROYED
    }

    pub fn is_unassigned_ram(&self) -> bool {
        self.is_unassigned() && self.get_masked_value(S2TTE::INVALID_RIPAS) == invalid_ripas::RAM
    }

    pub fn is_unassigned_ns(&self) -> bool {
        self.get_masked_value(S2TTE::DESC_TYPE) == desc_type::LX_INVALID
            && self.get_masked_value(S2TTE::INVALID_HIPAS) == invalid_hipas::UNASSIGNED
            && self.get_masked_value(S2TTE::NS) != 0
    }

    pub fn is_destroyed(&self) -> bool {
        self.get_masked_value(S2TTE::DESC_TYPE) == desc_type::LX_INVALID
            && self.get_masked_value(S2TTE::INVALID_RIPAS) == invalid_ripas::DESTROYED
    }

    pub fn is_assigned(&self) -> bool {
        self.get_masked_value(S2TTE::NS) == 0
            && self.get_masked_value(S2TTE::DESC_TYPE) == desc_type::LX_INVALID
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

    pub fn has_ripas(&self, level: usize) -> bool {
        self.get_masked_value(S2TTE::NS) == 0 && !self.is_table(level)
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

    pub fn addr_as_block(&self, level: usize) -> PhysAddr {
        match level {
            1 => PhysAddr::from(self.get_masked(S2TTE::ADDR_BLK_L1)),
            2 => PhysAddr::from(self.get_masked(S2TTE::ADDR_BLK_L2)),
            3 => PhysAddr::from(self.get_masked(S2TTE::ADDR_BLK_L3)),
            _ => unreachable!(),
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

    // TODO: remvoe mut
    pub fn is_homogeneous(entries: &mut Iter<'_, entry::Entry>, level: usize) -> bool {
        let mut hipas = 0;
        let mut ripas = 0;
        let mut desc_type = 0;
        let mut pa = 0;
        let mut attr: u64 = 0;
        let mut ns = 0;
        let mut first = true;
        let map_size = mapping_size(level) as u64;
        for entry in entries {
            let s2tte = S2TTE::new(entry.pte());
            if first {
                desc_type = s2tte.get_masked_value(S2TTE::DESC_TYPE);
                hipas = s2tte.get_masked_value(S2TTE::INVALID_HIPAS);
                ripas = s2tte.get_masked_value(S2TTE::INVALID_RIPAS);
                ns = s2tte.get_masked_value(S2TTE::NS);
                first = false;
                if s2tte.is_assigned_ns(level) | s2tte.is_assigned_ram(level) | s2tte.is_assigned()
                {
                    // level is 2 or 3
                    if level != 2 && level != 3 {
                        return false;
                    }
                    pa = s2tte.addr_as_block(level).into(); //XXX: check this again
                    attr = entry.pte() & !level_mask(level).unwrap_or(0);
                    // output of first entry is algned to parent's mapping size
                    if (pa & !level_mask(level - 1).unwrap_or(0)) != 0 {
                        return false;
                    }
                }
                continue;
            }

            if desc_type != s2tte.get_masked_value(S2TTE::DESC_TYPE) {
                return false;
            } else if s2tte.is_assigned_ns(level) {
                // addr is contiguous
                pa += map_size;
                if pa != s2tte.addr_as_block(level).into() {
                    return false;
                }
                // attributes are identical
                if attr != s2tte.get() & !level_mask(level).unwrap_or(0) {
                    return false;
                }
            } else if s2tte.is_assigned_ram(level) || s2tte.is_assigned() {
                // addr is contiguous
                pa += map_size;
                if pa != s2tte.addr_as_block(level).into() {
                    return false;
                }
                // ripas is identical
                if ripas != s2tte.get_masked_value(S2TTE::INVALID_RIPAS) {
                    return false;
                }
            } else if s2tte.is_unassigned_ns() {
                // ns is always 1
                if ns != s2tte.get_masked_value(S2TTE::NS) {
                    return false;
                }
                // hipas is always UNASSIGNED
                if hipas != s2tte.get_masked_value(S2TTE::INVALID_HIPAS) {
                    return false;
                }
            } else if s2tte.is_unassigned() {
                // hipas is always UNASSIGNED
                if hipas != s2tte.get_masked_value(S2TTE::INVALID_HIPAS) {
                    return false;
                }
                // ripas is identical
                if ripas != s2tte.get_masked_value(S2TTE::INVALID_RIPAS) {
                    return false;
                }
            }
        }
        true
    }
}
