use armv9a::{define_bitfield, define_bits, define_mask};

define_bits!(
    RawPTE,
    NS[55 - 55], // DDI0615A: For a Block or Page descriptor fetched for stage 2 in the Realm Security state, bit 55 is the NS field. if set, it means output address is in NS PAS.
    XN[54 - 54],
    CONT[52 - 52],
    // https://armv8-ref.codingbelief.com/en/chapter_d4/d43_1_vmsav8-64_translation_table_descriptor_formats.html
    ADDR_BLK_L0[47 - 39],      // block descriptor; level 0 w/o concatenation
    ADDR_BLK_L1[47 - 30],      // block descriptor; level 1
    ADDR_BLK_L2[47 - 21],      // block descriptor; level 2
    ADDR_BLK_L3[47 - 12],      // block descriptor; level 2
    ADDR_TBL[47 - 12],         // table descriptor; ldevel 0 - 2
    ADDR_TBL_OR_PAGE[47 - 12], // table descriptor(level 0-2) || page descriptor(level3)
    AF[10 - 10],
    SH[9 - 8],   // pte_shareable
    S2AP[7 - 6], // pte_access_perm
    ATTR[5 - 2], // pte_mem_attr
    TYPE[1 - 1], // pte_type ; block(0) or table(1)
    VALID[0 - 0]
);

pub fn mapping_size(level: usize) -> usize {
    match level {
        3 => 1 << RawPTE::ADDR_BLK_L3.trailing_zeros(), // 4096
        2 => 1 << RawPTE::ADDR_BLK_L2.trailing_zeros(),
        1 => 1 << RawPTE::ADDR_BLK_L1.trailing_zeros(),
        0 => 1 << RawPTE::ADDR_BLK_L0.trailing_zeros(),
        _ => unreachable!(),
    }
}
