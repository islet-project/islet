pub fn bits_in_reg(mask: u64, val: u64) -> u64 {
    (val << (mask.trailing_zeros())) & mask
}

pub mod tcr_cacheable {
    pub const NONE: u64 = 0b00; // NonCacheable
    pub const WBWA: u64 = 0b01; // Write-Back; Read-Alloc; Write-Alloc
    pub const WTNWA: u64 = 0b10; // Write-thru; Read-Alloc; No Write-Alloc
    pub const WBNWA: u64 = 0b11; // Write-Back; Read-Alloc; No Write-Alloc
}

pub mod tcr_shareable {
    pub const NONE: u64 = 0b00;
    pub const OUTER: u64 = 0b10;
    pub const INNER: u64 = 0b11;
}

pub mod tcr_granule {
    pub const G_4K: u64 = 0b00;
    pub const G_64K: u64 = 0b01;
    pub const G_16K: u64 = 0b10;
}

// Starting level with 4KB granule
pub mod tcr_start_level {
    pub const L2: u64 = 0b00;
    pub const L1: u64 = 0b01;
    pub const L0: u64 = 0b10;
}

pub mod tcr_paddr_size {
    pub const PS_4G: u64 = 0b000; // 32bits
    pub const PS_64G: u64 = 0b001; // 36bits
    pub const PS_1T: u64 = 0b010; // 40bits
    pub const PS_4T: u64 = 0b011; // 42bits
    pub const PS_16T: u64 = 0b100; // 44bits
    pub const PS_256T: u64 = 0b101; // 48bits
    pub const PS_4P: u64 = 0b110; // 52bits
}

// possible AttrIndx values in a Long-descriptor format translation table entry
// for stage 1 translations at EL2
pub mod mair_attr {
    // N: non
    // G: Gathering, R: Reodering, E: Early write-back
    pub const MT_DEVICE_NGNRNE: u64 = 0b0000; // 0x0
    pub const MT_DEVICE_NGNRE: u64 = 0b0100; // 0x4
    pub const MT_DEVICE_GRE: u64 = 0b1100; // 0xc
    pub const MT_NORMAL_NC: u64 = 0b01000100; // 0x44, normal memory, non-cacheable
    pub const MT_NORMAL: u64 = 0b11111111; // 0xff, nomral memory, inner read-alloc, write-alloc,wb, non-transient
}
