use core::mem::size_of;
use vmsa::address::PhysAddr;

use super::address::GuestPhysAddr;
use crate::granule::GRANULE_SIZE;
use crate::realm::mm::page_table::pte::{attribute, shareable};
use crate::realm::registry::get_realm;
use crate::rmi::error::{Error, InternalError::NotExistRealm};
use crate::rmi::rtt::{RTT_MIN_BLOCK_LEVEL, RTT_PAGE_LEVEL};
use armv9a::{define_bitfield, define_bits, define_mask};
use vmsa::guard::Content;

pub const INVALID_UNPROTECTED: u64 = 0x0;

pub mod invalid_hipas {
    pub const UNASSIGNED: u64 = 0b00;
    pub const ASSIGNED: u64 = 0b01;
    pub const DESTROYED: u64 = 0b10;
}

pub mod invalid_ripas {
    pub const EMPTY: u64 = 0b0;
    pub const RAM: u64 = 0b1;
}

pub mod desc_type {
    pub const L012_TABLE: u64 = 0x3;
    pub const L012_BLOCK: u64 = 0x1;
    pub const L3_PAGE: u64 = 0x3;
    pub const LX_INVALID: u64 = 0x0;
}

#[repr(C)]
#[derive(Copy, Clone)]
pub struct RttPage([u64; GRANULE_SIZE / size_of::<u64>()]);

impl Content for RttPage {}

impl RttPage {
    pub fn len(&self) -> usize {
        self.0.len()
    }

    pub fn get(&self, index: usize) -> Option<&u64> {
        self.0.get(index)
    }

    pub fn get_mut(&mut self, index: usize) -> Option<&mut u64> {
        self.0.get_mut(index)
    }
}

define_bits!(
    S2TTE,
    NS[55 - 55],
    XN[54 - 54],
    ADDR_L0_PAGE[47 - 39], // XXX: check this again
    ADDR_L1_PAGE[47 - 30], // XXX: check this again
    ADDR_L2_PAGE[47 - 21], // XXX: check this again
    ADDR_L3_PAGE[47 - 12], // XXX: check this again
    ADDR_FULL[55 - 12],
    AF[10 - 10],
    SH[9 - 8],
    AP[7 - 6],
    INVALID_RIPAS[6 - 6],
    INVALID_HIPAS[5 - 2],
    MEMATTR[5 - 2],
    DESC_TYPE[1 - 0],
    PAGE_FLAGS[11 - 0]
);

impl From<usize> for S2TTE {
    fn from(val: usize) -> Self {
        Self(val as u64)
    }
}

impl S2TTE {
    pub fn get_s2tte(
        realm_id: usize,
        ipa: usize,
        level: usize,
        error_code: Error,
    ) -> Result<(S2TTE, usize), Error> {
        let (s2tte, last_level) = get_realm(realm_id)
            .ok_or(Error::RmiErrorOthers(NotExistRealm))?
            .lock()
            .page_table
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
            1 => tmp.get_masked(S2TTE::ADDR_L1_PAGE),
            2 => tmp.get_masked(S2TTE::ADDR_L2_PAGE),
            3 => tmp.get_masked(S2TTE::ADDR_L3_PAGE),
            _ => return false,
        };
        let mask = addr_mask
            | tmp.get_masked(S2TTE::MEMATTR)
            | tmp.get_masked(S2TTE::AP)
            | tmp.get_masked(S2TTE::SH);

        if (self.get() & !mask) != 0 {
            return false;
        }

        if self.get_masked_value(S2TTE::MEMATTR) == attribute::FWB_RESERVED {
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

    pub fn is_destroyed(&self) -> bool {
        self.get_masked_value(S2TTE::DESC_TYPE) == desc_type::LX_INVALID
            && self.get_masked_value(S2TTE::INVALID_HIPAS) == invalid_hipas::DESTROYED
    }

    pub fn is_assigned(&self) -> bool {
        self.get_masked_value(S2TTE::DESC_TYPE) == desc_type::LX_INVALID
            && self.get_masked_value(S2TTE::INVALID_HIPAS) == invalid_hipas::ASSIGNED
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

    pub fn address(&self, level: usize) -> Option<PhysAddr> {
        match level {
            1 => Some(PhysAddr::from(self.get_masked(S2TTE::ADDR_L1_PAGE))),
            2 => Some(PhysAddr::from(self.get_masked(S2TTE::ADDR_L2_PAGE))),
            3 => Some(PhysAddr::from(self.get_masked(S2TTE::ADDR_L3_PAGE))),
            _ => None,
        }
    }

    pub fn get_ripas(&self) -> u64 {
        self.get_masked_value(S2TTE::INVALID_RIPAS)
    }
}
