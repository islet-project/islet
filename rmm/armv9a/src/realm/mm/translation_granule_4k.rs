use crate::{define_bitfield, define_bits, define_mask};

define_bits!(
    RawGPA, // ref. K6.1.2
    L0Index[47 - 39],
    L1Index[38 - 30],
    L2Index[29 - 21],
    L3Index[20 - 12]
);

impl From<usize> for RawGPA {
    fn from(addr: usize) -> Self {
        Self(addr as u64)
    }
}

define_bits!(
    RawPTE,
    XN[54 - 54],
    CONT[52 - 52],
    // https://armv8-ref.codingbelief.com/en/chapter_d4/d43_1_vmsav8-64_translation_table_descriptor_formats.html
    ADDR_BLK_L1[47 - 30],      // block descriptor; level 1
    ADDR_BLK_L2[47 - 21],      // block descriptor; level 2
    ADDR_TBL_OR_PAGE[47 - 12], // table descriptor(level 0-2) || page descriptor(level3)
    AF[10 - 10],
    SH[9 - 8],   // pte_shareable
    S2AP[7 - 6], // pte_access_perm
    NS[5 - 5],
    ATTR[4 - 2], // pte_mem_attr
    TYPE[1 - 1], // pte_type
    VALID[0 - 0]
);
