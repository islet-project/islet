pub mod shareable {
    pub const NONE: u64 = 0b00;
    pub const OUTER: u64 = 0b10;
    pub const INNER: u64 = 0b11;
}

pub mod permission {
    pub const RW: u64 = 0b00;
    pub const RO: u64 = 0b10;
}

pub mod page_type {
    pub const BLOCK: u64 = 0b0;
    pub const TABLE_OR_PAGE: u64 = 0b1;
}

pub mod mair_idx {
    pub const RMM_MEM: u64 = 0b0;
    pub const DEVICE_MEM: u64 = 0b1;
}
