#![no_std]
#![feature(const_fn)]
#![feature(const_fn_fn_ptr_basics)]
#![feature(const_mut_refs)]
#![feature(const_btree_new)]
#![feature(arc_new_cyclic)]
#![feature(specialization)]
#![allow(incomplete_features)]
#![warn(rust_2018_idioms)]

pub mod call;
pub mod communication;
pub mod config;
pub mod error;
pub mod io;
pub mod logger;
pub mod r#macro;
pub mod mainloop;
pub mod mm;
pub mod realm;

#[macro_use]
extern crate log;
