pub mod shareable {
    pub const INNER: u64 = 0b11;
}

pub mod permission {
    pub const RW: u64 = 0b11;
    pub const WO: u64 = 0b10;
    pub const RO: u64 = 0b01;
    pub const NONE: u64 = 0b00;
}

pub mod attribute {
    pub const NORMAL: u64 = 0b100;
    pub const NORMAL_NC: u64 = 0b011;
    pub const DEVICE_NGNRE: u64 = 0b001;
}

pub mod page_type {
    pub const BLOCK: u64 = 0b0;
    pub const TABLE_OR_PAGE: u64 = 0b1;
}
