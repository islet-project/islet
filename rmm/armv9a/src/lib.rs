#![no_std]
#![feature(alloc_error_handler)]
#![feature(asm_const)]
#![feature(const_btree_new)]
#![feature(const_mut_refs)]
#![feature(naked_functions)]
#![warn(rust_2018_idioms)]

pub mod allocator;
pub mod config;
pub mod cpu;
pub mod exception;
pub mod helper;
pub mod panic;
pub mod realm;
pub mod rmi;
pub mod smc;

extern crate alloc;

#[macro_use]
extern crate log;
