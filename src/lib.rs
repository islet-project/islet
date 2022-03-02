#![no_std]
#![feature(const_fn)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]
#![feature(const_btree_new)]
#![feature(llvm_asm)]
#![warn(rust_2018_idioms)]
#![feature(specialization)]

pub mod call;
pub mod communication;
pub mod error;
pub mod io;
pub mod r#macro;
pub mod mainloop;
pub mod virt;

#[macro_use(bitflags)]
extern crate bitflags;
