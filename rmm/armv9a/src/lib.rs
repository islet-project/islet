#![no_std]
#![feature(const_fn)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]
#![feature(const_btree_new)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(global_asm)]
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
