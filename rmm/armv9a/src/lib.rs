#![no_std]
#![feature(const_fn)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]
#![feature(const_btree_new)]
#![feature(llvm_asm)]
#![feature(alloc_error_handler)]
#![feature(naked_functions)]
#![feature(global_asm)]
#![feature(specialization)]
#![warn(rust_2018_idioms)]

pub mod aarch64;
pub mod allocator;
pub mod config;
pub mod panic;
pub mod realm;
pub mod rmi;
pub mod smc;

extern crate alloc;

#[macro_use(bitflags)]
extern crate bitflags;
