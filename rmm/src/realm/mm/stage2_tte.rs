use vmsa::address::PhysAddr;

use armv9a::{define_bitfield, define_bits, define_mask};

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

define_bits!(
    S2TTE,
    NS[55 - 55],
    XN[54 - 54],
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
    pub fn is_valid(self, level: usize, is_ns: bool) -> bool {
        let ns = self.get_masked_value(S2TTE::NS);
        let ns_valid = if is_ns { ns == 1 } else { ns == 0 };
        ns_valid
            && ((level == 3 && self.get_masked_value(S2TTE::DESC_TYPE) == desc_type::L3_PAGE)
                || (level == 2 && self.get_masked_value(S2TTE::DESC_TYPE) == desc_type::L012_BLOCK))
    }

    pub fn is_unassigned(self) -> bool {
        self.get_masked_value(S2TTE::INVALID_HIPAS) == invalid_hipas::UNASSIGNED
    }

    pub fn is_destroyed(self) -> bool {
        self.get_masked_value(S2TTE::INVALID_HIPAS) == invalid_hipas::DESTROYED
    }

    pub fn is_assigned(self) -> bool {
        self.get_masked_value(S2TTE::INVALID_HIPAS) == invalid_hipas::ASSIGNED
    }

    // level should be the value returned in page table walking
    // (== the last level that has been reached)
    pub fn is_table(self, level: usize) -> bool {
        (level < 3) && self.get_masked_value(S2TTE::DESC_TYPE) == desc_type::L012_TABLE
    }

    pub fn address(self, level: usize) -> Option<PhysAddr> {
        match level {
            1 => Some(PhysAddr::from(self.get_masked(S2TTE::ADDR_L1_PAGE))),
            2 => Some(PhysAddr::from(self.get_masked(S2TTE::ADDR_L2_PAGE))),
            3 => Some(PhysAddr::from(self.get_masked(S2TTE::ADDR_L3_PAGE))),
            _ => None,
        }
    }
}
