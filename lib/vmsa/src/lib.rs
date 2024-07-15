#![no_std]
#![warn(rust_2018_idioms)]

pub mod address;
pub mod error;
pub mod guard;
pub mod page;
pub mod page_table;

use armv9a::{define_bitfield, define_bits, define_mask};

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
