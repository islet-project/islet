#![no_std]
#![feature(alloc_error_handler)]
#![feature(asm_const)]
#![feature(const_mut_refs)]
#![warn(rust_2018_idioms)]
#![deny(warnings)]

pub mod allocator;
pub mod config;
pub mod cpu;
pub mod exception;
pub mod gic;
pub mod helper;
pub mod mm;
pub mod panic;
pub mod realm;
pub mod rmm;

extern crate alloc;

#[macro_use]
extern crate log;
