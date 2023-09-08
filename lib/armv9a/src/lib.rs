#![no_std]
#![warn(rust_2018_idioms)]
#![deny(warnings)]

#[macro_use]
pub mod r#macro;

pub mod regs;
pub use regs::*;

pub const fn bits_in_reg(mask: u64, val: u64) -> u64 {
    (val << (mask.trailing_zeros())) & mask
}
