pub mod shareable {
    pub const INNER: u64 = 0b11;
}

pub mod permission {
    pub const RW: u64 = 0b11;
    pub const WO: u64 = 0b10;
    pub const RO: u64 = 0b01;
    pub const NONE: u64 = 0b00;
}

// MemAttr[3:2]:
//      0b01 - Normal, Outer Non-cacheable
//      0b10 - Normal, Outer Write-Through Cacheable
//      0b10 - Normal, Outer Write-Back Cacheable
// MemAttr[1:0]: if MemAttr[3:2] != 0b00
//      0b01 - Inner Non-cacheable
//      0b10 - Inner Write-Through Cacheable
//      0b11 - Inner Write-Back Cacheable
// MemAttr[1:0]: if MemAttr[3:2] == 0b00
//      0b00 - Device-nGnRnE
//      0b01 - Device-nGnRE
//      0b10 - Device-nGRE
//      0b11 - Device-GRE
pub mod attribute {
    pub const NORMAL: u64 = 0b1111;
    pub const NORMAL_NC: u64 = 0b0101;
    pub const DEVICE_NGNRE: u64 = 0b0001;
}

pub mod page_type {
    pub const BLOCK: u64 = 0b0;
    pub const TABLE_OR_PAGE: u64 = 0b1;
}
